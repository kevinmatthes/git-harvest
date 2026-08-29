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

use git_harvest::{Changelog, Cli, Command, InitArguments, Section};

fn init(path: std::path::PathBuf, force: bool) -> sysexits::Result<()> {
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

#[test]
fn a_section_with_a_release_moment_survives_a_ron_round_trip() {
    let mut document = Changelog::default();

    document.sections.push(Section {
        version: semver::Version::new(1, 0, 0),
        released: "2026-08-29T12:47:30Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .ok(),
        introduction: None,
        references: std::collections::BTreeMap::new(),
        changes: std::collections::BTreeMap::new(),
    });

    let ron = document.to_ron().unwrap();
    let parsed: Changelog = ron::from_str(&ron).unwrap();

    assert!(parsed.sections[0].released.is_some());
    assert_eq!(parsed, document);
}

/******************************************************************************/
