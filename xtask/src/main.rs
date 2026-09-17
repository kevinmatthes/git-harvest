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

//! Generates or checks the workspace's own committed `man/` pages.
//!
//! Run explicitly (`cargo run -p xtask`) rather than from `build.rs`, since
//! this crate depends on `git-harvest` itself and a build script cannot
//! depend on the crate it builds.  `CI` set selects checking over
//! rewriting, matching the committed pages against a fresh render;
//! `clap_mangen::generate_to` renders one page per subcommand, recursively,
//! so `man git-harvest-scan` and its siblings resolve rather than dangling.

use std::collections::BTreeMap;

use clap::CommandFactory as _;

/// The workspace root, one level up from this crate's own manifest.
fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every regular file directly inside `directory`, name to contents.
fn snapshot(
    directory: &std::path::Path,
) -> std::io::Result<BTreeMap<String, Vec<u8>>> {
    let mut found = BTreeMap::new();

    let Ok(entries) = std::fs::read_dir(directory) else {
        return Ok(found);
    };

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        found.insert(name, std::fs::read(entry.path())?);
    }

    Ok(found)
}

fn render(directory: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(directory)?;
    clap_mangen::generate_to(git_harvest::Cli::command(), directory)
}

fn main() {
    let man = root().join("man");
    let checking = std::env::var_os("CI").is_some();
    let target = if checking {
        std::env::temp_dir()
            .join(format!("git-harvest-man-{}", std::process::id()))
    } else {
        man.clone()
    };

    if let Err(error) = render(&target) {
        eprintln!("{error}");
        std::process::exit(1);
    }

    if !checking {
        return;
    }

    let fresh = snapshot(&target);
    let _ = std::fs::remove_dir_all(&target);

    match (fresh, snapshot(&man)) {
        (Ok(fresh), Ok(committed)) if fresh == committed => {}
        (Ok(_), Ok(_)) => {
            eprintln!("man/ is missing or out of date");
            std::process::exit(1);
        }
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

/******************************************************************************/
