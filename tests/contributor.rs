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

//! A `Contributor` keeps its identities in insertion order through RON.

use git_harvest::Contributor;

#[test]
fn a_repeated_identity_is_not_recorded_twice() {
    let mut contributor = Contributor::new("kevinmatthes");
    contributor.add_email("92332892+kevinmatthes@users.noreply.github.com");
    contributor.add_email("92332892+kevinmatthes@users.noreply.github.com");

    assert_eq!(contributor.emails.len(), 1);
}

#[test]
fn the_identities_keep_their_insertion_order_through_ron() {
    let mut contributor = Contributor::new("claude");
    contributor.add_name("Claude");
    contributor.add_name("Claude Sonnet 5");
    contributor.add_email("noreply@anthropic.com");
    contributor.add_url("https://claude.com/claude-code");
    contributor.add_url("https://www.anthropic.com");

    let ron = ron::ser::to_string(&contributor).unwrap();
    let parsed: Contributor = ron::from_str(&ron).unwrap();

    assert_eq!(parsed, contributor);
    assert_eq!(parsed.primary_name(), Some("Claude"));
    assert_eq!(parsed.primary_email(), Some("noreply@anthropic.com"));
    assert_eq!(parsed.primary_url(), Some("https://claude.com/claude-code"));
    assert_eq!(
        parsed.urls.iter().map(String::as_str).collect::<Vec<_>>(),
        [
            "https://claude.com/claude-code",
            "https://www.anthropic.com"
        ],
    );
}

#[test]
fn a_bare_alias_carries_no_identities() {
    let contributor = Contributor::new("renovate");

    assert!(contributor.primary_name().is_none());
    assert!(contributor.primary_email().is_none());
    assert!(contributor.primary_url().is_none());
}

/******************************************************************************/
