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

//! `git-harvest licences` reproduces the dependency licence notices.

use std::process::Command;

/// Run `git-harvest licences` with `extra` arguments, returning success and
/// the captured standard output.
fn licences(extra: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_git-harvest"))
        .arg("licences")
        .args(extra)
        .output()
        .expect("the binary must build");

    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("licences prints UTF-8"),
    )
}

#[test]
fn it_reproduces_dependencies_and_a_verbatim_body() {
    let (ok, report) = licences(&[]);

    assert!(ok, "git-harvest licences must succeed");
    assert!(report.contains("clap"), "a direct dependency is named");
    assert!(report.contains("gix"), "another direct dependency is named");
    assert!(
        report.contains("Permission is hereby granted"),
        "an MIT body is reproduced word for word"
    );
}

#[test]
fn a_crate_name_narrows_the_report() {
    let (ok_all, all) = licences(&[]);
    let (ok_one, one) = licences(&["clap"]);

    assert!(ok_all && ok_one, "both invocations succeed");
    assert!(one.contains("clap"), "the named crate stays");
    assert!(!one.contains("gix"), "an unrelated crate is dropped");
    assert!(one.len() < all.len(), "the narrowed report is shorter");
}

/******************************************************************************/
