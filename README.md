# git-harvest

Harvest a CHANGELOG from your Git history.

`git-harvest` turns the commits you already write into a CHANGELOG.

## Status

Early.  `git-harvest init` writes a starter CHANGELOG carrying the default
configuration; the harvest and assembly passes are next.  The repository holds
the full project scaffold — manifest, licence, continuous integration and the
conventions harness.

## How it will work

Two passes:

1. **Harvest.**  Each structured commit subject — by default `Bucket ::= entry`,
   for example `Added ::= a new option` — becomes a RON fragment under
   `changelog.d/`, one file per branch.  The grammar is configurable.
2. **Assemble.**  A second pass merges the fragments into a machine-owned RON
   CHANGELOG, stamps it with the release version and date, and renders a Keep
   a Changelog Markdown file beside it.

Assembling release notes is the tool's job; version bumping stays with each
repository's own release workflow.

## Licence

GNU General Public License v3.0 or later.  See `LICENCE` for the full text.

<!-------------------------------------------------------------------------- -->
