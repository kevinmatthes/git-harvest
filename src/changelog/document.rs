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

//! The CHANGELOG document:  the root of the RON source of truth.

/// The fixed header every rendered CHANGELOG opens with.
const PREAMBLE: &str = "# Changelog\n\n\
All notable changes to this project are documented in this file.\n\n\
The format follows \
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project \
adheres to \
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n";

/// The whole CHANGELOG, as read from and written to `CHANGELOG.ron`.
///
/// The document carries its own [`crate::Configuration`], so a repository
/// needs no configuration file beside it.  `sections` is newest first.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Changelog {
    /// The harvest settings for this repository.
    pub configuration: crate::Configuration,

    /// The contributors credited across this document, keyed by alias.
    #[serde(default)]
    pub contributors: std::collections::BTreeMap<String, crate::Contributor>,

    /// An optional prose lead for the whole document.
    pub introduction: Option<String>,

    /// Link labels shared across sections, mapped to their targets.
    pub references: std::collections::BTreeMap<String, String>,

    /// The version sections, newest first.
    pub sections: Vec<crate::Section>,
}

impl Changelog {
    /// Serialise the document to pretty RON, indented two spaces.
    ///
    /// # Errors
    ///
    /// Returns [`sysexits::ExitCode::Software`] if the document cannot be
    /// represented as RON — which should not happen for a value built by this
    /// crate — after printing the reason to standard error.
    pub fn to_ron(&self) -> sysexits::Result<String> {
        let pretty = ron::ser::PrettyConfig::new().indentor("  ".to_owned());

        match ron::ser::to_string_pretty(self, pretty) {
            Ok(body) => Ok(format!("{body}\n")),
            Err(reason) => {
                eprintln!(
                    "git-harvest:  cannot serialise the CHANGELOG:  {reason}"
                );
                Err(sysexits::ExitCode::Software)
            }
        }
    }

    /// Render the document as a *Keep a Changelog* Markdown file.
    ///
    /// Only released sections appear:  the loose fragments are the pending
    /// state and are not shown (`git-harvest.md` D21).  Buckets render in the
    /// configuration's order, then any others alphabetically.  An entry's
    /// commit is provenance for the RON, not for readers, so it is left out.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::from(PREAMBLE);

        if let Some(lead) = &self.introduction {
            out.push_str(lead);
            out.push_str("\n\n");
        }

        for section in &self.sections {
            if section.released.is_some() {
                out.push_str(&section_markdown(&self.configuration, section));
            }
        }

        out.push_str(&reference_block(self));
        format!("{}\n", out.trim_end())
    }
}

/// One bucket's heading and its entries, or nothing when it is empty.
fn bucket_markdown(bucket: &str, entries: &[crate::Entry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut out = format!("### {bucket}\n\n");

    for entry in entries {
        out.push_str("- ");
        out.push_str(entry.text());
        out.push('\n');
    }

    out.push('\n');
    out
}

/// The trailing `[label]: url` block:  document references, then any the
/// released sections add.
fn reference_block(changelog: &Changelog) -> String {
    let mut references = changelog.references.clone();

    for section in &changelog.sections {
        if section.released.is_some() {
            for (label, url) in &section.references {
                references
                    .entry(label.clone())
                    .or_insert_with(|| url.clone());
            }
        }
    }

    let mut out = String::new();

    for (label, url) in &references {
        out.push('[');
        out.push_str(label);
        out.push_str("]: ");
        out.push_str(url);
        out.push('\n');
    }

    out
}

/// One released section:  its heading, lead and buckets.
fn section_markdown(
    configuration: &crate::Configuration,
    section: &crate::Section,
) -> String {
    let date = section
        .released
        .map(|moment| moment.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let mut out = format!(
        "## [{}.{}.{}] - {date}\n\n",
        section.version.major, section.version.minor, section.version.patch,
    );

    if let Some(lead) = &section.introduction {
        out.push_str(lead);
        out.push_str("\n\n");
    }

    let mut seen = std::collections::BTreeSet::new();

    for bucket in &configuration.buckets {
        if let Some(entries) = section.changes.get(bucket) {
            out.push_str(&bucket_markdown(bucket, entries));
            seen.insert(bucket);
        }
    }

    for (bucket, entries) in &section.changes {
        if !seen.contains(bucket) {
            out.push_str(&bucket_markdown(bucket, entries));
        }
    }

    out
}

/******************************************************************************/
