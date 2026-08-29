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

//! `git-harvest scan` reads a branch's structured commits into a fragment.

use git_harvest::{Entry, Fragment};
use std::path::Path;
use std::process::Command;

/// Run `git` in `directory` with a fixed identity and no commit signing.
fn git(directory: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .current_dir(directory)
        .args(["-c", "commit.gpgsign=false", "-c", "user.name=Test"])
        .args(["-c", "user.email=test@example.com"])
        .args(arguments)
        .status()
        .expect("git must be on PATH");

    assert!(status.success(), "git {arguments:?} failed");
}

/// An initialised repository with one commit on `main`.
fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();

    git(path, &["init", "-b", "main"]);
    git(path, &["commit", "--allow-empty", "-m", "the root commit"]);

    directory
}

/// Run `git-harvest scan` in `directory` and return its exit success.
fn scan(directory: &Path, extra: &[&str]) -> bool {
    Command::new(env!("CARGO_BIN_EXE_git-harvest"))
        .current_dir(directory)
        .args(["scan", "--base", "main"])
        .args(extra)
        .status()
        .expect("the binary must build")
        .success()
}

/// The single fragment written under `changelog.d/`, parsed.
fn fragment(directory: &Path) -> Fragment {
    let entries: Vec<_> = std::fs::read_dir(directory.join("changelog.d"))
        .expect("changelog.d must exist")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|end| end == "ron"))
        .collect();

    assert_eq!(entries.len(), 1, "exactly one fragment expected");

    ron::from_str(&std::fs::read_to_string(&entries[0]).unwrap()).unwrap()
}

#[test]
fn harvested_entries_are_bucketed_with_their_commit() {
    let repository = repository();
    let path = repository.path();

    git(path, &["checkout", "-b", "enhancement/thing"]);
    git(
        path,
        &["commit", "--allow-empty", "-m", "Added ::= a first thing"],
    );
    git(
        path,
        &["commit", "--allow-empty", "-m", "not a structured change"],
    );
    git(
        path,
        &["commit", "--allow-empty", "-m", "Fixed ::= a second thing"],
    );

    assert!(scan(path, &[]));

    let fragment = fragment(path);

    assert_eq!(fragment.changes.len(), 2);
    assert_eq!(fragment.changes["Added"].len(), 1);
    assert_eq!(fragment.changes["Fixed"].len(), 1);
    assert_eq!(fragment.changes["Added"][0].text(), "a first thing");
    assert_eq!(fragment.changes["Fixed"][0].text(), "a second thing");
    assert_eq!(fragment.changes["Added"][0].commit().unwrap().len(), 7);
}

#[test]
fn a_merge_commit_is_never_harvested() {
    let repository = repository();
    let path = repository.path();

    git(path, &["checkout", "-b", "enhancement/thing"]);
    git(
        path,
        &["commit", "--allow-empty", "-m", "Added ::= the branch work"],
    );
    git(path, &["checkout", "-b", "enhancement/side"]);
    git(
        path,
        &["commit", "--allow-empty", "-m", "Added ::= the side work"],
    );
    git(path, &["checkout", "enhancement/thing"]);
    git(
        path,
        &[
            "merge",
            "--no-ff",
            "enhancement/side",
            "-m",
            "Added ::= the merge commit",
        ],
    );

    assert!(scan(path, &[]));

    let fragment = fragment(path);
    let harvested: Vec<&str> =
        fragment.changes["Added"].iter().map(Entry::text).collect();

    assert!(harvested.contains(&"the branch work"));
    assert!(harvested.contains(&"the side work"));
    assert!(!harvested.contains(&"the merge commit"));
}

#[test]
fn a_branch_with_no_structured_commit_writes_nothing() {
    let repository = repository();
    let path = repository.path();

    git(path, &["checkout", "-b", "documentation/notes"]);
    git(path, &["commit", "--allow-empty", "-m", "just some prose"]);

    assert!(scan(path, &[]));
    assert!(!path.join("changelog.d").exists());
}

#[test]
fn the_bracket_grammar_is_read_from_the_changelog() {
    let repository = repository();
    let path = repository.path();

    std::fs::write(
        path.join("CHANGELOG.ron"),
        "(configuration:(delimiter:\"::=\",grammar:bracketed,buckets:[\
         \"Added\"],fallback_bucket:None,renderer:markdown),introduction:\
         None,references:{},sections:[])\n",
    )
    .unwrap();

    git(path, &["checkout", "-b", "enhancement/thing"]);
    git(
        path,
        &["commit", "--allow-empty", "-m", "[Added] a bracketed thing"],
    );

    assert!(scan(path, &[]));

    assert_eq!(
        fragment(path).changes["Added"][0].text(),
        "a bracketed thing"
    );
}

/******************************************************************************/
