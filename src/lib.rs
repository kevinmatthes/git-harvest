/*********************** GNU General Public License 3.0 ***********************\
|                                                                              |
|  Copyright (C) 2026 Kevin Matthes                                            |
|                                                                              |
|  This program is free software: you can redistribute it and/or modify        |
|  it under the terms of the GNU General Public License as published by        |
|  the Free Software Foundation, either version 3 of the License, or           |
|  (at your option) any later version.                                         |
|                                                                              |
|  This program is distributed in the hope that it will be useful,             |
|  but WITHOUT ANY WARRANTY; without even the implied warranty of              |
|  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the               |
|  GNU General Public License for more details.                                |
|                                                                              |
|  You should have received a copy of the GNU General Public License           |
|  along with this program.  If not, see <https://www.gnu.org/licenses/>.      |
|                                                                              |
\******************************************************************************/

//! Harvest a CHANGELOG from a repository's Git history.
//!
//! Two passes:  `git-harvest scan` harvests a branch's structured commits
//! into a RON fragment, and `git-harvest assemble` merges the fragments into
//! a new section of the RON CHANGELOG.  `git-harvest init` writes a fresh
//! CHANGELOG to start from, `git-harvest render` exports it as Markdown, and
//! `git-harvest licences` reproduces the dependency licence notices.

mod changelog;
mod cli;
mod git;

use crate::changelog::registry;

pub use crate::{
    changelog::{
        Changelog, Configuration, Contributor, Entry, Fragment, Grammar,
        Renderer, Section,
    },
    cli::{
        AssembleArguments, Cli, Command, IdArguments, IdCommand, InitArguments,
        MergeArguments, RegisterArguments, RenderArguments, ScanArguments,
        UpdateArguments,
    },
};

/// Run `git-harvest` with its arguments already parsed.
///
/// # Errors
///
/// Returns the [`sysexits::ExitCode`] to terminate with; the human-readable
/// reason is printed to standard error at the point of failure.
pub fn run(cli: Cli) -> sysexits::Result<()> {
    match cli.command {
        Command::Assemble(arguments) => assemble(&arguments),
        Command::Id(arguments) => id(&arguments),
        Command::Init(arguments) => init(&arguments),
        Command::Licences(command) => {
            licences(&command);
            Ok(())
        }
        Command::Render(arguments) => render(&arguments),
        Command::Scan(arguments) => scan(&arguments),
    }
}

/// The embedded licences of `git-harvest` and its dependencies.
///
/// Written to `OUT_DIR` by the build script and checked against the
/// committed `THIRDPARTY.md` on every continuous-integration build.
static LICENCES: list_my_licence::Attribution = list_my_licence::embed!();

/// Reproduce the licence notices the `licences` subcommand asks for.
fn licences(command: &list_my_licence::cli::LicenceCommand) {
    print!("{}", command.render(&LICENCES));
}

/// Write a fresh CHANGELOG holding [`Changelog::default`].
fn init(arguments: &InitArguments) -> sysexits::Result<()> {
    if arguments.output.exists() && !arguments.force {
        eprintln!(
            "git-harvest:  {} exists already; pass --force to overwrite it",
            arguments.output.display()
        );
        return Err(sysexits::ExitCode::CantCreat);
    }

    let document = Changelog::default().to_ron()?;

    std::fs::write(&arguments.output, document).map_err(|reason| {
        eprintln!(
            "git-harvest:  cannot write {}:  {reason}",
            arguments.output.display()
        );
        sysexits::ExitCode::IoErr
    })
}

/// The harvest configuration to scan with:  the CHANGELOG's, or the default.
fn configuration(
    changelog: &std::path::Path,
) -> sysexits::Result<Configuration> {
    let Ok(source) = std::fs::read_to_string(changelog) else {
        eprintln!(
            "git-harvest:  {} not found; scanning with the default \
             configuration",
            changelog.display()
        );
        return Ok(Configuration::default());
    };

    match ron::from_str::<Changelog>(&source) {
        Ok(document) => Ok(document.configuration),
        Err(reason) => {
            eprintln!(
                "git-harvest:  cannot parse {}:  {reason}",
                changelog.display()
            );
            Err(sysexits::ExitCode::DataErr)
        }
    }
}

/// Harvest this branch's structured commits into a `changelog.d/` fragment.
fn scan(arguments: &ScanArguments) -> sysexits::Result<()> {
    let configuration = configuration(&arguments.changelog)?;
    let repository = git::open()?;
    let commits = git::commits_since(&repository, &arguments.base)?;

    let mut fragment = Fragment::default();

    for commit in &commits {
        if let Some((bucket, text)) = configuration.parse(&commit.subject) {
            let mut entry = Entry::harvested(&text, &commit.short_hash);

            for identity in &commit.identities {
                let alias = registry::register(
                    &mut fragment.contributors,
                    &identity.name,
                    &identity.email,
                );
                entry.credit(&alias);
            }

            fragment.record(&bucket, entry);
        }
    }

    if fragment.is_empty() {
        eprintln!(
            "git-harvest:  no structured commits since {}; wrote nothing",
            arguments.base
        );
        return Ok(());
    }

    let stamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
    let leaf = git::branch_leaf(&repository);
    let path = arguments.output.join(format!("{stamp}_{leaf}.ron"));

    std::fs::create_dir_all(&arguments.output).map_err(|reason| {
        eprintln!(
            "git-harvest:  cannot create {}:  {reason}",
            arguments.output.display()
        );
        sysexits::ExitCode::CantCreat
    })?;

    if path.exists() && !arguments.force {
        eprintln!(
            "git-harvest:  {} exists already; pass --force to overwrite it",
            path.display()
        );
        return Err(sysexits::ExitCode::CantCreat);
    }

    let count: usize = fragment.changes.values().map(Vec::len).sum();
    let noun = if count == 1 { "entry" } else { "entries" };
    let document = fragment.to_ron()?;

    std::fs::write(&path, document).map_err(|reason| {
        eprintln!("git-harvest:  cannot write {}:  {reason}", path.display());
        sysexits::ExitCode::IoErr
    })?;

    eprintln!("git-harvest:  {count} {noun} -> {}", path.display());
    Ok(())
}

/// Render the CHANGELOG as a *Keep a Changelog* Markdown file.
fn render(arguments: &RenderArguments) -> sysexits::Result<()> {
    let changelog = read_changelog(&arguments.changelog)?;
    let released = changelog
        .sections
        .iter()
        .filter(|section| section.released.is_some())
        .count();

    std::fs::write(&arguments.output, changelog.to_markdown()).map_err(
        |reason| {
            eprintln!(
                "git-harvest:  cannot write {}:  {reason}",
                arguments.output.display()
            );
            sysexits::ExitCode::IoErr
        },
    )?;

    let noun = if released == 1 { "section" } else { "sections" };
    eprintln!(
        "git-harvest:  {released} {noun} -> {}",
        arguments.output.display()
    );
    Ok(())
}

/// Read and parse a whole CHANGELOG document.
fn read_changelog(path: &std::path::Path) -> sysexits::Result<Changelog> {
    let source = std::fs::read_to_string(path).map_err(|reason| {
        eprintln!(
            "git-harvest:  cannot read {}:  {reason}; run `git-harvest init` \
             first",
            path.display()
        );
        sysexits::ExitCode::NoInput
    })?;

    ron::from_str(&source).map_err(|reason| {
        eprintln!("git-harvest:  cannot parse {}:  {reason}", path.display());
        sysexits::ExitCode::DataErr
    })
}

/// Every `*.ron` fragment in `directory`, sorted by name.
fn fragment_paths(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut paths: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|end| end == "ron"))
        .collect();

    paths.sort();
    paths
}

/// Sort every bucket's entries and fold ones describing the same change,
/// unioning their credited aliases into the first occurrence.
fn tidy(changes: &mut std::collections::BTreeMap<String, Vec<Entry>>) {
    for entries in changes.values_mut() {
        entries.sort_by(|a, b| {
            a.text.cmp(&b.text).then_with(|| a.commit.cmp(&b.commit))
        });
        entries.dedup_by(|later, kept| {
            if later.text != kept.text || later.commit != kept.commit {
                return false;
            }
            kept.aliases.extend(std::mem::take(&mut later.aliases));
            true
        });
    }
}

/// The credited contributors a fragment or a document keeps, keyed by alias.
type Registry = std::collections::BTreeMap<String, Contributor>;

/// Read the `paths` fragments into one section for `version` at `released`,
/// alongside the contributor registrations they carry.
fn harvested_section(
    version: semver::Version,
    released: chrono::DateTime<chrono::Utc>,
    paths: &[std::path::PathBuf],
) -> sysexits::Result<(Section, Registry)> {
    let mut section = Section {
        version,
        released: Some(released),
        introduction: None,
        references: std::collections::BTreeMap::new(),
        changes: std::collections::BTreeMap::new(),
    };
    let mut contributors = Registry::new();

    for path in paths {
        let source = std::fs::read_to_string(path).map_err(|reason| {
            eprintln!(
                "git-harvest:  cannot read {}:  {reason}",
                path.display()
            );
            sysexits::ExitCode::NoInput
        })?;
        let fragment: Fragment = ron::from_str(&source).map_err(|reason| {
            eprintln!(
                "git-harvest:  cannot parse {}:  {reason}",
                path.display()
            );
            sysexits::ExitCode::DataErr
        })?;

        section.references.extend(fragment.references);
        for (bucket, entries) in fragment.changes {
            section.changes.entry(bucket).or_default().extend(entries);
        }
        for (alias, contributor) in fragment.contributors {
            contributors
                .entry(alias)
                .and_modify(|known| registry::fold(known, &contributor))
                .or_insert(contributor);
        }
    }

    tidy(&mut section.changes);
    Ok((section, contributors))
}

/// Rewrite every credited alias in `section` through `remap`.
fn remap_credits(
    section: &mut Section,
    remap: &std::collections::BTreeMap<String, String>,
) {
    for entries in section.changes.values_mut() {
        for entry in entries {
            entry.aliases = entry
                .aliases
                .iter()
                .map(|alias| remap.get(alias).unwrap_or(alias).clone())
                .collect();
        }
    }
}

/// Serialise `changelog` back to its RON file.
fn write_changelog(
    path: &std::path::Path,
    changelog: &Changelog,
) -> sysexits::Result<()> {
    std::fs::write(path, changelog.to_ron()?).map_err(|reason| {
        eprintln!("git-harvest:  cannot write {}:  {reason}", path.display());
        sysexits::ExitCode::IoErr
    })
}

/// Merge `section` into the CHANGELOG, joining a same-version section or
/// inserting a new one in descending version order.
fn splice(changelog: &mut Changelog, section: Section) {
    if let Some(existing) = changelog
        .sections
        .iter_mut()
        .find(|existing| existing.version == section.version)
    {
        existing.references.extend(section.references);
        for (bucket, entries) in section.changes {
            existing.changes.entry(bucket).or_default().extend(entries);
        }
        existing.released = existing.released.max(section.released);
        tidy(&mut existing.changes);
    } else {
        let at = changelog
            .sections
            .iter()
            .position(|existing| existing.version < section.version)
            .unwrap_or(changelog.sections.len());
        changelog.sections.insert(at, section);
    }
}

/// Merge the harvested fragments into a new CHANGELOG section.
fn assemble(arguments: &AssembleArguments) -> sysexits::Result<()> {
    let version =
        arguments
            .version
            .parse::<semver::Version>()
            .map_err(|reason| {
                eprintln!(
                    "git-harvest:  {:?} is not a version:  {reason}",
                    arguments.version
                );
                sysexits::ExitCode::Usage
            })?;

    let released = match &arguments.released {
        None => chrono::Utc::now(),
        Some(text) => {
            text.parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|reason| {
                    eprintln!(
                        "git-harvest:  {text:?} is not a moment:  {reason}"
                    );
                    sysexits::ExitCode::Usage
                })?
        }
    };

    let mut changelog = read_changelog(&arguments.changelog)?;
    let fragments = fragment_paths(&arguments.input);

    if fragments.is_empty() {
        eprintln!(
            "git-harvest:  no fragments in {}; nothing to assemble",
            arguments.input.display()
        );
        return Ok(());
    }

    let (mut section, contributors) =
        harvested_section(version.clone(), released, &fragments)?;
    let remap = registry::absorb(&mut changelog.contributors, &contributors)?;
    remap_credits(&mut section, &remap);
    for existing in &mut changelog.sections {
        remap_credits(existing, &remap);
    }
    splice(&mut changelog, section);

    write_changelog(&arguments.changelog, &changelog)?;

    for path in &fragments {
        std::fs::remove_file(path).map_err(|reason| {
            eprintln!(
                "git-harvest:  cannot delete {}:  {reason}",
                path.display()
            );
            sysexits::ExitCode::IoErr
        })?;
    }

    eprintln!(
        "git-harvest:  {} fragments -> section {version} of {}",
        fragments.len(),
        arguments.changelog.display()
    );
    Ok(())
}

/// Register or maintain the contributor registry (`git-harvest.md` D57).
fn id(arguments: &IdArguments) -> sysexits::Result<()> {
    let mut changelog = read_changelog(&arguments.changelog)?;

    match &arguments.command {
        IdCommand::Inherit => id_inherit(&mut changelog)?,
        IdCommand::Register(register) => registry::register_curated(
            &mut changelog.contributors,
            &register.alias,
            &register.name,
            &register.email,
        )?,
        IdCommand::Update(update) => id_update(&mut changelog, update)?,
        IdCommand::Merge(merge) => id_merge(&mut changelog, merge)?,
    }

    write_changelog(&arguments.changelog, &changelog)
}

/// Register the identity from the local Git configuration.
fn id_inherit(changelog: &mut Changelog) -> sysexits::Result<()> {
    let identity = git::identity()?;
    let alias = registry::register(
        &mut changelog.contributors,
        &identity.name,
        &identity.email,
    );

    eprintln!("git-harvest:  registered <{}> as {alias:?}", identity.email);
    Ok(())
}

/// Whether an `id update` request asks for any change at all.
fn requests_change(arguments: &UpdateArguments) -> bool {
    arguments.rename.is_some()
        || arguments.primary_name.is_some()
        || arguments.primary_email.is_some()
        || arguments.primary_url.is_some()
        || [
            &arguments.add_name,
            &arguments.remove_name,
            &arguments.add_email,
            &arguments.remove_email,
            &arguments.add_url,
            &arguments.remove_url,
        ]
        .iter()
        .any(|list| !list.is_empty())
}

/// Apply one validated `id update` request (`git-harvest.md` D58).
fn id_update(
    changelog: &mut Changelog,
    arguments: &UpdateArguments,
) -> sysexits::Result<()> {
    if !requests_change(arguments) {
        eprintln!("git-harvest:  nothing to update for {:?}", arguments.alias);
        return Err(sysexits::ExitCode::Usage);
    }

    let Some(original) = changelog.contributors.get(&arguments.alias).cloned()
    else {
        eprintln!("git-harvest:  no contributor {:?}", arguments.alias);
        return Err(sysexits::ExitCode::Usage);
    };

    for (adds, removes, field) in [
        (&arguments.add_name, &arguments.remove_name, "name"),
        (&arguments.add_email, &arguments.remove_email, "e-mail"),
        (&arguments.add_url, &arguments.remove_url, "URL"),
    ] {
        if let Some(clash) = adds.iter().find(|value| removes.contains(value)) {
            eprintln!(
                "git-harvest:  {clash:?} is both added and removed as a \
                 {field}"
            );
            return Err(sysexits::ExitCode::Usage);
        }
    }

    let mut updated = original.clone();

    for name in &arguments.add_name {
        updated.names.insert(name.clone());
    }
    for email in &arguments.add_email {
        updated.emails.insert(email.clone());
    }
    for url in &arguments.add_url {
        updated.urls.insert(url.clone());
    }

    for name in &arguments.remove_name {
        updated.names.shift_remove(name);
    }
    for email in &arguments.remove_email {
        updated.emails.shift_remove(email);
    }
    for url in &arguments.remove_url {
        updated.urls.shift_remove(url);
    }

    if updated.emails.is_empty() {
        eprintln!(
            "git-harvest:  {:?} would keep no e-mail address",
            arguments.alias
        );
        return Err(sysexits::ExitCode::Usage);
    }

    if original.emails.contains(&original.alias)
        && !updated.emails.contains(&original.alias)
        && arguments.rename.is_none()
    {
        eprintln!(
            "git-harvest:  removing <{}> leaves {:?} without the e-mail \
             that names it; pass --rename",
            original.alias, arguments.alias
        );
        return Err(sysexits::ExitCode::Usage);
    }

    if let Some(name) = &arguments.primary_name {
        registry::promote(&mut updated.names, name);
    }
    if let Some(email) = &arguments.primary_email {
        registry::promote(&mut updated.emails, email);
    }
    if let Some(url) = &arguments.primary_url {
        registry::promote(&mut updated.urls, url);
    }

    let key = match &arguments.rename {
        None => arguments.alias.clone(),
        Some(rename) => {
            if rename != &arguments.alias
                && changelog.contributors.contains_key(rename)
            {
                eprintln!(
                    "git-harvest:  {rename:?} is registered already; use \
                     `git harvest id merge`"
                );
                return Err(sysexits::ExitCode::DataErr);
            }
            updated.alias.clone_from(rename);
            rename.clone()
        }
    };

    changelog.contributors.remove(&arguments.alias);
    changelog.contributors.insert(key.clone(), updated);

    if key != arguments.alias {
        let remap = std::iter::once((arguments.alias.clone(), key)).collect();
        for section in &mut changelog.sections {
            remap_credits(section, &remap);
        }
    }

    Ok(())
}

/// Fold several registered contributors into one (`git-harvest.md` D59).
fn id_merge(
    changelog: &mut Changelog,
    arguments: &MergeArguments,
) -> sysexits::Result<()> {
    let split = arguments.aliases.len() - 1;
    let sources = &arguments.aliases[..split];
    let target = arguments.aliases[split].clone();

    for source in sources {
        if !changelog.contributors.contains_key(source) {
            eprintln!("git-harvest:  no contributor {source:?}");
            return Err(sysexits::ExitCode::Usage);
        }
    }

    let mut order: Vec<String> = Vec::new();
    if changelog.contributors.contains_key(&target) {
        order.push(target.clone());
    }
    for source in sources {
        if !order.contains(source) {
            order.push(source.clone());
        }
    }

    let mut merged = changelog.contributors[&order[0]].clone();
    merged.alias.clone_from(&target);
    for alias in &order[1..] {
        let taken = changelog.contributors[alias].clone();
        registry::fold(&mut merged, &taken);
    }

    for alias in &order {
        changelog.contributors.remove(alias);
    }
    changelog.contributors.insert(target.clone(), merged);

    let remap: std::collections::BTreeMap<String, String> = order
        .iter()
        .map(|alias| (alias.clone(), target.clone()))
        .collect();
    for section in &mut changelog.sections {
        remap_credits(section, &remap);
    }

    Ok(())
}

/******************************************************************************/
