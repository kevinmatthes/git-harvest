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

//! The document formats a CHANGELOG and its fragments can be written in.

/// A document format, told apart by the extension of its file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Format {
    /// Rusty Object Notation, the native format.
    Ron,

    /// YAML.
    Yaml,
}

impl Format {
    /// The extension a file of this format is written with.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Ron => "ron",
            Self::Yaml => "yaml",
        }
    }

    /// The format the extension of `path` names, if it is a supported one.
    #[must_use]
    pub fn of(path: &std::path::Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "ron" => Some(Self::Ron),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    /// Parse `source` as a document of this format.
    ///
    /// # Errors
    ///
    /// Returns the parser's reason if `source` is not a valid document.
    pub fn parse<T: serde::de::DeserializeOwned>(
        self,
        source: &str,
    ) -> Result<T, String> {
        match self {
            Self::Ron => {
                ron::from_str(source).map_err(|reason| reason.to_string())
            }
            Self::Yaml => serde_saphyr::from_str(source)
                .map_err(|reason| reason.to_string()),
        }
    }

    /// Serialise `value` as a document of this format, ending in a newline.
    ///
    /// # Errors
    ///
    /// Returns the serialiser's reason if `value` cannot be represented.
    pub fn render<T: serde::Serialize>(
        self,
        value: &T,
    ) -> Result<String, String> {
        let pretty = ron::ser::PrettyConfig::new().indentor("  ".to_owned());
        let body = match self {
            Self::Ron => ron::ser::to_string_pretty(value, pretty)
                .map_err(|reason| reason.to_string()),
            Self::Yaml => serde_saphyr::to_string(value)
                .map_err(|reason| reason.to_string()),
        }?;

        Ok(format!("{}\n", body.trim_end()))
    }

    /// Like [`Self::render`], printing the failure and mapping it to a code.
    pub(crate) fn serialised<T: serde::Serialize>(
        self,
        value: &T,
        subject: &str,
    ) -> sysexits::Result<String> {
        self.render(value).map_err(|reason| {
            eprintln!(
                "git-harvest:  cannot serialise the {subject}:  {reason}"
            );
            sysexits::ExitCode::Software
        })
    }
}

/******************************************************************************/
