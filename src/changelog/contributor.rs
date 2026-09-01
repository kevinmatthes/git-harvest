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

//! One contributor:  the identities gathered under a single alias.

/// A person or bot credited in the CHANGELOG, gathered under one alias.
///
/// `names`, `emails` and `urls` keep their insertion order:  element `0` of
/// each is the primary — the value shown or linked where only one is wanted.
/// A later harvest appends to them; it never reorders or overwrites.  The
/// `alias` is mandatory and, before curation, is simply the raw e-mail the
/// contributor was first registered under.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Contributor {
    /// The short handle this contributor is credited as.
    pub alias: String,

    /// The author names seen for this contributor, primary first.
    #[serde(default)]
    pub names: indexmap::IndexSet<String>,

    /// The e-mail addresses seen for this contributor, primary first.
    #[serde(default)]
    pub emails: indexmap::IndexSet<String>,

    /// The web addresses of this contributor, primary first.
    #[serde(default)]
    pub urls: indexmap::IndexSet<String>,
}

impl Contributor {
    /// Append `email`, unless it is recorded already.
    pub fn add_email(&mut self, email: &str) {
        self.emails.insert(email.to_owned());
    }

    /// Append `name`, unless it is recorded already.
    pub fn add_name(&mut self, name: &str) {
        self.names.insert(name.to_owned());
    }

    /// Append `url`, unless it is recorded already.
    pub fn add_url(&mut self, url: &str) {
        self.urls.insert(url.to_owned());
    }

    /// A contributor known only by `alias`, carrying no identities yet.
    #[must_use]
    pub fn new(alias: &str) -> Self {
        Self {
            alias: alias.to_owned(),
            names: indexmap::IndexSet::new(),
            emails: indexmap::IndexSet::new(),
            urls: indexmap::IndexSet::new(),
        }
    }

    /// The primary e-mail address, or `None` when none is recorded.
    #[must_use]
    pub fn primary_email(&self) -> Option<&str> {
        self.emails.get_index(0).map(String::as_str)
    }

    /// The primary author name, or `None` when none is recorded.
    #[must_use]
    pub fn primary_name(&self) -> Option<&str> {
        self.names.get_index(0).map(String::as_str)
    }

    /// The primary web address, or `None` when none is recorded.
    #[must_use]
    pub fn primary_url(&self) -> Option<&str> {
        self.urls.get_index(0).map(String::as_str)
    }
}

/******************************************************************************/
