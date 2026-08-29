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

//! One change entry:  its prose, and where it came from.

/// One line of a CHANGELOG:  the change described, and the commit it was
/// harvested from.
///
/// Field `0` is the human-readable text.  Field `1` is the abbreviated hash
/// of the commit the harvest read it off, or `None` for an entry written by
/// hand into a fragment.  The tuple shape keeps a hand-authored entry terse
/// in RON:  `("a new option", None)`.
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Entry(pub String, pub Option<String>);

impl Entry {
    /// The abbreviated commit hash this entry was harvested from, if any.
    #[must_use]
    pub fn commit(&self) -> Option<&str> {
        self.1.as_deref()
    }

    /// An entry harvested from a commit, carrying its abbreviated hash.
    #[must_use]
    pub fn harvested(text: &str, commit: &str) -> Self {
        Self(text.to_owned(), Some(commit.to_owned()))
    }

    /// The prose describing the change.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

/******************************************************************************/
