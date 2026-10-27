## Context

The [search analysis](../../../docs/parity/search.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `0ce74d6`, `gh-pages` `8132c95`) with an `lsg` build from `1e72969` plus `--search`. `crates/logseq-gardener-sdk/src/site/search.rs` builds one page record per published page and one content record per bullet from the parsed garden, and writes them as one corpus file, `lsg-search-<revision>.json`: 101,671 records, 32.2 MB decoded, 5.6 MB gzipped. `search.mjs` evaluates a query against the whole corpus in memory; `search-ui.mjs` loads manifest, corpus and engine on first open and draws one list of 20 results.

Gitpa hosts the same dialog. It reads `link[rel=lsg-search]` from swapped pages and calls `logseqGardenerInitSearch(document, { resolvePage })` ([`site/navigation.js`](https://github.com/codekiln/gitpa/blob/main/site/navigation.js)). Both stay as they are.

The right sidebar change ([PR #204](https://github.com/codekiln/logseq-gardener/pull/204), `match-logseq-right-sidebar`) is still a proposal; this change leaves Shift+Enter and Mod+Enter to the sidebar work.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's search dialog: its look, groups, five rows with Show more, highlighting, `/` filters, commands and keyboard.
- The first open shows the dialog at once and page results soon after, on the whole encode garden, on a phone profile; each keystroke loads only what its words need.
- The CLI and the language server can answer the same query with the same results as the published site.

**Non-Goals:**

- Opening a result or the whole search in the right sidebar with Shift+Enter or Mod+Enter, after `match-logseq-right-sidebar` lands ([Issue #119](https://github.com/codekiln/logseq-gardener/issues/119)).
- An `lsg search` CLI command over the new SDK query.
- Fuzzy matching inside block text, so typos find blocks as well as pages.

## Decisions

**Where the work lives.** The SDK builds and queries the index; the publisher writes files and draws the dialog.

| Piece | Lives in | Answer |
| --- | --- | --- |
| `search::records(garden, published)` | SDK | Page records and block records, with the privacy rules now in `site/search.rs`. |
| `search::WordIndex` | SDK | Normalized word → block record numbers, in record order. |
| `search::query(index, query, filter)` | SDK | Grouped, ordered results and full counts, the same as the browser engine. |
| `search::shards(index, budget)` | SDK | The prefix shards and record chunks, each with a content digest, so a rebuild can reuse unchanged ones. |
| Writing files, manifest, dialog, styles, commands, keyboard | Publisher | `site/search.rs`, `search.mjs`, `search-ui.mjs`, `search-ui.css`. |

Moving the record building into the SDK lets the language server and CLI use the same records. The shared fixture test (`tests/search-fixture.json`, queries with expected results) runs in `cargo test` and in the Node tests of `search.mjs`, so the two engines cannot drift.

**Files.** All names below are relative to the manifest.

| File | Holds | Loaded |
| --- | --- | --- |
| `lsg-search-manifest-<revision>.json` | format version 2, revision, the title index name and digest, a map from shard prefix to shard file and digest, the list of chunk files and digests, and the engine name | on first open or button hover/focus |
| `lsg-search-titles-<digest>.json` | per page: title, route id, aliases | with the manifest |
| `lsg-search-shard-<digest>.json` | for each word with the shard's prefix: the word and its block record numbers, delta-encoded | when a query word needs the prefix |
| `lsg-search-chunk-<digest>.json` | 256 consecutive block records: text, page id, up to three parents, fragment | when a result row from the chunk is drawn |

Shards are keyed by the first two characters of a word and split by the next character while one exceeds 64 KB gzip. File names use content digests, so a rebuild after one edit changes only the manifest, the shards whose words changed and the chunk that holds the edited page; browsers keep the rest in cache. Block records are ordered by page title and position, so one page's blocks share a chunk and a page's results draw from few chunks.

**Matching.** Page titles and aliases: the query's characters, ignoring spaces, brackets, slashes, underscores and parentheses as Logseq does, must appear in order. Ranking: exact title, exact alias, prefix, then Logseq's fuzzy `score` from [`search.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search.cljs#L30-L90), the same port the All pages change uses. Blocks: every query word must be the start of a word in the block, found through the shards: the words in a shard that start with the query word give the block numbers, and the blocks that appear for every query word match. Counts are exact, from the block numbers; rows are at most 100 per group, as Logseq's.

**Loading.** The existing starter still loads after paint. The dialog module, the engine, the manifest and the title index start loading on pointer enter or focus of the search button, and on Cmd/Ctrl+K; the dialog shows with Commands as soon as its module arrives, before any search data. Each keystroke computes the prefixes its words need, fetches missing shards in parallel, then fetches only the chunks for rows it draws. A result list waits for its shards; rows whose chunk is still loading show the page title and a placeholder line. Fetched files stay in memory for the visit, keyed by publication.

**Groups and commands.** Filters, Pages, Commands, Current page and Blocks, in Logseq's order, leaving out Create, Files, Whiteboards and Recents. The commands are the actions the shell already has: toggle the theme, the left sidebar and wide mode, and go to the home page, All pages and journals when the site has them. The footer tip rotates between `Type / to filter search results` and the Show more hint.

**Look.** `search-ui.css` takes the measured light values from the analysis: 896px (`90dvw`, max `56rem`) dialog with 8px radius; 20px transparent borderless input on a grey row with a bottom border; 32px group headers in 12px grey text with a bold name and a count badge; 14px weight 500 rows with a 20×20 icon box; highlighted row on the grey hover background; matched words in `span.ui__list-item-highlighted-span` on `rgb(198,212,249)`. Dark values use the existing theme tokens. The dialog keeps `aria-label="Search"` and a visually hidden input label in place of the visible title, and closes on Escape or a click outside it.

**Budgets.** Measured by `parity:perf --search-site` on the whole encode garden with [`scripts/parity/search-queries.txt`](../../../scripts/parity/search-queries.txt), next to Logseq's build:

| Measure | Budget |
| --- | --- |
| Search script before the title | unchanged: the 2 KB bootstrap, within `script_before_title_kb` |
| Dialog visible after Cmd/Ctrl+K, first open | 100 ms desktop, 300 ms phone |
| Title index, gzip and decoded | 300 KB and 1.5 MB |
| First page results after first open | 500 ms desktop, 2,500 ms phone |
| Largest shard, gzip | 64 KB |
| Largest record chunk, gzip | 32 KB |
| Bytes fetched by one keystroke, gzip | 200 KB |
| Keystroke that fetches a shard, p95 | 150 ms desktop, 1,000 ms phone |
| Keystroke on loaded shards, p95 | unchanged: 50 ms desktop, 100 ms phone |
| Phone heap growth after the whole query list | unchanged: 30 MB |
| Whole-garden build time with `--search` | at most 25% above the build without it |

`scripts/parity/budgets.toml` `[search]` replaces `corpus_decoded_mb` and `corpus_gzip_kb` with these keys. `parity:budgets` checks the title index, shard and chunk sizes in CI on the fixture garden and fails naming the largest offender; `parity:perf` marks the timing, heap and build values in `docs/parity/performance.md`. `parity:build` passes `--search`.

**Comparison with Pagefind.** [Pagefind](https://github.com/CloudCannon/pagefind) splits its index the same way. Before writing the shard format, the Apply step indexes the same `lsg` build with Pagefind in `experiments/pagefind/` and records its index size, bytes per query and keystroke latency beside the budgets above. The purpose-built index stays only if it is within those numbers; it is needed anyway for Logseq's block groups, aliases, breadcrumbs and block anchors.

## Risks / Trade-offs

- [Block text has no typo tolerance] → Logseq's Fuse match finds blocks with typos; word-prefix matching does not. Page titles keep fuzzy matching, which covers the common case of a misspelt page name. Fuzzy block search is listed as a non-goal.
- [A query word of one character fetches a large set] → One-character words go to their own shard, and a one-character query searches only titles until a second character arrives, as titles are already in memory.
- [Many small files] → The encode garden needs about 2,000 shards and 400 chunks; digest names make them cacheable, and the build report counts them separately from pages.
- [Gitpa depends on the version 1 shape] → Gitpa reads only the manifest link and the initializer, which stay; its tests in `codekiln/gitpa` are run against a new encode-garden build during Apply, and any failure is reported on [Issue #118](https://github.com/codekiln/logseq-gardener/issues/118).
- [The Rust and browser engines drift] → The shared fixture fails `mise run ci` when they disagree.

## Resolved Questions

### 1 - How is the index split?

Decided by the agent without codekiln: a title index loaded on first open, content shards keyed by two-character word prefixes, and record chunks of 256 block records, all named by content digest. Logseq's approach, the whole graph in memory, is what the 32 MB corpus copies and what fails the phone budgets; prefix shards keep each keystroke's download bounded and match the client-side approach on Issue #106.

### 2 - How fuzzy is matching?

Decided by the agent without codekiln: page titles and aliases match with Logseq's in-order characters rule and fuzzy score; blocks match by word prefix. Fuzzy block matching would need every block's text on the client, which is the load-everything cost the shards avoid.

### 3 - Does the Create group appear?

Decided by the agent without codekiln: no. Logseq's published `Create page` makes a page only in the reader's memory, which disappears on reload; a static site has nothing to create.

### 4 - Which commands appear?

Decided by the agent without codekiln: the published site's own actions (theme, left sidebar, wide mode, home, All pages, journals). Logseq lists 70 commands, most of which edit the graph; listing only the ones that work keeps every row honest.

### 5 - Does Recents appear?

Decided by the agent without codekiln: no. The live published build showed no Recents group after opening pages in the same session, and keeping a history would need storage on the reader's device.

### 6 - Which `/` filters appear?

Decided by the agent without codekiln: pages, current page, blocks and commands. `lsg` publishes no whiteboards or files, so those filters would always be empty.

### 7 - Do the visible title and Close button stay?

Decided by the agent without codekiln: no, to match Logseq's look. The dialog keeps an accessible name and a hidden input label, and closes on Escape and on a click or tap outside it, so keyboard, screen reader and touch readers can still name and leave it.

### 8 - What happens to search format version 1?

Decided by the agent without codekiln: version 2 replaces it, and the controller rejects version 1 with the existing reload message. Only `lsg`'s own controller reads the files; hosts use the manifest link and the initializer, which keep working.

### 9 - Where does the index code live?

Decided by the agent without codekiln: records, word index, sharding and query in the SDK, file writing and the dialog in the publisher. Issue #106 asks that search results agree with what `lsg` and the language server report, and the shared fixture enforces it.

## Open Questions

None.
