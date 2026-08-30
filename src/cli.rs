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
///
/// `#[non_exhaustive]`:  a new subcommand is a minor release, not a major
/// one — downstream code that matches on this enum must carry a wildcard
/// arm.
#[derive(clap::Subcommand, Debug)]
#[non_exhaustive]
pub enum Command {
    /// Merge the harvested fragments into a new CHANGELOG section.
    Assemble(AssembleArguments),

    /// Write a fresh CHANGELOG carrying the default configuration.
    Init(InitArguments),

    /// Reproduce the licences of `git-harvest` and its dependencies.
    #[command(flatten)]
    Licences(list_my_licence::cli::LicenceCommand),

    /// Render the CHANGELOG as a Keep a Changelog Markdown file.
    Render(RenderArguments),

    /// Harvest this branch's structured commits into a fragment.
    Scan(ScanArguments),
}

/// The arguments of `git-harvest assemble`.
#[derive(clap::Args, Debug)]
pub struct AssembleArguments {
    /// The CHANGELOG to merge the fragments into.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub changelog: std::path::PathBuf,

    /// The directory the fragments are read from and then cleared.
    #[arg(default_value = "changelog.d", long, short)]
    pub input: std::path::PathBuf,

    /// The publish moment, RFC 3339; defaults to now, in UTC.
    #[arg(long, short)]
    pub released: Option<String>,

    /// The version the new section documents, as `major.minor.patch`.
    pub version: String,
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

/// The arguments of `git-harvest render`.
#[derive(clap::Args, Debug)]
pub struct RenderArguments {
    /// The CHANGELOG to read.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub changelog: std::path::PathBuf,

    /// The Markdown file to write; it is always overwritten.
    #[arg(default_value = "CHANGELOG.md", long, short)]
    pub output: std::path::PathBuf,
}

/// The arguments of `git-harvest scan`.
#[derive(clap::Args, Debug)]
pub struct ScanArguments {
    /// The ref the branch diverged from; its merge base bounds the walk.
    #[arg(default_value = "main", long, short)]
    pub base: String,

    /// The CHANGELOG to read the harvest configuration from, if it exists.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub changelog: std::path::PathBuf,

    /// Overwrite the fragment when one of the same name exists already.
    #[arg(long, short)]
    pub force: bool,

    /// The directory to write the fragment into.
    #[arg(default_value = "changelog.d", long, short)]
    pub output: std::path::PathBuf,
}

/******************************************************************************/
