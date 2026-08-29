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
//! CHANGELOG to start from, and `git-harvest render` exports it as Markdown.

mod changelog;
mod cli;
mod git;

pub use crate::{
    changelog::{
        Changelog, Configuration, Entry, Fragment, Grammar, Renderer, Section,
    },
    cli::{
        AssembleArguments, Cli, Command, InitArguments, RenderArguments,
        ScanArguments,
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
        Command::Init(arguments) => init(&arguments),
        Command::Render(arguments) => render(&arguments),
        Command::Scan(arguments) => scan(&arguments),
    }
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
            fragment
                .record(&bucket, Entry::harvested(&text, &commit.short_hash));
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

/// Sort and deduplicate every bucket's entries.
fn tidy(changes: &mut std::collections::BTreeMap<String, Vec<Entry>>) {
    for entries in changes.values_mut() {
        entries.sort();
        entries.dedup();
    }
}

/// Read the `paths` fragments into one section for `version` at `released`.
fn harvested_section(
    version: semver::Version,
    released: chrono::DateTime<chrono::Utc>,
    paths: &[std::path::PathBuf],
) -> sysexits::Result<Section> {
    let mut section = Section {
        version,
        released: Some(released),
        introduction: None,
        references: std::collections::BTreeMap::new(),
        changes: std::collections::BTreeMap::new(),
    };

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
    }

    tidy(&mut section.changes);
    Ok(section)
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

    let section = harvested_section(version.clone(), released, &fragments)?;
    splice(&mut changelog, section);

    std::fs::write(&arguments.changelog, changelog.to_ron()?).map_err(
        |reason| {
            eprintln!(
                "git-harvest:  cannot write {}:  {reason}",
                arguments.changelog.display()
            );
            sysexits::ExitCode::IoErr
        },
    )?;

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

/******************************************************************************/
