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

//! An `Entry` carries its credited aliases through RON, primary first.

use git_harvest::Entry;

#[test]
fn a_repeated_credit_is_not_recorded_twice() {
    let mut entry = Entry::authored("a change");
    entry.credit("octocat");
    entry.credit("octocat");

    assert_eq!(entry.aliases.len(), 1);
}

#[test]
fn the_credits_keep_their_order_through_ron() {
    let mut entry = Entry::harvested("a change", "abc1234");
    entry.credit("kevinmatthes");
    entry.credit("octocat");

    let ron = ron::ser::to_string(&entry).unwrap();
    let parsed: Entry = ron::from_str(&ron).unwrap();

    assert_eq!(parsed, entry);
    assert_eq!(
        parsed
            .aliases
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["kevinmatthes", "octocat"],
    );
}

#[test]
fn an_uncredited_entry_serialises_without_the_field() {
    let ron = ron::ser::to_string(&Entry::authored("a change")).unwrap();

    assert!(!ron.contains("aliases"));
}

#[test]
fn a_credited_entry_round_trips_from_the_terse_form() {
    let parsed: Entry =
        ron::from_str(r#"(text:"a change",commit:None)"#).unwrap();

    assert_eq!(parsed, Entry::authored("a change"));
    assert!(parsed.aliases.is_empty());
}

/******************************************************************************/
