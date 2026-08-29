# git-harvest

Harvest a CHANGELOG from your Git history.

`git-harvest` turns the commits you already write into a CHANGELOG.

## Status

Furniture only.  The harvesting and assembly logic is not written yet.  This
repository currently holds the project scaffold — manifest, licence, continuous
integration and the conventions harness — so that the first feature lands on a
finished workbench.

## How it will work

Two stages:

1. **Harvest.**  Each structured commit subject — `[Category] scope: subject`
   — becomes a RON fragment under `changelog.d/`, one file per branch.
2. **Assemble.**  A second pass merges the fragments into a machine-owned RON
   CHANGELOG, stamps it with the release version and date, and renders a Keep
   a Changelog `CHANGELOG.md` beside it.

The tool assembles release notes only.  Bumping the version stays with each
repository's own release workflow.

## Licence

GNU General Public License v3.0 or later.  See `LICENCE` for the full text.

<!-------------------------------------------------------------------------- -->
