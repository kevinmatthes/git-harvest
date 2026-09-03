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

//! `git-harvest assemble` merges the fragments into a CHANGELOG section.

use git_harvest::{
    AssembleArguments, Changelog, Cli, Command, Entry, Fragment, InitArguments,
};

/// A test bench:  a temp directory, an initialised CHANGELOG, an input dir.
struct Bench {
    _directory: tempfile::TempDir,
    changelog: std::path::PathBuf,
    input: std::path::PathBuf,
}

impl Bench {
    /// Merge the current fragments for `version`, released at a fixed moment.
    fn assemble(&self, version: &str) -> sysexits::Result<()> {
        git_harvest::run(Cli {
            command: Command::Assemble(AssembleArguments {
                changelog: self.changelog.clone(),
                input: self.input.clone(),
                released: Some("2026-09-01T00:00:00Z".to_owned()),
                version: version.to_owned(),
            }),
        })
    }

    /// The parsed CHANGELOG as it stands on disk.
    fn changelog(&self) -> Changelog {
        ron::from_str(&std::fs::read_to_string(&self.changelog).unwrap())
            .unwrap()
    }

    /// A fresh bench with an initialised CHANGELOG and an empty input dir.
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let changelog = directory.path().join("CHANGELOG.ron");
        let input = directory.path().join("changelog.d");

        git_harvest::run(Cli {
            command: Command::Init(InitArguments {
                output: changelog.clone(),
                force: false,
            }),
        })
        .unwrap();
        std::fs::create_dir(&input).unwrap();

        Self {
            _directory: directory,
            changelog,
            input,
        }
    }

    /// Drop a fragment named `name` carrying `changes` into the input dir.
    fn drop_fragment(&self, name: &str, changes: &[(&str, &[Entry])]) {
        let mut fragment = Fragment::default();

        for (bucket, entries) in changes {
            for entry in *entries {
                fragment.record(bucket, entry.clone());
            }
        }

        std::fs::write(
            self.input.join(format!("{name}.ron")),
            fragment.to_ron().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn fragments_become_one_section_and_are_then_deleted() {
    let bench = Bench::new();

    bench.drop_fragment(
        "2026-09-01T00-00-00Z_one",
        &[("Added", &[Entry::harvested("a first change", "aaaa111")])],
    );
    bench.drop_fragment(
        "2026-09-01T00-00-01Z_two",
        &[("Fixed", &[Entry::harvested("a repair", "bbbb222")])],
    );

    bench.assemble("0.2.0").unwrap();

    let changelog = bench.changelog();

    assert_eq!(changelog.sections.len(), 1);
    assert_eq!(changelog.sections[0].version, semver::Version::new(0, 2, 0));
    assert!(changelog.sections[0].released.is_some());
    assert_eq!(
        changelog.sections[0].changes["Added"][0].text(),
        "a first change"
    );
    assert_eq!(changelog.sections[0].changes["Fixed"][0].text(), "a repair");
    assert_eq!(std::fs::read_dir(&bench.input).unwrap().count(), 0);
}

#[test]
fn a_second_assemble_of_the_same_version_unions_the_entries() {
    let bench = Bench::new();

    bench.drop_fragment(
        "first",
        &[(
            "Added",
            &[Entry::harvested("the earlier change", "aaaa111")],
        )],
    );
    bench.assemble("0.2.0").unwrap();

    bench.drop_fragment(
        "second",
        &[("Added", &[Entry::harvested("the later change", "bbbb222")])],
    );
    bench.assemble("0.2.0").unwrap();

    let changelog = bench.changelog();
    let added: Vec<&str> = changelog.sections[0].changes["Added"]
        .iter()
        .map(Entry::text)
        .collect();

    assert_eq!(changelog.sections.len(), 1);
    assert_eq!(added, ["the earlier change", "the later change"]);
}

#[test]
fn sections_stay_in_descending_version_order() {
    let bench = Bench::new();

    for version in ["0.2.0", "0.4.0", "0.1.0", "0.3.0"] {
        bench.drop_fragment(
            version,
            &[("Changed", &[Entry::harvested("something", "cccc333")])],
        );
        bench.assemble(version).unwrap();
    }

    let versions: Vec<String> = bench
        .changelog()
        .sections
        .iter()
        .map(|section| section.version.to_string())
        .collect();

    assert_eq!(versions, ["0.4.0", "0.3.0", "0.2.0", "0.1.0"]);
}

#[test]
fn the_same_change_credited_apart_folds_into_one_entry() {
    let bench = Bench::new();

    let mut first = Entry::harvested("the fix", "a1b2c3d");
    first.credit("kevinmatthes");
    let mut second = Entry::harvested("the fix", "a1b2c3d");
    second.credit("claude");

    bench.drop_fragment("1_kevinmatthes", &[("Fixed", &[first])]);
    bench.drop_fragment("2_claude", &[("Fixed", &[second])]);

    bench.assemble("0.2.0").unwrap();

    let changelog = bench.changelog();
    let fixed = &changelog.sections[0].changes["Fixed"];

    assert_eq!(fixed.len(), 1);
    assert_eq!(
        fixed[0]
            .aliases
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["kevinmatthes", "claude"],
    );
}

#[test]
fn a_pre_release_version_is_refused_at_write_time() {
    let bench = Bench::new();

    bench.drop_fragment(
        "one",
        &[("Added", &[Entry::harvested("a change", "aaaa111")])],
    );

    assert!(bench.assemble("1.0.0-rc.1").is_err());
}

/******************************************************************************/
