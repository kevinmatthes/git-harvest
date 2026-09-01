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

//! `git-harvest render` exports the CHANGELOG as Keep a Changelog Markdown.

use git_harvest::{
    Changelog, Cli, Command, Entry, InitArguments, RenderArguments, Section,
};

/// A section for `version`, released on 2026-09-01, with the given changes.
fn section(version: &str, changes: &[(&str, &[Entry])]) -> Section {
    let mut map = std::collections::BTreeMap::new();

    for (bucket, entries) in changes {
        map.insert((*bucket).to_owned(), entries.to_vec());
    }

    Section {
        version: version.parse().unwrap(),
        released: "2026-09-01T00:00:00Z".parse().ok(),
        introduction: None,
        references: std::collections::BTreeMap::new(),
        changes: map,
    }
}

#[test]
fn a_released_section_renders_in_keep_a_changelog_form() {
    let mut changelog = Changelog::default();
    changelog.sections.push(section(
        "0.2.0",
        &[
            ("Added", &[Entry::authored("a new flag")]),
            ("Fixed", &[Entry::authored("a crash")]),
        ],
    ));

    let markdown = changelog.to_markdown();

    assert!(markdown.starts_with("# Changelog\n"));
    assert!(markdown.contains("## [0.2.0] - 2026-09-01\n"));
    assert!(markdown.contains("### Added\n\n- a new flag\n"));
    assert!(markdown.contains("### Fixed\n\n- a crash\n"));
}

#[test]
fn a_pending_section_is_left_out() {
    let mut changelog = Changelog::default();
    let mut pending = section("0.3.0", &[("Added", &[Entry::authored("x")])]);
    pending.released = None;
    changelog.sections.push(pending);

    assert!(!changelog.to_markdown().contains("0.3.0"));
}

#[test]
fn buckets_follow_the_configuration_order() {
    let mut changelog = Changelog::default();
    changelog.configuration.buckets =
        vec!["Fixed".to_owned(), "Added".to_owned()];
    changelog.sections.push(section(
        "0.2.0",
        &[
            ("Added", &[Entry::authored("an addition")]),
            ("Fixed", &[Entry::authored("a repair")]),
        ],
    ));

    let markdown = changelog.to_markdown();
    let fixed = markdown.find("### Fixed").unwrap();
    let added = markdown.find("### Added").unwrap();

    assert!(fixed < added);
}

#[test]
fn an_entry_renders_its_text_without_the_commit() {
    let mut changelog = Changelog::default();
    changelog.sections.push(section(
        "0.2.0",
        &[("Added", &[Entry::harvested("some change", "abc1234")])],
    ));

    let markdown = changelog.to_markdown();

    assert!(markdown.contains("- some change\n"));
    assert!(!markdown.contains("abc1234"));
}

#[test]
fn render_writes_the_markdown_beside_the_ron() {
    let directory = tempfile::tempdir().unwrap();
    let ron = directory.path().join("CHANGELOG.ron");
    let markdown = directory.path().join("CHANGELOG.md");

    git_harvest::run(Cli {
        command: Command::Init(InitArguments {
            output: ron.clone(),
            force: false,
        }),
    })
    .unwrap();

    git_harvest::run(Cli {
        command: Command::Render(RenderArguments {
            changelog: ron,
            output: markdown.clone(),
        }),
    })
    .unwrap();

    assert!(
        std::fs::read_to_string(&markdown)
            .unwrap()
            .starts_with("# Changelog")
    );
}

/******************************************************************************/
