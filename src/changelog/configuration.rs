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

//! The harvest configuration, which lives inside the CHANGELOG itself.

/// How a commit subject encodes the bucket a change belongs to.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Grammar {
    /// `Bucket <delimiter> entry`, for example `Added ::= a new option`.
    #[default]
    Delimited,

    /// `[Bucket] entry`, for example `[Added] a new option`.
    Bracketed,
}

/// The shape the assembled CHANGELOG is rendered in for readers.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Renderer {
    /// A *Keep a Changelog* Markdown document beside the RON source.
    #[default]
    Markdown,
}

/// The harvest configuration.
///
/// `git-harvest` has no separate configuration file:  these settings live in
/// the CHANGELOG document, and [`Configuration::default`] is what
/// `git-harvest init` writes into a fresh one.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Configuration {
    /// The token separating a bucket from its entry under
    /// [`Grammar::Delimited`].
    pub delimiter: String,

    /// How a commit subject encodes its bucket.
    pub grammar: Grammar,

    /// The buckets a change may be filed under, in rendering order.
    pub buckets: Vec<String>,

    /// The bucket for a change whose commit names none, where one is wanted.
    pub fallback_bucket: Option<String>,

    /// The shape the assembled CHANGELOG is rendered in.
    pub renderer: Renderer,
}

impl Configuration {
    /// Parse a commit subject into a `(bucket, entry text)` pair.
    ///
    /// Returns `None` when the subject names no known bucket and no
    /// `fallback_bucket` is set, or when the entry text is empty.  Under
    /// [`Grammar::Delimited`] the bucket is what precedes `delimiter`; under
    /// [`Grammar::Bracketed`] it is what a leading `[..]` encloses.
    pub fn parse(&self, subject: &str) -> Option<(String, String)> {
        let named = match self.grammar {
            Grammar::Delimited => subject.split_once(self.delimiter.as_str()),
            Grammar::Bracketed => subject
                .trim_start()
                .strip_prefix('[')
                .and_then(|rest| rest.split_once(']')),
        };

        let (named, text) = named.map_or_else(
            || (None, subject.trim()),
            |(a, b)| (Some(a.trim()), b.trim()),
        );

        if text.is_empty() {
            return None;
        }

        named
            .filter(|bucket| self.buckets.iter().any(|known| known == bucket))
            .map(str::to_owned)
            .or_else(|| self.fallback_bucket.clone())
            .map(|bucket| (bucket, text.to_owned()))
    }
}

impl Default for Configuration {
    /// The defaults `git-harvest init` writes:  the `::=` delimiter, the
    /// delimited grammar, the six *Keep a Changelog* buckets, no fallback,
    /// and Markdown rendering.
    fn default() -> Self {
        Self {
            delimiter: "::=".to_owned(),
            grammar: Grammar::Delimited,
            buckets: [
                "Added",
                "Changed",
                "Deprecated",
                "Fixed",
                "Removed",
                "Security",
            ]
            .iter()
            .map(|bucket| (*bucket).to_owned())
            .collect(),
            fallback_bucket: None,
            renderer: Renderer::Markdown,
        }
    }
}

/******************************************************************************/
