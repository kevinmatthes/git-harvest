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
#[non_exhaustive]
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
    /// commit is provenance for the RON, not for readers, so it is left out;
    /// its credited contributors are shown inline and gathered into a
    /// `### Contributors` block, linked through the trailing reference list.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::from(PREAMBLE);
        let mut references = self.references.clone();

        if let Some(lead) = &self.introduction {
            out.push_str(lead);
            out.push_str("\n\n");
        }

        for section in &self.sections {
            if section.released.is_some() {
                out.push_str(&section_markdown(
                    &self.configuration,
                    section,
                    &self.contributors,
                    &mut references,
                ));
            }
        }

        for section in &self.sections {
            if section.released.is_some() {
                for (label, url) in &section.references {
                    references
                        .entry(label.clone())
                        .or_insert_with(|| url.clone());
                }
            }
        }

        out.push_str(&reference_block(&references));
        format!("{}\n", out.trim_end())
    }
}

/// The contributor registry, keyed by alias.
type Registry = std::collections::BTreeMap<String, crate::Contributor>;

/// The link-reference definitions gathered while rendering, keyed by label.
type References = std::collections::BTreeMap<String, String>;

/// How one credited `alias` reads:  `@alias` for a curated alias, or the
/// contributor's primary name — falling back to the bare alias — while it is
/// still keyed by a raw e-mail.
fn credit_label(alias: &str, contributors: &Registry) -> String {
    if !alias.contains('@') {
        return format!("@{alias}");
    }

    contributors
        .get(alias)
        .and_then(crate::Contributor::primary_name)
        .map_or_else(|| alias.to_owned(), ToOwned::to_owned)
}

/// The inline token for one credited `alias`:  the [`credit_label`] wrapped
/// as a reference link when the contributor has a URL — recorded in
/// `references` — or left bare.
fn credit_token(
    alias: &str,
    contributors: &Registry,
    references: &mut References,
) -> String {
    let label = credit_label(alias, contributors);

    contributors
        .get(alias)
        .and_then(crate::Contributor::primary_url)
        .map_or_else(
            || label.clone(),
            |url| {
                references
                    .entry(label.clone())
                    .or_insert_with(|| url.to_owned());
                format!("[{label}]")
            },
        )
}

/// The buckets of `section`, configuration order first, then any others.
fn ordered_buckets<'a>(
    configuration: &'a crate::Configuration,
    section: &'a crate::Section,
) -> Vec<&'a str> {
    let mut seen = std::collections::BTreeSet::new();
    let mut order = Vec::new();

    for bucket in &configuration.buckets {
        if section.changes.contains_key(bucket) {
            order.push(bucket.as_str());
            seen.insert(bucket.as_str());
        }
    }

    for bucket in section.changes.keys() {
        if !seen.contains(bucket.as_str()) {
            order.push(bucket.as_str());
        }
    }

    order
}

/// The trailing `[label]: url` block.
fn reference_block(references: &References) -> String {
    let mut out = String::new();

    for (label, url) in references {
        out.push('[');
        out.push_str(label);
        out.push_str("]: ");
        out.push_str(url);
        out.push('\n');
    }

    out
}

/// One bucket's heading and its entries, or nothing when it is empty.
fn bucket_markdown(
    bucket: &str,
    entries: &[crate::Entry],
    contributors: &Registry,
    references: &mut References,
) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut out = format!("### {bucket}\n\n");

    for entry in entries {
        out.push_str("- ");
        out.push_str(entry.text());

        if !entry.aliases.is_empty() {
            let credit: Vec<String> = entry
                .aliases
                .iter()
                .map(|alias| credit_token(alias, contributors, references))
                .collect();
            out.push_str(" (");
            out.push_str(&credit.join(", "));
            out.push(')');
        }

        out.push('\n');
    }

    out.push('\n');
    out
}

/// The `### Contributors` block:  every alias credited anywhere in the
/// section, each linked and named by its primaries.
fn contributors_markdown(
    order: &[&str],
    section: &crate::Section,
    contributors: &Registry,
    references: &mut References,
) -> String {
    let mut credited = indexmap::IndexSet::new();

    for bucket in order {
        for entry in &section.changes[*bucket] {
            for alias in &entry.aliases {
                credited.insert(alias.clone());
            }
        }
    }

    if credited.is_empty() {
        return String::new();
    }

    let mut out = String::from("### Contributors\n\n");

    for alias in &credited {
        out.push_str("- ");
        out.push_str(&credit_token(alias, contributors, references));

        if !alias.contains('@')
            && let Some(name) = contributors
                .get(alias)
                .and_then(crate::Contributor::primary_name)
        {
            out.push_str(" — ");
            out.push_str(name);
        }

        out.push('\n');
    }

    out.push('\n');
    out
}

/// One released section:  its heading, lead, buckets and contributors.
fn section_markdown(
    configuration: &crate::Configuration,
    section: &crate::Section,
    contributors: &Registry,
    references: &mut References,
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

    let order = ordered_buckets(configuration, section);

    for bucket in &order {
        out.push_str(&bucket_markdown(
            bucket,
            &section.changes[*bucket],
            contributors,
            references,
        ));
    }

    out.push_str(&contributors_markdown(
        &order,
        section,
        contributors,
        references,
    ));
    out
}

/******************************************************************************/
