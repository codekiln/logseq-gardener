## Why

On the whole encode garden, `lsg`'s search loads one 32.2 MB corpus (5.6 MB gzipped): the first search on the phone profile takes 29.4 s and grows the heap by 75 MB, three times over the budgets. Its dialog also differs from Logseq's: one ungrouped list of 20 results, literal matching, no highlighting and no `/` filters ([Search analysis](../../../docs/parity/search.md), [Issue #118 — Close the remaining gaps between Logseq search and the static search](https://github.com/codekiln/logseq-gardener/issues/118)).

Logseq's own search needs the whole graph first (22.6 MB before any page) and then takes 776 ms per keystroke at the median on desktop and 2,950 ms on the phone profile. `lsg` can show Logseq's dialog while loading only a small title index on first open and the content shards each query needs.

## What Changes

- The SDK gains a search index shared by the CLI, the language server and the publisher. It holds one record per published page (title, aliases, route) and one per searchable block (text, page, the nearest three parents, destination), an inverted index from each normalized word to the blocks that contain it, and a query operation in Rust.
- The publisher writes that index as a title index (every page title and alias), content shards keyed by the first two characters of a word (split further when a shard grows past its budget), and record chunks that hold block text and breadcrumbs. Search format version 2 replaces version 1; the manifest link, `logseqGardenerInitSearch` and the host route resolver keep working as they do for Gitpa.
- Opening search fetches nothing before the dialog shows. The title index loads when the reader opens search or points at the search button; each keystroke fetches only the shards and record chunks its words need, and keeps fetched ones for the visit.
- Page titles and aliases match fuzzily: the query's characters in order, ranked by exact title, exact alias, prefix and then Logseq's fuzzy score, so `tmx` finds `tmux`. Blocks match when every query word is the start of a word in the block.
- The dialog takes Logseq's look and groups: Pages, Commands, Current page and Blocks, each with a header, a count badge (`99+` from 100) and five rows until `Show more ⌘ ↓`, `mod+Down` or a click on the group name expands it. Rows show the matched words highlighted; block rows show a breadcrumb of the page and up to three parents; alias rows show `alias of <page>`.
- Typing `/` shows a Filters group for pages, current page, blocks and commands. Choosing one shows a `Search only: <kind>` chip and lists every row of that group.
- Commands are the published site's own actions (theme, left sidebar, wide mode, and going to the home page, All pages and journals); they show on an empty input and when the query matches them.
- Keyboard: Up, Down, Ctrl+P and Ctrl+N move across groups and wrap; Enter opens; Escape clears the input, then the filter, then closes.
- `mise run parity:build` passes `--search`, and the `[search]` budgets in `scripts/parity/budgets.toml` change to the split index: title index size, largest shard, bytes fetched per keystroke, cold and warm keystroke latency, phone heap and build time, measured on the whole encode garden next to Logseq.
- **BREAKING** for hosts that read the version 1 corpus file directly: the corpus file is replaced by the title index, shards and record chunks. Hosts that use the manifest link and the initializer, as Gitpa does, need no change.

## Capabilities

### New Capabilities

- `garden-search-index`: the SDK search records, the word index, sharding and the Rust query, with the same results as the browser engine.

### Modified Capabilities

- `static-search-contract`: the published index becomes a title index, content shards and record chunks in format version 2; queries match titles fuzzily and blocks by word prefix, grouped like Logseq.
- `accessible-static-search`: the dialog takes Logseq's look, groups, five-row limit, highlighting, `/` filters, commands and keyboard, and loads shards per keystroke.
- `static-search-validation`: the budgets are measured on the whole encode garden with the parity build's search enabled.

## Impact

- SDK: a new `search` module beside `references` and `page_names` takes over the record building now in `crates/logseq-gardener-sdk/src/site/search.rs` and adds the word index and query.
- Publisher: `site/search.rs` writes the split artifacts; `site/search.mjs`, `site/search-ui.mjs` and `site/search-ui.css` change for the new loader, groups and look.
- Parity harness: `scripts/parity/budgets.toml` `[search]`, `parity:build`, `parity:perf --search-site` and `parity:budgets`.
- Gitpa keeps its integration through the manifest link and `logseqGardenerInitSearch`; its tests in `codekiln/gitpa` are read-only from this repository and are checked against a build of the encode garden.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
