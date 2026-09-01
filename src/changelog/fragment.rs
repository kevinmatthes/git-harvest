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

//! One harvested fragment:  the changes of a single branch, not yet assembled.

/// The changes harvested from one branch, written to `changelog.d/`.
///
/// A fragment is the pending state of one line of work.  Pass two merges
/// every fragment in `changelog.d/` into a [`crate::Section`] and deletes
/// them.  `changes` maps a bucket name to its entries.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
#[non_exhaustive]
pub struct Fragment {
    /// The contributors this branch registers, keyed by alias.
    #[serde(default)]
    pub contributors: std::collections::BTreeMap<String, crate::Contributor>,

    /// Link labels used by this fragment, mapped to their targets.
    pub references: std::collections::BTreeMap<String, String>,

    /// The harvested entries, keyed by bucket.
    pub changes: std::collections::BTreeMap<String, Vec<crate::Entry>>,
}

impl Fragment {
    /// Whether the fragment holds no entries at all.
    pub fn is_empty(&self) -> bool {
        self.changes.values().all(std::vec::Vec::is_empty)
    }

    /// File `entry` under `bucket`, preserving harvest order.
    pub fn record(&mut self, bucket: &str, entry: crate::Entry) {
        self.changes
            .entry(bucket.to_owned())
            .or_default()
            .push(entry);
    }

    /// Serialise the fragment to pretty RON, indented two spaces.
    ///
    /// # Errors
    ///
    /// Returns [`sysexits::ExitCode::Software`] if the fragment cannot be
    /// represented as RON — which should not happen for a value built by this
    /// crate — after printing the reason to standard error.
    pub fn to_ron(&self) -> sysexits::Result<String> {
        let pretty = ron::ser::PrettyConfig::new().indentor("  ".to_owned());

        match ron::ser::to_string_pretty(self, pretty) {
            Ok(body) => Ok(format!("{body}\n")),
            Err(reason) => {
                eprintln!(
                    "git-harvest:  cannot serialise the fragment:  {reason}"
                );
                Err(sysexits::ExitCode::Software)
            }
        }
    }
}

/******************************************************************************/
