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

//! One released, or pending, version of the CHANGELOG.

/// One section of the CHANGELOG:  the changes of a single version.
///
/// A section with no `released` date is the pending, not-yet-published one.
/// `changes` maps a bucket name to its entries, in the order they render.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Section {
    /// The version this section documents, as written in the manifest.
    pub version: String,

    /// The publication date in `YYYY-MM-DD` form, or `None` while pending.
    pub released: Option<String>,

    /// An optional prose lead for this version.
    pub introduction: Option<String>,

    /// Link labels used by this section, mapped to their targets.
    pub references: std::collections::BTreeMap<String, String>,

    /// The entries of this section, keyed by bucket.
    pub changes: std::collections::BTreeMap<String, Vec<String>>,
}

/******************************************************************************/
