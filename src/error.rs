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

//! Everything `git-harvest` can fail with.

/// A `git-harvest` failure, ready to print and to exit on.
#[derive(Debug)]
pub enum Error {
    /// The target path exists already and `--force` was not given.
    TargetExists(std::path::PathBuf),

    /// The CHANGELOG could not be serialised to RON.
    Serialise(ron::Error),

    /// The CHANGELOG file could not be written.
    Write(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetExists(path) => write!(
                f,
                "{} exists already; pass --force to overwrite it",
                path.display()
            ),
            Self::Serialise(reason) => {
                write!(f, "cannot serialise the CHANGELOG:  {reason}")
            }
            Self::Write(reason) => {
                write!(f, "cannot write the CHANGELOG:  {reason}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TargetExists(_) => None,
            Self::Serialise(reason) => Some(reason),
            Self::Write(reason) => Some(reason),
        }
    }
}

/******************************************************************************/
