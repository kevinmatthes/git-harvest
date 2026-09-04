# `git-harvest`

Harvest a CHANGELOG from your Git history.

`git-harvest` turns the commits you already write into a CHANGELOG.

> **Made with Anthropic Claude.**  The implementation and the documentation
> were written by Claude, working to the direction and review of the author,
> with whom every design decision rests.  Each commit names the model in a
> `Co-Authored-By` trailer, so the record is per change rather than only
> here.

## Status

Early, but working end to end.  Four subcommands cover both passes:  `init`,
`scan`, `assemble` and `render`.  RON is the only fragment and CHANGELOG
format so far, the release version and date are supplied on the command
line, and there is no forge integration yet.  The repository holds the full
project scaffold — manifest, licence, continuous integration and the
conventions harness.

## How it works

Two passes, four subcommands.

`git harvest init` writes a starter `CHANGELOG.ron` carrying the default
configuration:  the commit-subject delimiter, the grammar, the bucket
vocabulary and the renderer.  It refuses to overwrite an existing file
without `--force`.

`git harvest scan`, run on a feature branch, reads that branch's structured
commit subjects — by default `Bucket ::= entry`, for example
`Added ::= a new option` — and writes them to a RON fragment under
`changelog.d/`, one file per branch.  Only commits absent from `main` are
read, and merges are skipped.  The grammar is configurable.

`git harvest assemble <version>` folds every fragment into a new section of
`CHANGELOG.ron`, stamps it with that version and the release date, inserts
it in descending version order, and deletes the fragments it consumed.

`git harvest render` writes a Keep a Changelog Markdown file from the
released sections of `CHANGELOG.ron`.

`git harvest licences` reproduces the verbatim licence notices of
`git-harvest` and every dependency it ships, harvested at build time by
[`list-my-licence`][list-my-licence].  Pass a crate name to narrow the
report to one package.

Because the binary is named `git-harvest`, Git runs it as a subcommand:
`git harvest init`, `git harvest scan` and the rest work with no further
setup.

Assembling release notes is the tool's job; version bumping stays with each
repository's own release workflow.

## Licence

GNU General Public License v3.0 or later.  See [`LICENCE`](LICENCE) for the
full text.

[list-my-licence]: https://crates.io/crates/list-my-licence

<!-------------------------------------------------------------------------- -->
