## Why

Logseq reads every Markdown file under the garden folder, and `lsg` reads only `pages/` and `journals/`. That is the last unexplained gap in the whole-garden comparison: the encode garden's `mise-tasks/microfreak/PARAMETERS.md` and a report under `assets/` are pages in Logseq's `gh-pages` build and missing from `lsg` ([Issue #252 — Decide whether links to .md files create pages](https://github.com/codekiln/logseq-gardener/issues/252), [Issue #106 comment of 2026-10-28](https://github.com/codekiln/logseq-gardener/issues/106)).

## What Changes

- The SDK loads `.md` files from every folder of the garden, as Logseq's `get-files` does ([graph.cljs](https://github.com/logseq/logseq/blob/6e7afa8eb/deps/common/src/logseq/common/graph.cljs#L42-L91)). Files outside `journals/` load as pages and take their title from the file name, as files in `pages/` do.
- The SDK skips the paths Logseq skips: names starting with `.`, `node_modules/`, and Logseq's own `logseq/bak/`, `logseq/.recycle/` and `logseq/version-files/`.
- The SDK skips files that match an entry of `:hidden` in `logseq/config.edn`, with Logseq's prefix rule, in every folder including `pages/` and `journals/`.
- Outside `pages/` and `journals/`, the SDK also skips files git ignores, so a local build publishes what a build from a fresh checkout publishes. In the encode garden that keeps the 10 Markdown files of other repositories under the ignored `tools/` folder out.
- When a file outside `pages/` and `journals/` has the same title as a page in `pages/`, the page in `pages/` wins and the build names the skipped file.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-garden-loading`: the loader reads Markdown from every visible folder, applies Logseq's skip rules and `:hidden`, and skips git-ignored files outside the note folders.

## Impact

- SDK: `load_garden` in `crates/logseq-gardener-sdk/src/garden.rs` walks the garden root and reads `:hidden` through `garden_config.rs`. Every view (outline, references, queries, search, All pages, graph) gets the new pages because they all read the loaded documents.
- Publisher: no rendering change.
- Encode garden: two pages join the build, `PARAMETERS` and `AI__Coding__Tool__Plugin__Report__26__Agentic_Coding_Plugin_Systems_and_Cross_Provider_Portability`. No page in `pages/` or `journals/` matches the garden's `:hidden` list today, so no page leaves.
- Performance: the walk visits about 5,200 more directory entries, most of them under the skipped `logseq/bak/`, plus one `git ls-files` call. Page sizes, scripts and search data per page are unchanged.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
