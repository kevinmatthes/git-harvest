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

/// A name and e-mail pair, as Git records an author or a co-author.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identity {
    /// The name Git carries for this identity.
    pub name: String,

    /// The e-mail address Git carries for this identity.
    pub email: String,
}

/// One commit on the branch, reduced to what the harvest needs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    /// The commit's abbreviated hash, for an entry's provenance.
    pub short_hash: String,

    /// The commit's subject line, trimmed.
    pub subject: String,

    /// The author first, then each `Co-authored-by` trailer, deduplicated.
    pub identities: Vec<Identity>,
}

/// Parse a `Name <e-mail>` string, as a `Co-authored-by` trailer carries it.
fn trailer_identity(raw: &str) -> Option<Identity> {
    let raw = raw.trim();
    let open = raw.rfind('<')?;
    let close = raw[open..].find('>')? + open;
    let email = raw[open + 1..close].trim().to_owned();

    (!email.is_empty()).then(|| Identity {
        name: raw[..open].trim().to_owned(),
        email,
    })
}

/// The author plus every `Co-authored-by` trailer of `message`, in order and
/// without duplicates.
fn identities(
    author: &gix::actor::SignatureRef<'_>,
    body: Option<&gix::bstr::BStr>,
) -> Vec<Identity> {
    let mut found = vec![Identity {
        name: author.name.to_string().trim().to_owned(),
        email: author.email.to_string().trim().to_owned(),
    }];

    for line in body.map(ToString::to_string).unwrap_or_default().lines() {
        if let Some((token, value)) = line.split_once(':')
            && token.trim().eq_ignore_ascii_case("co-authored-by")
            && let Some(identity) = trailer_identity(value)
            && !found.contains(&identity)
        {
            found.push(identity);
        }
    }

    found
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

        let author = commit.author().map_err(|reason| {
            eprintln!("git-harvest:  cannot read a commit author:  {reason}");
            sysexits::ExitCode::Software
        })?;

        commits.push(Commit {
            short_hash: info.id.to_hex_with_len(7).to_string(),
            subject: message.title.to_string().trim().to_owned(),
            identities: identities(&author, message.body),
        });
    }

    Ok(commits)
}

/// The `user.name` and `user.email` of the local Git configuration.
///
/// # Errors
///
/// [`sysexits::ExitCode::Unavailable`] when either value is unset.
pub fn identity() -> sysexits::Result<Identity> {
    let repository = open()?;
    let config = repository.config_snapshot();
    let (Some(name), Some(email)) =
        (config.string("user.name"), config.string("user.email"))
    else {
        eprintln!(
            "git-harvest:  user.name and user.email must both be set in the \
             Git configuration"
        );
        return Err(sysexits::ExitCode::Unavailable);
    };

    Ok(Identity {
        name: name.to_string(),
        email: email.to_string(),
    })
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
