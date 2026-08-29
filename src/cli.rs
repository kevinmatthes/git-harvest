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

//! The command line surface of `git-harvest`.

/// Harvest a CHANGELOG from a repository's Git history.
#[derive(clap::Parser, Debug)]
#[command(about, version)]
pub struct Cli {
    /// The task to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The tasks `git-harvest` can perform.
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Write a fresh CHANGELOG carrying the default configuration.
    Init(InitArguments),
}

/// The arguments of `git-harvest init`.
#[derive(clap::Args, Debug)]
pub struct InitArguments {
    /// The path to write the CHANGELOG to.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub output: std::path::PathBuf,

    /// Overwrite the target when it exists already.
    #[arg(long, short)]
    pub force: bool,
}

/******************************************************************************/
