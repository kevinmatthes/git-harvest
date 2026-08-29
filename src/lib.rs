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
//! writes a fresh CHANGELOG carrying the default configuration.

mod changelog;
mod cli;
mod error;

pub use crate::{
    changelog::{Changelog, Configuration, Grammar, Renderer, Section},
    cli::{Cli, Command, InitArguments},
    error::Error,
};

/// Run `git-harvest` with its arguments already parsed.
///
/// # Errors
///
/// Returns [`Error`] when the task fails:  for `init`, when the target exists
/// without `--force`, or cannot be serialised or written.
pub fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Command::Init(arguments) => init(&arguments),
    }
}

/// Write a fresh CHANGELOG holding [`Changelog::default`].
fn init(arguments: &InitArguments) -> Result<(), Error> {
    if arguments.output.exists() && !arguments.force {
        return Err(Error::TargetExists(arguments.output.clone()));
    }

    std::fs::write(&arguments.output, Changelog::default().to_ron()?)
        .map_err(Error::Write)
}

/******************************************************************************/
