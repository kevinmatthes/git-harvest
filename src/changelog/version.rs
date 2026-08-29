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

//! Carrying a [`semver::Version`] as a bare RON triple.
//!
//! A section's version is a `(major, minor, patch)` tuple in the RON, not a
//! string (`git-harvest.md` D19.1).  A pre-release or build field has no
//! room in the triple, and this crate's release family never ships one, so
//! serialising one is an error rather than a silent loss.

/// Read a `(major, minor, patch)` triple into a [`semver::Version`].
///
/// # Errors
///
/// Whatever `D` reports when the value is not a triple of unsigned
/// integers.
pub fn deserialize<'de, D>(deserializer: D) -> Result<semver::Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <(u64, u64, u64) as serde::Deserialize>::deserialize(deserializer)
        .map(|(major, minor, patch)| semver::Version::new(major, minor, patch))
}

/// Write a [`semver::Version`] as a `(major, minor, patch)` triple.
///
/// # Errors
///
/// When `version` carries a pre-release or build field:  the triple cannot
/// represent it.
pub fn serialize<S>(
    version: &semver::Version,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if !version.pre.is_empty() || !version.build.is_empty() {
        return Err(<S::Error as serde::ser::Error>::custom(format!(
            "{version} is not a plain major.minor.patch triple"
        )));
    }

    serde::Serialize::serialize(
        &(version.major, version.minor, version.patch),
        serializer,
    )
}

/******************************************************************************/
