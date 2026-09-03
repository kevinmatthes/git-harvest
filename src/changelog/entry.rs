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

//! One change entry:  its prose, its origin commit, and its credited authors.

/// One line of a CHANGELOG:  the change, the commit it was harvested from,
/// and the contributors credited for it.
///
/// `commit` is the abbreviated hash the harvest read the entry off, or `None`
/// for an entry written by hand into a fragment.  `aliases` names the
/// contributors credited for this one change, primary first; assembly unions
/// it across entries that describe the very same change.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Entry {
    /// The prose describing the change.
    pub text: String,

    /// The abbreviated hash of the commit this entry was harvested from.
    pub commit: Option<String>,

    /// The aliases credited for this change, primary first.
    #[serde(default, skip_serializing_if = "indexmap::IndexSet::is_empty")]
    pub aliases: indexmap::IndexSet<String>,
}

impl Entry {
    /// An entry written by hand, with no commit and no credit behind it.
    #[must_use]
    pub fn authored(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            commit: None,
            aliases: indexmap::IndexSet::new(),
        }
    }

    /// The abbreviated commit hash this entry was harvested from, if any.
    #[must_use]
    pub fn commit(&self) -> Option<&str> {
        self.commit.as_deref()
    }

    /// Credit `alias` for this change, unless it is credited already.
    pub fn credit(&mut self, alias: &str) {
        self.aliases.insert(alias.to_owned());
    }

    /// An entry harvested from a commit, carrying its abbreviated hash.
    #[must_use]
    pub fn harvested(text: &str, commit: &str) -> Self {
        Self {
            text: text.to_owned(),
            commit: Some(commit.to_owned()),
            aliases: indexmap::IndexSet::new(),
        }
    }

    /// The prose describing the change.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/******************************************************************************/
