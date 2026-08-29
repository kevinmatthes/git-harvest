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

//! `git-harvest init` writes a usable default CHANGELOG.

use git_harvest::{Changelog, Cli, Command, InitArguments};

fn init(
    path: std::path::PathBuf,
    force: bool,
) -> Result<(), git_harvest::Error> {
    git_harvest::run(Cli {
        command: Command::Init(InitArguments {
            output: path,
            force,
        }),
    })
}

#[test]
fn the_written_document_round_trips_to_the_default() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("CHANGELOG.ron");

    init(path.clone(), false).unwrap();

    let parsed: Changelog =
        ron::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

    assert_eq!(parsed, Changelog::default());
}

#[test]
fn an_existing_target_is_kept_unless_force_is_given() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("CHANGELOG.ron");

    init(path.clone(), false).unwrap();

    assert!(init(path.clone(), false).is_err());
    assert!(init(path, true).is_ok());
}

#[test]
fn the_default_configuration_is_keep_a_changelog() {
    let configuration = Changelog::default().configuration;
    let buckets: Vec<&str> =
        configuration.buckets.iter().map(String::as_str).collect();

    assert_eq!(configuration.delimiter, "::=");
    assert_eq!(
        buckets,
        [
            "Added",
            "Changed",
            "Deprecated",
            "Fixed",
            "Removed",
            "Security"
        ]
    );
}

/******************************************************************************/
