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

//! The one place `git-harvest` touches Git.
//!
//! Everything here is read-only and goes through `gix`'s high-level API, so
//! a later change of library stays contained to this file (`git-harvest.md`
//! D27).

/// One commit on the branch, reduced to what the harvest needs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    /// The commit's abbreviated hash, for an entry's provenance.
    pub short_hash: String,

    /// The commit's subject line, trimmed.
    pub subject: String,
}

/// The leaf of the current branch's name, or `HEAD` when detached.
#[must_use]
pub fn branch_leaf(repository: &gix::Repository) -> String {
    repository.head_name().ok().flatten().map_or_else(
        || "HEAD".to_owned(),
        |name| {
            let short = name.shorten().to_string();
            short.rsplit('/').next().unwrap_or(&short).to_owned()
        },
    )
}

/// The non-merge commits reachable from `HEAD` but not from `base`.
///
/// The walk stops at the merge base of `HEAD` and `base`, so only the work
/// added on this branch is returned, newest first.
///
/// # Errors
///
/// [`sysexits::ExitCode::Usage`] when `HEAD` or `base` cannot be resolved,
/// [`sysexits::ExitCode::Unavailable`] when the two share no history, and
/// [`sysexits::ExitCode::Software`] when the object database cannot be read.
pub fn commits_since(
    repository: &gix::Repository,
    base: &str,
) -> sysexits::Result<Vec<Commit>> {
    let head = repository.head_commit().map_err(|reason| {
        eprintln!("git-harvest:  cannot read HEAD:  {reason}");
        sysexits::ExitCode::Usage
    })?;

    let base = repository.rev_parse_single(base).map_err(|reason| {
        eprintln!("git-harvest:  cannot resolve {base:?}:  {reason}");
        sysexits::ExitCode::Usage
    })?;

    let boundary =
        repository
            .merge_base(head.id(), base.detach())
            .map_err(|reason| {
                eprintln!(
                    "git-harvest:  no shared history with the base:  {reason}"
                );
                sysexits::ExitCode::Unavailable
            })?;

    let walk = repository
        .rev_walk([head.id().detach()])
        .with_boundary([boundary.detach()])
        .all()
        .map_err(|reason| {
            eprintln!("git-harvest:  cannot walk the history:  {reason}");
            sysexits::ExitCode::Software
        })?;

    let mut commits = Vec::new();

    for step in walk {
        let info = step.map_err(|reason| {
            eprintln!("git-harvest:  the history walk failed:  {reason}");
            sysexits::ExitCode::Software
        })?;

        if info.parent_ids.len() > 1 {
            continue;
        }

        let commit = repository.find_commit(info.id).map_err(|reason| {
            eprintln!("git-harvest:  cannot read a commit:  {reason}");
            sysexits::ExitCode::Software
        })?;

        let message = commit.message().map_err(|reason| {
            eprintln!("git-harvest:  cannot read a commit message:  {reason}");
            sysexits::ExitCode::Software
        })?;

        commits.push(Commit {
            short_hash: info.id.to_hex_with_len(7).to_string(),
            subject: message.title.to_string().trim().to_owned(),
        });
    }

    Ok(commits)
}

/// Open the repository containing the working directory.
///
/// # Errors
///
/// [`sysexits::ExitCode::Usage`] when the working directory is not inside a
/// Git repository.
pub fn open() -> sysexits::Result<gix::Repository> {
    gix::discover(".").map_err(|reason| {
        eprintln!("git-harvest:  not inside a Git repository:  {reason}");
        sysexits::ExitCode::Usage
    })
}

/******************************************************************************/
