# Proposal

## Why

`lsg publish` selects pages by namespace root, and an empty include list selects nothing. To publish a whole garden, a build script must work out every page title the way `lsg` does and pass one `--include` per top-level namespace. The encode garden's `site:build` passes 1,048 of them, and this repository's `parity:build` derives its roots from file names. A title that `lsg` derives differently leaves a page out without an error. Since #454, a page that names itself in another spelling with `alias::` takes that spelling as its title: `pages/Solvespace.md` holds `alias:: [[SolveSpace]]` and is titled `SolveSpace`, so `--include Solvespace` misses it (issue #350).

## What Changes

- `lsg publish` gains `--include-all`, which selects every page of the garden. With `--journals`, it publishes every page and journal.
- `--exclude` still applies to pages under `--include-all`. `--include` roots given beside it are checked as before and change nothing.
- Pages and journals withheld by `public:: false`, by nesting or by a missing `public:: true` stay withheld, as under any selection.
- The SDK's namespace selection gains a way to select every page, so other SDK callers get the same behaviour.
- `mise run parity:build` passes `--include-all --journals` instead of one `--include` per root taken from file names, and stops recording skipped files.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publishing-namespace-selection`: a selection can include every page, with exclusions still applied.
- `cli-static-publishing`: `lsg publish` accepts `--include-all`.
- `parity-comparison-harness`: `parity:build` selects the whole garden with `--include-all`.

## Impact

- SDK: `NamespaceSelection` in `crates/logseq-gardener-sdk/src/publishing.rs`.
- CLI: `src/cli.rs` and the publish help in `src/help.rs`.
- Parity harness: `scripts/parity/build.py`, its tests and the Parity Tasks reference page.
- Docs graph: the Publish Options and Selection and Visibility reference pages.
- Follow-up outside this repository: the encode garden's `site:build` can drop its title derivation and pass `--include-all`.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
