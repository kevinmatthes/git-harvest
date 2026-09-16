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

//! Harvest the dependency licences at build time.
//!
//! `list-my-licence` resolves the graph, embeds the notices for
//! `git-harvest licences` to print, and refreshes the committed
//! `THIRDPARTY.md`.  Under continuous integration (`CI` set) it checks that
//! file against the graph instead of rewriting it, so licence drift cannot
//! be merged unnoticed.  `debian/copyright` is refreshed or checked
//! alongside it, hand-rolled because
//! [`list_my_licence::build::Emitter::check`] is markdown-specific — there
//! is no DEP-5 equivalent to call instead.

fn refresh_or_check_copyright(
    outcome: &list_my_licence::build::Outcome,
    checking: bool,
) -> std::io::Result<()> {
    let packages: Vec<list_my_licence::build::Reproduced<'_>> = outcome
        .packages
        .iter()
        .map(|(package, verdict)| (package, verdict))
        .collect();
    let expected = list_my_licence::build::Emitter::dep5(&packages);
    let path = std::path::Path::new("debian/copyright");

    if !checking {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        return std::fs::write(path, expected);
    }

    match std::fs::read_to_string(path) {
        Ok(found) if found == expected => Ok(()),
        _ => Err(std::io::Error::other(format!(
            "{} is missing or out of date",
            path.display()
        ))),
    }
}

fn main() {
    let checking = std::env::var_os("CI").is_some();

    let outcome = match list_my_licence::build::Builder::new()
        .publish("THIRDPARTY.md")
        .checking(checking)
        .run()
    {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    };

    if let Err(error) = refresh_or_check_copyright(&outcome, checking) {
        panic!("{error}");
    }
}

/******************************************************************************/
