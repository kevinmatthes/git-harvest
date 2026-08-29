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
}

/******************************************************************************/
