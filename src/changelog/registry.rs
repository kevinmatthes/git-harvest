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

//! Registering and consolidating the contributor registry.
//!
//! A registry is the `contributors` map of a [`crate::Fragment`] or a
//! [`crate::Changelog`], keyed by alias.  A freshly harvested entry is keyed
//! by a raw e-mail until someone curates it (`git-harvest.md` D54).

use crate::Contributor;
use std::collections::BTreeMap;

/// The registry a fragment or a document keeps, keyed by alias.
type Registry = BTreeMap<String, Contributor>;

/// Whether `contributor`'s alias has been curated — it is no longer simply
/// one of the contributor's own e-mail addresses.
pub fn curated(contributor: &Contributor) -> bool {
    !contributor.emails.contains(&contributor.alias)
}

/// Append every identity `source` carries that `target` lacks, leaving
/// `target`'s order — and so its primaries — untouched.
pub fn fold(target: &mut Contributor, source: &Contributor) {
    for name in &source.names {
        target.names.insert(name.clone());
    }
    for email in &source.emails {
        target.emails.insert(email.clone());
    }
    for url in &source.urls {
        target.urls.insert(url.clone());
    }
}

/// Move `value` to the front of `set`, inserting it first if it is absent.
pub fn promote(set: &mut indexmap::IndexSet<String>, value: &str) {
    if !set.contains(value) {
        set.insert(value.to_owned());
    }

    if let Some(index) = set.get_index_of(value) {
        set.move_index(index, 0);
    }
}

/// Record the `(name, email)` identity in `registry`, returning the alias it
/// now lives under.
///
/// A known e-mail folds the name into that entry; an unknown e-mail opens a
/// fresh entry keyed by the e-mail itself (`git-harvest.md` D54).
pub fn register(registry: &mut Registry, name: &str, email: &str) -> String {
    if let Some(contributor) = registry
        .values_mut()
        .find(|contributor| contributor.emails.contains(email))
    {
        if !name.is_empty() {
            contributor.names.insert(name.to_owned());
        }
        return contributor.alias.clone();
    }

    let mut contributor = Contributor::new(email);
    if !name.is_empty() {
        contributor.add_name(name);
    }
    contributor.add_email(email);
    registry.insert(email.to_owned(), contributor);
    email.to_owned()
}

/// Register a contributor under a chosen `alias`.
///
/// # Errors
///
/// [`sysexits::ExitCode::DataErr`] when `alias` is taken or `email` already
/// belongs to another contributor.
pub fn register_curated(
    registry: &mut Registry,
    alias: &str,
    name: &str,
    email: &str,
) -> sysexits::Result<()> {
    if registry.contains_key(alias) {
        eprintln!("git-harvest:  {alias:?} is registered already");
        return Err(sysexits::ExitCode::DataErr);
    }

    if let Some(owner) =
        registry.values().find(|known| known.emails.contains(email))
    {
        eprintln!(
            "git-harvest:  <{email}> belongs to {:?} already; use `git \
             harvest id update` or `git harvest id merge`",
            owner.alias
        );
        return Err(sysexits::ExitCode::DataErr);
    }

    let mut contributor = Contributor::new(alias);
    if !name.is_empty() {
        contributor.add_name(name);
    }
    contributor.add_email(email);
    registry.insert(alias.to_owned(), contributor);
    Ok(())
}

/// Merge `incoming` into `registry`, returning the map from each incoming
/// alias to the alias it ended up under.
///
/// Two contributors that share an e-mail are one person (`git-harvest.md`
/// D54):  they are folded, a curated alias winning over an e-mail-default
/// one.
///
/// # Errors
///
/// [`sysexits::ExitCode::DataErr`] when two *curated* contributors claim the
/// same e-mail; the message names both and points at `git harvest id merge`.
pub fn absorb(
    registry: &mut Registry,
    incoming: &Registry,
) -> sysexits::Result<BTreeMap<String, String>> {
    let mut moved = BTreeMap::new();

    for (alias, contributor) in incoming {
        let existing = registry
            .values()
            .find(|known| {
                known
                    .emails
                    .iter()
                    .any(|email| contributor.emails.contains(email))
            })
            .map(|known| known.alias.clone());

        let landed = if let Some(existing) = existing {
            fold(
                registry
                    .get_mut(&existing)
                    .expect("the match was just found"),
                contributor,
            );
            existing
        } else {
            registry
                .entry(alias.clone())
                .and_modify(|known| fold(known, contributor))
                .or_insert_with(|| contributor.clone());
            alias.clone()
        };

        moved.insert(alias.clone(), landed);
    }

    consolidate(registry, &mut moved)?;
    Ok(moved)
}

/// Fold every pair of registry entries that share an e-mail into one.
fn consolidate(
    registry: &mut Registry,
    moved: &mut BTreeMap<String, String>,
) -> sysexits::Result<()> {
    while let Some((first, second, email)) = shared_email(registry) {
        let curated_first = curated(&registry[&first]);
        let curated_second = curated(&registry[&second]);

        if curated_first && curated_second {
            eprintln!(
                "git-harvest:  {first:?} and {second:?} both claim <{email}> \
                 and are each curated; run `git harvest id merge {first} \
                 {second} <alias>` to settle it"
            );
            return Err(sysexits::ExitCode::DataErr);
        }

        let (keep, drop) = if curated_second {
            (second, first)
        } else {
            (first, second)
        };

        let taken = registry.remove(&drop).expect("the loser was just found");
        fold(
            registry.get_mut(&keep).expect("the winner was just found"),
            &taken,
        );

        for landed in moved.values_mut() {
            if *landed == drop {
                landed.clone_from(&keep);
            }
        }
    }

    Ok(())
}

/// Two aliases that share an e-mail, and that e-mail, if any such pair is
/// left in `registry`.
fn shared_email(registry: &Registry) -> Option<(String, String, String)> {
    let contributors: Vec<&Contributor> = registry.values().collect();

    for (index, first) in contributors.iter().enumerate() {
        for second in &contributors[index + 1..] {
            if let Some(email) = first
                .emails
                .iter()
                .find(|email| second.emails.contains(email.as_str()))
            {
                return Some((
                    first.alias.clone(),
                    second.alias.clone(),
                    email.clone(),
                ));
            }
        }
    }

    None
}

/******************************************************************************/
