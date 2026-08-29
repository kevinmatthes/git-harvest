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
//! Two passes:  structured commit subjects are harvested into RON fragments,
//! then a second pass assembles them into a RON CHANGELOG with a rendered
//! Markdown sibling.  So far the crate implements `git-harvest init`, which
//! writes a fresh CHANGELOG, and `git-harvest scan`, which harvests a
//! branch's fragment.

mod changelog;
mod cli;
mod git;

pub use crate::{
    changelog::{
        Changelog, Configuration, Entry, Fragment, Grammar, Renderer, Section,
    },
    cli::{Cli, Command, InitArguments, ScanArguments},
};

/// Run `git-harvest` with its arguments already parsed.
///
/// # Errors
///
/// Returns the [`sysexits::ExitCode`] to terminate with; the human-readable
/// reason is printed to standard error at the point of failure.
pub fn run(cli: Cli) -> sysexits::Result<()> {
    match cli.command {
        Command::Init(arguments) => init(&arguments),
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

/******************************************************************************/
