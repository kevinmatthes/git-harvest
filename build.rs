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

/// The copyright line a musl `COPYRIGHT` file states for itself.
///
/// Read from the file rather than pinned separately, so the year range can
/// never drift from whichever version's text is actually bundled.
fn musl_copyright_line(text: &str) -> String {
    text.lines()
        .find_map(|line| line.strip_prefix("Copyright © "))
        .expect("licences/musl/COPYRIGHT must state its own copyright line")
        .trim()
        .to_owned()
}

/// Every distributed binary artefact statically links musl — a system C
/// library `cargo metadata` never sees, reproduced here via
/// [`list_my_licence::build::Builder::extra`] instead.
///
/// `version` below is a manual pin, not Renovate-tracked:  it must match
/// whichever musl the `*-unknown-linux-musl` targets actually statically
/// link, which — since Rust's self-contained linking bundles its own
/// `musl-cross-make` build in the toolchain sysroot — is unrelated to any
/// system `musl-tools` package.  `release.yml`'s `binaries` job checks
/// this live against the sysroot's own `libc.a`, so a Rust-toolchain
/// update that moves the bundled musl must update this pin and
/// `licences/musl/COPYRIGHT` by hand, together, before it can pass.
fn extra_packages() -> Vec<list_my_licence::build::ResolvedPackage> {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let musl_dir = manifest_dir.join("licences/musl");
    let musl_copyright = std::fs::read_to_string(musl_dir.join("COPYRIGHT"))
        .expect("licences/musl/COPYRIGHT must exist");

    vec![list_my_licence::build::ResolvedPackage {
        name: "musl".into(),
        version: "1.2.5".into(),
        manifest_dir: musl_dir,
        licence: Some("MIT".into()),
        licence_file: None,
        authors: vec![musl_copyright_line(&musl_copyright)],
        repository: Some("https://git.musl-libc.org/cgit/musl".into()),
    }]
}

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

/// Whether this build's own target statically links musl.
fn targets_musl() -> bool {
    std::env::var("TARGET").is_ok_and(|target| target.ends_with("-musl"))
}

/// Re-runs the embedding pass alone, without musl, for every target but
/// the ones that actually link it.
///
/// `THIRDPARTY.md`/`debian/copyright` describe every platform this crate
/// could be built for, musl included; the embedded `git harvest licences`
/// output describes only this one binary, so a non-musl build must not
/// claim it.
fn strip_musl_from_embedding(checking: bool) {
    if targets_musl() {
        return;
    }

    if let Err(error) = list_my_licence::build::Builder::new()
        .checking(checking)
        .run()
    {
        panic!("{error}");
    }
}

fn main() {
    let checking = std::env::var_os("CI").is_some();

    let outcome = match list_my_licence::build::Builder::new()
        .publish("THIRDPARTY.md")
        .checking(checking)
        .extra(extra_packages())
        .run()
    {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    };

    if let Err(error) = refresh_or_check_copyright(&outcome, checking) {
        panic!("{error}");
    }

    strip_musl_from_embedding(checking);
}

/******************************************************************************/
