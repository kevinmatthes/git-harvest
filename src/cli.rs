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
///
/// `git-harvest` produces a Keep a Changelog-style CHANGELOG from a Git
/// repository's commit history in two passes.  `scan` harvests the
/// structured commits on the current branch into a fragment file under
/// `changelog.d/`; `assemble` later folds every pending fragment into a
/// new released section of the CHANGELOG, deleting the fragments it
/// consumes.  `render` exports the released history as Markdown, `id`
/// registers and maintains the contributor registry both passes
/// consult, and `licences` reproduces the licence notices of
/// `git-harvest` and its own dependencies.  With no task given, `scan`
/// runs with its own defaults.
#[derive(clap::Parser, Debug)]
#[command(about, version)]
pub struct Cli {
    /// The task to run; `scan` with its defaults when none is given.
    #[command(subcommand)]
    pub command: Option<Command>,
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
    ///
    /// Reads every fragment file in the input directory, folds duplicate
    /// entries — the same change recorded by more than one fragment —
    /// into a single entry crediting every contributor who reported it,
    /// and writes the result as a new section for the given version.
    /// Every fragment consumed this way is then deleted; a fragment for
    /// a still-unreleased change is left untouched.
    Assemble(AssembleArguments),

    /// Register and maintain the CHANGELOG's contributor registry.
    ///
    /// Wraps four registry operations — `inherit`, `register`, `update`
    /// and `merge` — used to keep the registry accurate as people join,
    /// change their Git identity, or need two recorded identities
    /// folded into one.
    Id(IdArguments),

    /// Write a fresh CHANGELOG carrying the default configuration.
    ///
    /// Writes a starter CHANGELOG carrying the default harvest
    /// configuration — the commit-message grammar, the accepted change
    /// buckets and the default renderer — so a repository has something
    /// for `scan` and `assemble` to read and write to from its very
    /// first commit.  Refuses to overwrite an existing CHANGELOG unless
    /// told to.
    Init(InitArguments),

    /// Reproduce the licences of `git-harvest` and its dependencies.
    #[command(flatten)]
    Licences(list_my_licence::cli::LicenceCommand),

    /// Render the CHANGELOG as a Keep a Changelog Markdown file.
    ///
    /// Renders only released sections — the CHANGELOG's unreleased
    /// state lives in `changelog.d/`'s fragments, never in the
    /// CHANGELOG file itself — as a Keep a Changelog Markdown document
    /// with reference-style contributor links.  The output file is
    /// always overwritten in full; it is a generated artefact, not one
    /// to hand-edit.
    Render(RenderArguments),

    /// Harvest this branch's structured commits into a fragment.
    ///
    /// Walks the commits on the current branch back to its merge base
    /// with `--base`, splits every commit subject on the CHANGELOG's
    /// configured delimiter into a bucket and an entry, and writes the
    /// harvested entries as one fragment file.  A commit with no
    /// delimiter, an unrecognised bucket, or the standing `Skip ::=`
    /// marker is dropped silently; a branch with nothing to harvest
    /// writes no file and exits successfully.
    Scan(ScanArguments),
}

/// The arguments of `git harvest assemble`.
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

/// The arguments of `git harvest id`.
#[derive(clap::Args, Debug)]
pub struct IdArguments {
    /// The CHANGELOG whose contributor registry is edited.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub changelog: std::path::PathBuf,

    /// The registry operation to perform.
    #[command(subcommand)]
    pub command: IdCommand,
}

/// The operations of `git harvest id`.
///
/// `#[non_exhaustive]`:  a new operation is a minor release.
#[derive(clap::Subcommand, Debug)]
#[non_exhaustive]
pub enum IdCommand {
    /// Register the identity from the local Git configuration.
    ///
    /// Reads `user.name` and `user.email` from the repository's local
    /// Git configuration and registers that identity proactively,
    /// rather than waiting for it to be discovered the first time
    /// `scan` harvests one of its commits.
    Inherit,

    /// Register a contributor under a chosen alias.
    ///
    /// Adds a contributor immediately under a chosen alias, for a bot
    /// or a person who has not committed yet and so cannot be
    /// auto-registered by `scan`.
    Register(RegisterArguments),

    /// Change a registered contributor's names, e-mails or URLs.
    ///
    /// Changes an already-registered contributor's names, e-mail
    /// addresses or URLs, promotes one of them to primary, or renames
    /// the contributor's alias — every requested change is validated
    /// together and applied in one atomic write, or none of them are.
    Update(UpdateArguments),

    /// Fold several registered contributors into one.
    ///
    /// Folds two or more registered contributors into one, for the
    /// cases `scan` cannot resolve automatically:  the same person
    /// recorded under different e-mail addresses, or two already
    /// curated aliases found to disagree.  The surviving alias may be
    /// a fresh name, not necessarily one of the merged entries.
    Merge(MergeArguments),
}

/// The arguments of `git harvest id register`.
#[derive(clap::Args, Debug)]
pub struct RegisterArguments {
    /// The alias to credit this contributor as.
    pub alias: String,

    /// The contributor's name.
    pub name: String,

    /// The contributor's e-mail address.
    pub email: String,
}

/// The arguments of `git harvest id update`.
///
/// Adds are applied first, then removes, then primary promotions, then the
/// rename; the request is validated whole and written once or not at all.
#[derive(clap::Args, Debug)]
pub struct UpdateArguments {
    /// The contributor to change.
    pub alias: String,

    /// Rename the contributor; the new alias must be unregistered.
    #[arg(long)]
    pub rename: Option<String>,

    /// Add a name; repeatable.
    #[arg(long)]
    pub add_name: Vec<String>,

    /// Remove a name; repeatable.
    #[arg(long)]
    pub remove_name: Vec<String>,

    /// Add an e-mail address; repeatable.
    #[arg(long)]
    pub add_email: Vec<String>,

    /// Remove an e-mail address; repeatable.
    #[arg(long)]
    pub remove_email: Vec<String>,

    /// Add a URL; repeatable.
    #[arg(long)]
    pub add_url: Vec<String>,

    /// Remove a URL; repeatable.
    #[arg(long)]
    pub remove_url: Vec<String>,

    /// Make this name the primary, adding it if absent.
    #[arg(long)]
    pub primary_name: Option<String>,

    /// Make this e-mail the primary, adding it if absent.
    #[arg(long)]
    pub primary_email: Option<String>,

    /// Make this URL the primary, adding it if absent.
    #[arg(long)]
    pub primary_url: Option<String>,
}

/// The arguments of `git harvest id merge`.
#[derive(clap::Args, Debug)]
pub struct MergeArguments {
    /// The aliases to fold together; the last one names the survivor and
    /// may be a fresh alias.
    #[arg(num_args = 2.., required = true)]
    pub aliases: Vec<String>,
}

/// The arguments of `git harvest init`.
#[derive(clap::Args, Debug)]
pub struct InitArguments {
    /// The path to write the CHANGELOG to.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub output: std::path::PathBuf,

    /// Overwrite the target when it exists already.
    #[arg(long, short)]
    pub force: bool,
}

/// The arguments of `git harvest render`.
#[derive(clap::Args, Debug)]
pub struct RenderArguments {
    /// The CHANGELOG to read.
    #[arg(default_value = "CHANGELOG.ron", long, short)]
    pub changelog: std::path::PathBuf,

    /// The Markdown file to write; it is always overwritten.
    #[arg(default_value = "CHANGELOG.md", long, short)]
    pub output: std::path::PathBuf,
}

/// The arguments of `git harvest scan`.
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

impl Default for ScanArguments {
    /// The same defaults `clap` gives an explicit `git harvest scan`.
    fn default() -> Self {
        Self {
            base: "main".to_owned(),
            changelog: "CHANGELOG.ron".into(),
            force: false,
            output: "changelog.d".into(),
        }
    }
}

/******************************************************************************/
