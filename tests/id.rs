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

//! `git-harvest id` registers and maintains the contributor registry.

use git_harvest::{
    Changelog, Cli, Command, IdArguments, IdCommand, MergeArguments,
    RegisterArguments, UpdateArguments,
};

/// A temp directory holding an initialised CHANGELOG.
struct Bench {
    _directory: tempfile::TempDir,
    changelog: std::path::PathBuf,
}

impl Bench {
    /// A fresh bench with an initialised CHANGELOG.
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let changelog = directory.path().join("CHANGELOG.ron");

        git_harvest::run(Cli {
            command: Command::Init(git_harvest::InitArguments {
                output: changelog.clone(),
                force: false,
            }),
        })
        .unwrap();

        Self {
            _directory: directory,
            changelog,
        }
    }

    /// Run one `id` operation against this bench's CHANGELOG.
    fn id(&self, command: IdCommand) -> sysexits::Result<()> {
        git_harvest::run(Cli {
            command: Command::Id(IdArguments {
                changelog: self.changelog.clone(),
                command,
            }),
        })
    }

    /// The parsed CHANGELOG as it stands on disk.
    fn changelog(&self) -> Changelog {
        ron::from_str(&std::fs::read_to_string(&self.changelog).unwrap())
            .unwrap()
    }
}

/// A `RegisterArguments` from its three parts.
fn register(alias: &str, name: &str, email: &str) -> RegisterArguments {
    RegisterArguments {
        alias: alias.to_owned(),
        name: name.to_owned(),
        email: email.to_owned(),
    }
}

/// An `UpdateArguments` for `alias` that changes nothing yet.
fn update(alias: &str) -> UpdateArguments {
    UpdateArguments {
        alias: alias.to_owned(),
        rename: None,
        add_name: Vec::new(),
        remove_name: Vec::new(),
        add_email: Vec::new(),
        remove_email: Vec::new(),
        add_url: Vec::new(),
        remove_url: Vec::new(),
        primary_name: None,
        primary_email: None,
        primary_url: None,
    }
}

#[test]
fn register_adds_a_curated_contributor() {
    let bench = Bench::new();

    bench
        .id(IdCommand::Register(register(
            "claude",
            "Claude",
            "noreply@anthropic.com",
        )))
        .unwrap();

    let contributor = &bench.changelog().contributors["claude"];
    assert_eq!(contributor.primary_name(), Some("Claude"));
    assert_eq!(contributor.primary_email(), Some("noreply@anthropic.com"));
}

#[test]
fn register_refuses_an_e_mail_that_is_already_known() {
    let bench = Bench::new();

    bench
        .id(IdCommand::Register(register("a", "A", "shared@test")))
        .unwrap();

    assert!(
        bench
            .id(IdCommand::Register(register("b", "B", "shared@test")))
            .is_err()
    );
}

#[test]
fn update_adds_removes_and_promotes_in_one_call() {
    let bench = Bench::new();
    bench
        .id(IdCommand::Register(register("k", "Kevin", "k@test")))
        .unwrap();

    let mut change = update("k");
    change.add_url = vec!["https://one.test".to_owned()];
    change.add_name = vec!["kevin".to_owned()];
    change.primary_name = Some("kevin".to_owned());
    bench.id(IdCommand::Update(change)).unwrap();

    let contributor = &bench.changelog().contributors["k"];
    assert_eq!(contributor.primary_name(), Some("kevin"));
    assert_eq!(contributor.primary_url(), Some("https://one.test"));
}

#[test]
fn update_keeps_at_least_one_e_mail() {
    let bench = Bench::new();
    bench
        .id(IdCommand::Register(register("k", "Kevin", "k@test")))
        .unwrap();

    let mut change = update("k");
    change.remove_email = vec!["k@test".to_owned()];

    assert!(bench.id(IdCommand::Update(change)).is_err());
}

#[test]
fn update_refuses_a_rename_onto_a_taken_alias() {
    let bench = Bench::new();
    bench
        .id(IdCommand::Register(register("a", "A", "a@test")))
        .unwrap();
    bench
        .id(IdCommand::Register(register("b", "B", "b@test")))
        .unwrap();

    let mut change = update("a");
    change.rename = Some("b".to_owned());

    assert!(bench.id(IdCommand::Update(change)).is_err());
}

#[test]
fn merge_folds_several_aliases_into_a_fresh_one() {
    let bench = Bench::new();
    bench
        .id(IdCommand::Register(register("a", "A", "a@test")))
        .unwrap();
    bench
        .id(IdCommand::Register(register("b", "B", "b@test")))
        .unwrap();

    bench
        .id(IdCommand::Merge(MergeArguments {
            aliases: vec![
                "a".to_owned(),
                "b".to_owned(),
                "everyone".to_owned(),
            ],
        }))
        .unwrap();

    let contributors = bench.changelog().contributors;
    assert_eq!(contributors.len(), 1);
    let merged = &contributors["everyone"];
    assert_eq!(
        merged.emails.iter().map(String::as_str).collect::<Vec<_>>(),
        ["a@test", "b@test"],
    );
}

/******************************************************************************/
