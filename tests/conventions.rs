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

//! The conventions no cargo subcommand can check.
//!
//! Several of them:  eighty characters a line, the licence header on every
//! hand-written file, prose written in British English with English Spacing,
//! exactly one space after a semicolon, and a category opening every commit
//! subject.  The prose rules were enforced by two Python scripts until
//! 2026-08-23; a Rust repository should hold Rust, so they are a test harness
//! now.
//!
//! Four of these tests check the checker rather than the repository.  A
//! checker which reports nothing is worthless until it has been shown to
//! report something, and the Python original earned that lesson twice; a
//! fourth confirms the checker leaves the changelog RON files alone.
//!
//! Prose living outside the repository — the session reports — is held to the
//! same conventions by naming it in `CONVENTIONS_EXTRA_PROSE`, colon
//! separated.  Only the language check reads it; a report's tables are wider
//! than eighty characters by nature, but its semicolons obey the one-space
//! rule like any source.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The longest line this project allows, in characters rather than bytes.
///
/// `awk`'s `length` counts bytes, which over-reports every line carrying an
/// em dash; that once cost a change to a line which did not need one.
const WIDTH: usize = 80;

/// The first line of the licence header, by which the header is recognised.
const NOTICE: &str = "GNU General Public License 3.0";

/// Files carrying no licence header:  generated, or the notice itself.
const UNHEADED: [&str; 3] = ["Cargo.lock", "LICENCE", "README.md"];

/// Extensions expected to carry a header; a fixture or a datum is not.
const HEADED: [&str; 8] = [
    ".cff",
    ".gitattributes",
    ".gitignore",
    ".py",
    ".rs",
    ".toml",
    ".yml",
    "CODEOWNERS",
];

/// Formats whose prose lives in hash comments.
///
/// Kept apart from [`HEADED`] deliberately:  whether a file must open with the
/// notice and where its language is found are different questions, and
/// conflating them meant a new format could only be checked for prose by also
/// being made to carry a header.
const HASH_COMMENTED: [&str; 7] = [
    ".cff",
    ".gitattributes",
    ".gitignore",
    ".py",
    ".toml",
    ".yml",
    "CODEOWNERS",
];

/// Files holding no prose of this project's own.
///
/// The licence is a verbatim quotation of somebody else's words, and the
/// lock file is generated.
const UNCHECKED: [&str; 2] = ["Cargo.lock", "LICENCE"];

/// Whether the file is a changelog RON document rather than prose.
///
/// `CHANGELOG.ron` and the `changelog.d/` fragments hold entries and RON
/// syntax `git-harvest` writes — one item to a line, `key: value` with a
/// single space — not text a person wrapped and spaced by hand.  The width
/// and language rules step over them the way they step over `Cargo.lock`.
fn changelog_ron(name: &str) -> bool {
    Path::new(name).extension().is_some_and(|end| end == "ron")
}

/// Where the language fixtures live.
///
/// They are deliberately incorrect prose, so they are data rather than
/// language:  checking them would report exactly the findings they exist to
/// provoke.  Keeping them out here is what lets the harness itself be checked
/// in full, which the Python original never was.
/// Files which carry no closing rule.
///
/// `Cargo.lock` is generated and `LICENCE` is somebody else's text, so
/// neither is this project's to shape.  `renovate.json` is JSON, which has no
/// comment syntax to write a rule in — the same standing exception the
/// licence header makes for it.
const UNRULED: [&str; 3] = ["Cargo.lock", "LICENCE", "renovate.json"];

const FIXTURES: &str = "tests/assets/language/";

/// Spellings which are American, as whole words.
const AMERICAN_WORDS: [&str; 2] = ["color", "center"];

/// Spellings which are American wherever a word begins with them.
const AMERICAN_PREFIXES: [&str; 5] =
    ["artifact", "behavior", "fulfill", "license", "licensi"];

/// Endings which are American wherever a word closes with them.
const AMERICAN_SUFFIXES: [&str; 12] = [
    "ization", "izations", "ize", "ized", "izer", "izes", "izing", "yze",
    "yzed", "yzer", "yzes", "yzing",
];

/// Words the endings above catch which are correct British English.
const INNOCENT: [&str; 14] = [
    "capsize", "downsize", "maize", "prize", "prizes", "resize", "resized",
    "resizes", "seize", "seizes", "size", "sized", "sizer", "sizes",
];

/// Terms fixed by something outside this project.
///
/// A quoted interface label, or a name belonging to somebody else.
/// Correcting one would break the reference rather than improve the prose.
const FOREIGN: [&str; 1] = ["authorization"];

/// Abbreviations whose full stop does not end a sentence.
const ABBREVIATIONS: [&str; 8] =
    ["Dr.", "Mr.", "St.", "al.", "cf.", "e.g.", "etc.", "i.e."];

/// Contexts in which `license` is a name rather than a misspelling.
///
/// It is a Cargo manifest key, another author's crate name, a field of
/// `cargo_metadata`, part of a URL, and the conventional file name of the
/// wider ecosystem.  It is therefore exempt only where it is plainly one of
/// those, and flagged everywhere else:  a blanket exemption would hide the
/// word in genuine prose, which it did until 2026-08-22.
const CODE_LICENCE: [&str; 9] = [
    "/licenses/",
    "LICENSE-",
    "LICENSE.",
    "LICENSE_",
    "LICENSES",
    "UNLICENSE",
    "dep:license",
    "license-file",
    "license_id",
];

/// Fragments marking a line as Rust being emitted rather than English.
///
/// A code generator's output is governed by Rust's rules, not by these.
const CODE_LITERAL: [&str; 6] =
    ["&[", "::", "fn ", "include_str!", "pub ", "{{"];

/// The categories a commit subject may open with.
///
/// A squash-merged pull request keeps its title as the subject, so the same
/// set governs both:  `[<category>] <something>`, with an optional trailing
/// ` (#123)` that the forge appends.
const COMMIT_CATEGORIES: &[&str] = &[
    "Renovate",
    "GitHub Actions",
    "Bugfix",
    "Documentation",
    "Enhancement",
];

/// Whether `word` is one of this project's accepted exceptions.
fn accepted(word: &str) -> bool {
    INNOCENT.contains(&word) || FOREIGN.contains(&word)
}

/// Every finding the American spelling rules produce in one fragment.
fn american(text: &str) -> Vec<String> {
    let mut findings = Vec::new();

    for (start, word) in words(text) {
        let lower = word.to_lowercase();

        if accepted(&lower) {
            continue;
        }

        let hit = AMERICAN_WORDS.contains(&lower.as_str())
            || AMERICAN_SUFFIXES.iter().any(|end| lower.ends_with(end))
            || AMERICAN_PREFIXES.iter().any(|head| lower.starts_with(head));

        if !hit {
            continue;
        }

        if proper_name(text, start, word.len()) {
            continue;
        }

        if lower.starts_with("licens") && code_licence(text, start) {
            continue;
        }

        findings.push(format!("[SPELLING] {word:?}"));
    }

    findings
}

/// Whether `license` at `start` sits in one of the contexts of
/// [`CODE_LICENCE`].
fn code_licence(text: &str, start: usize) -> bool {
    let window = window(text, start, 12, 16);

    if CODE_LICENCE.iter().any(|form| window.contains(form)) {
        return true;
    }

    let before = text[..start].chars().next_back();
    let after = text[start..].chars().nth("license".len());

    matches!(before, Some('.' | ':' | '"' | '`' | '/'))
        || matches!(after, Some(' ' | '=') if window.contains('='))
}

/// The rule which closes this file, where its format has one.
///
/// Three forms, one for each comment syntax in use here, and every one of
/// them exactly [`WIDTH`] characters:  the rule is the width, drawn.  It
/// marks where a file ends, so nothing may follow it — appending below it is
/// how the lint tables first went into the manifest.
fn closing_rule(path: &Path) -> Option<String> {
    let name = show(path);
    let base = name.rsplit('/').next().unwrap_or(&name).to_owned();

    if UNRULED.contains(&base.as_str()) || name.starts_with(FIXTURES) {
        return None;
    }

    if base == "README.md" {
        return Some(format!("<!--{} -->", "-".repeat(WIDTH - 8)));
    }

    if Path::new(&name).extension().is_some_and(|e| e == "rs") {
        return Some(format!("/{}/", "*".repeat(WIDTH - 2)));
    }

    HASH_COMMENTED
        .iter()
        .any(|end| name.ends_with(end))
        .then(|| "#".repeat(WIDTH))
}
/// Every finding in one file, as `line:  message`.
///
/// `semicolons` asks for the one-space-after-a-semicolon rule as well.  It is
/// off only for the language fixtures, which are deliberately incorrect
/// language rather than this project's prose; the session reports named in
/// `CONVENTIONS_EXTRA_PROSE` are held to it like every source.
fn findings(path: &Path, text: &str, semicolons: bool) -> Vec<String> {
    let mut found = Vec::new();
    let mut fenced = false;
    let mut heading = false;

    for (number, line) in text.lines().enumerate() {
        for fragment in prose(path, line, &mut fenced, &mut heading) {
            let stripped = strip(&fragment);
            let trimmed = stripped.trim();

            if trimmed.is_empty() {
                continue;
            }

            if CODE_LITERAL.iter().any(|form| trimmed.contains(form)) {
                continue;
            }

            for finding in american(trimmed) {
                found.push(format!("{}:{}  {finding}", show(path), number + 1));
            }

            for finding in spacing(trimmed) {
                found.push(format!("{}:{}  {finding}", show(path), number + 1));
            }

            if semicolons {
                for finding in semicolon(despan(&fragment).trim()) {
                    found.push(format!(
                        "{}:{}  {finding}",
                        show(path),
                        number + 1
                    ));
                }
            }
        }
    }

    found
}

/// The index of the licence header's closing line, if the file opens with one.
///
/// A script's shebang has to be the very first line of it, so the notice
/// follows rather than opens.
fn header_end(lines: &[&str]) -> Option<usize> {
    let opening = lines.iter().position(|line| {
        line.contains(NOTICE)
            && (line.starts_with('#') || line.starts_with('/'))
    })?;

    if opening > 2 {
        return None;
    }

    lines
        .iter()
        .enumerate()
        .skip(opening + 1)
        .find_map(|(index, line)| {
            let rule = line.len() >= 40
                && (line.chars().all(|character| character == '#')
                    || (line.starts_with('\\') && line.ends_with("*/")));

            rule.then_some(index)
        })
}

/// Whether a file of this name is expected to carry the licence header.
fn is_headed(path: &Path) -> bool {
    let name = show(path);
    let base = name.rsplit('/').next().unwrap_or(&name).to_owned();

    !UNHEADED.contains(&base.as_str())
        && HEADED.iter().any(|end| name.ends_with(end))
}

/// The prose fragments of one line, or nothing where it carries none.
///
/// Code is not prose:  a type annotation is allowed one space after its
/// colon, and a method chain one after its full stop.  Only comments,
/// documentation and substantial string literals are considered.
fn prose(
    path: &Path,
    line: &str,
    fenced: &mut bool,
    heading: &mut bool,
) -> Vec<String> {
    let name = show(path);
    let trimmed = line.trim_start();

    if line.contains(NOTICE)
        && (trimmed.starts_with('/') || trimmed.starts_with('#'))
    {
        *heading = true;
        return Vec::new();
    }

    if *heading {
        if rule(line) {
            *heading = false;
        }

        return Vec::new();
    }

    if rule(line) || table(line) {
        return Vec::new();
    }

    if Path::new(&name).extension().is_some_and(|e| e == "json") {
        return Vec::new();
    }

    if Path::new(&name).extension().is_some_and(|e| e == "rs") {
        return rust(line, fenced);
    }

    if Path::new(&name).extension().is_some_and(|e| e == "md") {
        if trimmed.starts_with("```") {
            *fenced = !*fenced;
            return Vec::new();
        }

        if *fenced || line.starts_with("    ") {
            return Vec::new();
        }

        return vec![line.to_owned()];
    }

    if HASH_COMMENTED.iter().any(|end| name.ends_with(end)) {
        return hash_comment(line).map_or_else(Vec::new, |body| vec![body]);
    }

    vec![line.to_owned()]
}

/// The body of a hash comment, where the line is one.
fn hash_comment(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let body = trimmed.strip_prefix('#')?.trim_start_matches('#');

    Some(body.strip_prefix(' ').unwrap_or(body).to_owned())
}

/// Whether a licence's real title stands near the hit.
///
/// `License` inside a proper name — the title of "Apache License 2.0", or of
/// the "GNU General Public License" — is that licence's own name, and
/// `LicenseRef` is an SPDX keyword.  Neither is this project's to correct.
fn proper_name(text: &str, start: usize, length: usize) -> bool {
    let window = window(text, start, 24, length + 10);

    if window.contains("LicenseRef")
        || window.contains("LicenceRef")
        || window.contains("DocumentRef")
    {
        return true;
    }

    window.match_indices("License").any(|(at, _)| {
        let before = window[..at].trim_end();

        if before.len() == window[..at].len() {
            return false;
        }

        let mut word: Vec<char> = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
            .collect();

        word.reverse();

        word.first().is_some_and(|first| first.is_uppercase())
    })
}

/// Whether the line is a horizontal rule, carrying no language.
fn rule(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.len() < 2 {
        return false;
    }

    trimmed.chars().all(|character| character == '#') && trimmed.len() >= 20
        || trimmed.starts_with("/*") && trimmed.ends_with("*/")
        || trimmed.starts_with('\\') && trimmed.ends_with("*/")
        || trimmed.starts_with("/*") && trimmed.ends_with('\\')
}

/// The prose of one line of Rust.
fn rust(line: &str, fenced: &mut bool) -> Vec<String> {
    let trimmed = line.trim_start();

    for marker in ["///", "//!", "//"] {
        if let Some(body) = trimmed.strip_prefix(marker) {
            let body = body.strip_prefix(' ').unwrap_or(body);

            if body.trim_start().starts_with("```") {
                *fenced = !*fenced;
                return Vec::new();
            }

            return if *fenced {
                Vec::new()
            } else {
                vec![body.to_owned()]
            };
        }
    }

    string_literals(line)
}

/// The path as this project writes it, relative to the repository root.
fn show(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Every English Spacing finding in one fragment.
///
/// One space after a sentence's close where the convention asks for two.
fn spacing(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut findings = Vec::new();

    for (index, character) in text.char_indices() {
        if !matches!(character, '.' | '!' | '?' | ':') {
            continue;
        }

        let previous = text[..index].chars().next_back();

        if previous.is_some_and(|c| c == '.' || c.is_ascii_digit()) {
            continue;
        }

        if bytes.get(index + 1) != Some(&b' ')
            || bytes.get(index + 2) == Some(&b' ')
            || index + 2 > bytes.len()
        {
            continue;
        }

        let opening = index.saturating_sub(5);

        if ABBREVIATIONS
            .iter()
            .any(|short| text[opening..=index].ends_with(short))
        {
            continue;
        }

        findings.push(format!("[SPACING] {character:?}"));
    }

    findings
}

/// Every semicolon-spacing finding in one fragment.
///
/// A semicolon joins two clauses rather than closing a sentence, so English
/// Spacing does not double the space after it:  exactly one follows, and two
/// or more are the finding.
fn semicolon(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut findings = Vec::new();

    for (index, character) in text.char_indices() {
        if character != ';' {
            continue;
        }

        if bytes.get(index + 1) == Some(&b' ')
            && bytes.get(index + 2) == Some(&b' ')
        {
            findings.push("[SEMICOLON] two spaces after ';'".to_owned());
        }
    }

    findings
}

/// Substantial string literals of one line of Rust.
///
/// Short ones are identifiers and separators rather than sentences.
fn string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    let mut inside = false;

    for character in line.chars() {
        if escaped {
            escaped = false;
            current.push(character);
            continue;
        }

        match character {
            '\\' if inside => escaped = true,
            '"' if inside => {
                if current.len() > 12 && current.contains(' ') {
                    literals.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }

                inside = false;
            }
            '"' => inside = true,
            _ if inside => current.push(character),
            _ => {}
        }
    }

    literals
}

/// The fragment with its code spans and web addresses removed.
///
/// Text between backticks is code — an identifier, a file name, a command —
/// and is governed by the language of whatever it names.  A web address is
/// somebody else's name for something:  quoted, never corrected, however it
/// happens to be spelled.
fn strip(text: &str) -> String {
    let mut inside = false;
    let mut spanless = String::new();

    for part in text.split('`') {
        if inside {
            spanless.push(' ');
        } else {
            spanless.push_str(part);
        }

        inside = !inside;
    }

    let mut result = String::new();
    let mut rest = spanless.as_str();

    while let Some(at) = rest.find("http") {
        let opening = rest[..at].trim_end_matches('<');
        let closing = rest[at..]
            .find(char::is_whitespace)
            .map_or(rest.len(), |offset| at + offset);

        result.push_str(opening);
        result.push(' ');
        rest = &rest[closing..];
    }

    result.push_str(rest);
    result
}

/// The fragment with each inline code span reduced to one non-space token.
///
/// The semicolon check counts the spaces after a `;`, and [`strip`] would
/// turn `word; `span` next` into `word;  next` — a doubled space that is not
/// in the prose.  Collapsing every span to a single `_` keeps the real
/// spacing intact and leaves a `;` inside a span to the code's own rules.
fn despan(text: &str) -> String {
    text.split('`')
        .enumerate()
        .map(|(part, text)| if part % 2 == 0 { text } else { "_" })
        .collect()
}

/// Whether the line is a table row, which carries no prose of its own.
fn table(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1
}

/// Every file the repository knows about, staged additions included.
///
/// `-co --exclude-standard` is not decoration.  A file which is added but not
/// yet committed is precisely the one whose header and width nobody has
/// checked, and the Python original carried these flags for that reason.  The
/// port dropped them, and twice let a red commit reach the remote because the
/// harness could not see the file that broke it.
fn tracked() -> Vec<PathBuf> {
    let output = Command::new("git")
        .args(["ls-files", "-co", "--exclude-standard"])
        .current_dir(root())
        .output()
        .expect("git ls-files must run inside the repository");

    assert!(output.status.success(), "git ls-files failed");

    String::from_utf8(output.stdout)
        .expect("git ls-files must return UTF-8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// The repository root, wherever cargo was invoked from.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The text around a hit, for the exemptions to be judged against.
fn window(text: &str, start: usize, before: usize, after: usize) -> &str {
    let opening = (0..=start.saturating_sub(before))
        .rev()
        .find(|index| text.is_char_boundary(*index))
        .unwrap_or(0);
    let closing = (start.saturating_add(after).min(text.len())..=text.len())
        .find(|index| text.is_char_boundary(*index))
        .unwrap_or(text.len());

    &text[opening..closing]
}

/// The words of a fragment, with the byte offset each begins at.
fn words(text: &str) -> Vec<(usize, String)> {
    let mut current = String::new();
    let mut start = 0;
    let mut found = Vec::new();

    for (index, character) in text.char_indices() {
        if character.is_alphanumeric() || character == '_' {
            if current.is_empty() {
                start = index;
            }

            current.push(character);
        } else if !current.is_empty() {
            found.push((start, std::mem::take(&mut current)));
        }
    }

    if !current.is_empty() {
        found.push((start, current));
    }

    found
}

/// One of the language fixtures, read from [`FIXTURES`].
fn fixture(name: &str) -> String {
    let path = root().join(FIXTURES).join(name);

    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("cannot read {}", path.display()))
}

/// Everything the language check reports across the repository.
fn language_findings() -> Vec<String> {
    let mut found = Vec::new();

    for path in tracked() {
        let name = show(&path);
        let base = name.rsplit('/').next().unwrap_or(&name).to_owned();

        if UNCHECKED.contains(&base.as_str())
            || name.starts_with(FIXTURES)
            || changelog_ron(&name)
        {
            continue;
        }

        let full = root().join(&path);

        if let Ok(text) = std::fs::read_to_string(&full) {
            found.extend(findings(&path, &text, true));
        }
    }

    for extra in std::env::var("CONVENTIONS_EXTRA_PROSE")
        .unwrap_or_default()
        .split(':')
        .filter(|entry| !entry.is_empty())
    {
        let path = PathBuf::from(extra);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("cannot read {extra}"));

        found.extend(findings(&path, &text, true));
    }

    found
}

#[test]
fn every_file_closes_with_its_rule() {
    let mut wrong = Vec::new();

    for path in tracked() {
        let Some(rule) = closing_rule(&path) else {
            continue;
        };

        let Ok(text) = std::fs::read_to_string(root().join(&path)) else {
            continue;
        };

        let mut lines = text.lines().rev();

        if lines.next() != Some(rule.as_str()) {
            wrong
                .push(format!("{}  does not close with its rule", show(&path)));
            continue;
        }

        if lines.next().is_some_and(|line| !line.trim().is_empty()) {
            wrong.push(format!(
                "{}  no blank line before the closing rule",
                show(&path)
            ));
        }
    }

    assert!(
        wrong.is_empty(),
        "the closing rule must be a file's last line:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn every_file_holds_its_lines_within_eighty_characters() {
    let mut wide = Vec::new();

    for path in tracked() {
        let name = show(&path);
        let base = name.rsplit('/').next().unwrap_or(&name).to_owned();

        if UNCHECKED.contains(&base.as_str()) || changelog_ron(&name) {
            continue;
        }

        let Ok(text) = std::fs::read_to_string(root().join(&path)) else {
            continue;
        };

        for (number, line) in text.lines().enumerate() {
            let length = line.chars().count();

            if length > WIDTH {
                wide.push(format!(
                    "{name}:{}  {length} characters",
                    number + 1
                ));
            }
        }
    }

    assert!(
        wide.is_empty(),
        "lines wider than {WIDTH}:\n{}",
        wide.join("\n")
    );
}

#[test]
fn every_hand_written_file_carries_the_licence_header() {
    let mut bare = Vec::new();

    for path in tracked() {
        if !is_headed(&path) {
            continue;
        }

        let Ok(text) = std::fs::read_to_string(root().join(&path)) else {
            continue;
        };

        if header_end(&text.lines().collect::<Vec<_>>()).is_none() {
            bare.push(show(&path));
        }
    }

    assert!(bare.is_empty(), "no licence header:\n{}", bare.join("\n"));
}

#[test]
fn exactly_one_blank_line_follows_the_licence_header() {
    let mut wrong = Vec::new();

    for path in tracked() {
        let Ok(text) = std::fs::read_to_string(root().join(&path)) else {
            continue;
        };

        let lines: Vec<_> = text.lines().collect();

        let Some(end) = header_end(&lines) else {
            continue;
        };

        let blanks = lines[end + 1..]
            .iter()
            .take_while(|line| line.trim().is_empty())
            .count();

        if blanks != 1 {
            wrong.push(format!("{}  {blanks} blank lines", show(&path)));
        }
    }

    assert!(
        wrong.is_empty(),
        "exactly one blank line must follow the header:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn the_prose_is_british_english_with_english_spacing() {
    let found = language_findings();

    assert!(found.is_empty(), "language findings:\n{}", found.join("\n"));
}

#[test]
fn the_language_check_reports_a_known_bad_fixture() {
    let bad = fixture("bad.txt");
    let found = findings(Path::new("bad.rs"), &bad, false);
    let kinds: BTreeSet<_> = found
        .iter()
        .map(|finding| finding.split_once("  ").unwrap().1.to_owned())
        .collect();

    assert_eq!(found.len(), 6, "expected six findings, got:\n{found:#?}");
    assert!(kinds.iter().any(|kind| kind.starts_with("[SPELLING]")));
    assert!(kinds.iter().any(|kind| kind.starts_with("[SPACING]")));
}

#[test]
fn the_language_check_accepts_a_known_good_fixture() {
    let good = fixture("good.txt");
    let found = findings(Path::new("good.rs"), &good, false);

    assert!(found.is_empty(), "expected nothing, got:\n{found:#?}");
}

#[test]
fn the_semicolon_check_reports_a_doubled_space_only_when_asked() {
    let fixture = fixture("semicolons.txt");
    let asked = findings(Path::new("semicolons.rs"), &fixture, true);
    let silent = findings(Path::new("semicolons.rs"), &fixture, false);

    assert_eq!(
        asked.len(),
        1,
        "one doubled space after a semicolon, got:\n{asked:#?}"
    );
    assert!(
        asked[0].contains("[SEMICOLON]"),
        "the finding names the semicolon rule, got:\n{asked:#?}"
    );
    assert!(
        silent.is_empty(),
        "the rule stays off where it is not asked for, got:\n{silent:#?}"
    );
}

#[test]
fn the_width_and_language_rules_skip_changelog_ron() {
    assert!(changelog_ron("CHANGELOG.ron"));
    assert!(changelog_ron("changelog.d/2026-01-02T03-04-05Z_branch.ron"));
    assert!(!changelog_ron("src/changelog/document.rs"));
    assert!(!changelog_ron("Cargo.toml"));
}

/// The commit subjects since the most recent tag, or nothing where there is
/// no tag to measure from.
///
/// A shallow clone carries no tags — `actions/checkout` fetches none without
/// `fetch-depth: 0` — and a fresh fork may have none at all.  Neither is a
/// convention being broken, so the check steps aside rather than failing.
fn commit_subjects() -> Option<Vec<String>> {
    let tag = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(root())
        .output()
        .expect("git describe must run inside the repository");

    let tag = String::from_utf8(tag.stdout)
        .expect("git describe must return UTF-8")
        .trim()
        .to_owned();

    if tag.is_empty() {
        return None;
    }

    let log = Command::new("git")
        .args(["log", "--format=%s", "--no-merges", &format!("{tag}..HEAD")])
        .current_dir(root())
        .output()
        .expect("git log must run inside the repository");

    assert!(log.status.success(), "git log failed");

    Some(
        String::from_utf8(log.stdout)
            .expect("git log must return UTF-8")
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

/// Whether a commit subject opens with `[<category>] ` and then a word.
fn subject_names_a_category(subject: &str) -> bool {
    COMMIT_CATEGORIES.iter().any(|category| {
        subject
            .strip_prefix(&format!("[{category}] "))
            .is_some_and(|rest| rest.starts_with(|c: char| !c.is_whitespace()))
    })
}

#[test]
fn every_commit_since_the_last_tag_names_a_category() {
    let Some(subjects) = commit_subjects() else {
        eprintln!("no tag to measure from; skipping the commit-category check");
        return;
    };

    let wrong: Vec<_> = subjects
        .into_iter()
        .filter(|subject| !subject_names_a_category(subject))
        .collect();

    assert!(
        wrong.is_empty(),
        "a commit subject must open with one of {COMMIT_CATEGORIES:?}:\n{}",
        wrong.join("\n")
    );
}

/******************************************************************************/
