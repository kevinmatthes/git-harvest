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

//! A CHANGELOG and its fragments can be written in YAML as well as in RON.

use git_harvest::{
    AssembleArguments, Changelog, Cli, Command, Entry, Format, Fragment,
    IdArguments, IdCommand, InitArguments, RegisterArguments, RenderArguments,
    Section,
};
use std::path::Path;

/// Run one subcommand in process.
fn run(command: Command) -> sysexits::Result<()> {
    git_harvest::run(Cli {
        command: Some(command),
    })
}

/// Write a fresh CHANGELOG to `path`.
fn init(path: &Path) -> sysexits::Result<()> {
    run(Command::Init(InitArguments {
        output: path.to_path_buf(),
        force: false,
    }))
}

/// A document with one released section holding a single entry.
fn document() -> Changelog {
    let mut document = Changelog::default();
    let mut section = Section {
        version: semver::Version::new(1, 0, 0),
        released: "2026-09-01T00:00:00Z".parse().ok(),
        introduction: None,
        references: std::collections::BTreeMap::new(),
        changes: std::collections::BTreeMap::new(),
    };

    section
        .changes
        .insert("Added".to_owned(), vec![Entry::authored("a thing")]);
    document.sections.push(section);
    document
}

/// Render the CHANGELOG at `path` to Markdown and return the Markdown.
fn markdown(path: &Path, output: &Path) -> String {
    run(Command::Render(RenderArguments {
        changelog: path.to_path_buf(),
        output: output.to_path_buf(),
    }))
    .unwrap();

    std::fs::read_to_string(output).unwrap()
}

#[test]
fn the_format_follows_the_extension() {
    assert_eq!(Format::of(Path::new("a.ron")), Some(Format::Ron));
    assert_eq!(Format::of(Path::new("a.yaml")), Some(Format::Yaml));
    assert_eq!(Format::of(Path::new("a.yml")), Some(Format::Yaml));
    assert_eq!(Format::of(Path::new("a.json")), None);
    assert_eq!(Format::of(Path::new("a")), None);
    assert_eq!(Format::Ron.extension(), "ron");
    assert_eq!(Format::Yaml.extension(), "yaml");
}

#[test]
fn init_writes_yaml_for_a_yaml_path() {
    let directory = tempfile::tempdir().unwrap();

    for name in ["CHANGELOG.yaml", "CHANGELOG.yml"] {
        let path = directory.path().join(name);

        init(&path).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let parsed: Changelog = Format::Yaml.parse(&text).unwrap();

        assert_eq!(parsed, Changelog::default());
        assert!(!text.starts_with('('));
    }
}

#[test]
fn init_refuses_an_unsupported_extension() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("CHANGELOG.json");

    assert_eq!(init(&path), Err(sysexits::ExitCode::Usage));
    assert!(!path.exists());
}

#[test]
fn assemble_folds_ron_and_yaml_fragments_into_a_yaml_changelog() {
    let directory = tempfile::tempdir().unwrap();
    let changelog = directory.path().join("CHANGELOG.yaml");
    let input = directory.path().join("changelog.d");
    let mut first = Fragment::default();
    let mut second = Fragment::default();

    init(&changelog).unwrap();
    std::fs::create_dir(&input).unwrap();
    first.record("Added", Entry::authored("from ron"));
    second.record("Added", Entry::authored("from yaml"));
    std::fs::write(input.join("a.ron"), first.to_ron().unwrap()).unwrap();
    std::fs::write(input.join("b.yaml"), second.to_yaml().unwrap()).unwrap();
    std::fs::write(input.join("c.txt"), "left alone").unwrap();

    run(Command::Assemble(AssembleArguments {
        changelog: changelog.clone(),
        input: input.clone(),
        released: Some("2026-09-01T00:00:00Z".to_owned()),
        version: "1.0.0".to_owned(),
    }))
    .unwrap();

    let text = std::fs::read_to_string(&changelog).unwrap();
    let parsed: Changelog = Format::Yaml.parse(&text).unwrap();

    assert_eq!(parsed.sections[0].changes["Added"].len(), 2);
    assert!(!input.join("a.ron").exists());
    assert!(!input.join("b.yaml").exists());
    assert!(input.join("c.txt").exists());
}

#[test]
fn render_gives_the_same_markdown_from_either_format() {
    let directory = tempfile::tempdir().unwrap();
    let ron = directory.path().join("CHANGELOG.ron");
    let yaml = directory.path().join("CHANGELOG.yaml");

    std::fs::write(&ron, document().to_ron().unwrap()).unwrap();
    std::fs::write(&yaml, document().to_yaml().unwrap()).unwrap();

    let from_ron = markdown(&ron, &directory.path().join("ron.md"));
    let from_yaml = markdown(&yaml, &directory.path().join("yaml.md"));

    assert!(from_ron.contains("## [1.0.0]"));
    assert_eq!(from_ron, from_yaml);
}

#[test]
fn id_keeps_a_yaml_changelog_in_yaml() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("CHANGELOG.yaml");

    init(&path).unwrap();
    run(Command::Id(IdArguments {
        changelog: path.clone(),
        command: IdCommand::Register(RegisterArguments {
            alias: "ada".to_owned(),
            name: "Ada Lovelace".to_owned(),
            email: "ada@example.org".to_owned(),
        }),
    }))
    .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    let parsed: Changelog = Format::Yaml.parse(&text).unwrap();

    assert!(parsed.contributors.contains_key("ada"));
    assert!(!text.starts_with('('));
}

#[test]
fn the_repository_changelog_survives_a_conversion_to_yaml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.ron");
    let original: Changelog = Format::Ron
        .parse(&std::fs::read_to_string(path).unwrap())
        .unwrap();
    let yaml = original.to_yaml().unwrap();
    let back: Changelog = Format::Yaml.parse(&yaml).unwrap();

    assert_eq!(back, original);
}

#[test]
fn text_that_looks_like_other_scalars_survives_yaml() {
    let lookalikes = [
        "1234567", "1e300000", "0123456", "0x1f2e3d", "null", "yes", "~",
        "12:30", "1_000", "-", "  lead", "a: b", "# c", "",
    ];

    for text in lookalikes {
        let mut fragment = Fragment::default();

        fragment.record("Added", Entry::harvested(text, text));

        let yaml = fragment.to_yaml().unwrap();
        let back: Fragment = Format::Yaml.parse(&yaml).unwrap();

        assert_eq!(back, fragment);
    }
}

#[test]
fn a_hand_written_yaml_document_parses_and_a_short_version_is_refused() {
    let good = "\
configuration:
  delimiter: '::='
  grammar: delimited
  buckets: [Added]
  fallback_bucket: null
  renderer: markdown
introduction: null
references: {}
sections:
  - version: [1, 2, 3]
    released: 2026-09-01T00:00:00Z
    introduction: null
    references: {}
    changes: {}
";
    let bad = good.replace("[1, 2, 3]", "[1, 2]");
    let parsed: Changelog = Format::Yaml.parse(good).unwrap();

    assert_eq!(parsed.sections[0].version, semver::Version::new(1, 2, 3));
    assert!(Format::Yaml.parse::<Changelog>(&bad).is_err());
}

/******************************************************************************/
