# Search

Logseq's search dialog groups results by kind (Pages, Blocks, Commands, Create, and others), shows five rows per group with `Show more`, highlights the matched text, narrows to one group through `/` filters, and opens a result in the right sidebar with Shift+Enter. `lsg`'s search, built in #94–#96, shows one ungrouped list of at most 20 results with no highlighting, no filters and no sidebar.

The larger problem is size. On the whole encode garden, `lsg`'s search corpus is one 32.2 MB JSON file (5.6 MB gzipped) with 101,671 records. The first search on the phone profile takes 29.4 s and grows the heap by 75 MB, over the budgets in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml) (8 MB decoded, 1 MB gzipped, 10 s, 30 MB). The [static search validation](search-performance.md) measured only 996 pages, and the parity build does not pass `--search`, so this was not visible before. Matching Logseq's groups is mostly dialog work in the publisher. Meeting the budgets on the whole garden needs a split, incrementally loaded index built from the SDK's garden index.

Logseq is far slower: 776 ms median and 1,836 ms 95th-percentile per keystroke on desktop, 2,950 ms and 13,675 ms on the phone profile, after the whole graph has loaded ([Parity performance](performance.md)). `lsg` already types faster once its corpus has loaded: 8.8 ms median and 36.9 ms 95th percentile on desktop, 39.7 ms and 85.1 ms on phone.

Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (`gh-pages` `8132c95`, served locally, light theme, 1280×800) and an `lsg` build of the same commit from `lsg` `1e72969` with the `parity:build` options plus `--search`. Timings come from `mise run parity:perf --search-site` with [`scripts/parity/search-queries.txt`](../../scripts/parity/search-queries.txt).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`; `cmdk.cljs` = `frontend/components/cmdk.cljs`; `list_item` = `deps/shui/src/logseq/shui/list_item/v1.cljs`). Measured live values come first and win where they differ from the checkout or from `docs/logseq-reference/Overlays.md` and `Prototype.md`. The board's 720px white dialog, bold matches and `PAGES`/`BLOCKS` labels do not match the live build.

None of the search code checks whether the site is published, so the published dialog is the desktop app's dialog, including commands and page creation.

### Opening

- The header search button, `mod+k`, `mod+shift+p` (commands only) and `mod+shift+k` (current page only) open it ([`shortcut/config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/modules/shortcut/config.cljs#L364-L369), [`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L897-L901)). Other shortcuts are off while it is open.
- A modal 896px wide (`90dvw`, max `56rem`), 8px radius, with no title and no close button ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L966)).
- The input: 20px, transparent, no border, placeholder `What are you looking for?`, on a grey row with a bottom border ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L733-L753)).
- The results area is 65% of the viewport height and scrolls. A footer shows one tip (`Type / to filter search results`, `⌘ ⏎ to open search in the sidebar`) and the keys for the highlighted row (`Open ⏎`, `Open in sidebar ⇧ ⏎`).

### Groups

- Order: Filters (when the input contains `/`), Pages, Create, Commands, Current page, Blocks, Files, Recents. Empty groups are left out ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L105-L144)).
- Each group header is 32px, 12px grey text with a bold name, then a count badge that reads `99+` at 100 or more ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L574-L604)).
- **Empty input:** Commands (70 in the published build, starting `Toggle wide mode`, `Toggle between dark/light theme`, `Toggle settings`) and Recents. Recents reads recently opened pages from Logseq's in-memory database ([`handler/recent.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/recent.cljs#L6-L18)); after opening `mise` in the same session, the measured dialog still showed only Commands.
- **Query `tmux`:** Pages `99+` (`tmux`, `tmux/Option/pane-border-style`, `tmux.conf.local` with `alias of tmux/Config/local`, `tmux/Window/Name`, `tmux/session/Picker/Order`), then Blocks `99+`, each row a breadcrumb of the page and up to three parents above the block text. Blocks on the page being viewed move to a Current page group ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L298-L311)).
- **Query `logseq query`:** no Pages group, a Create group (`Create page`, `Create whiteboard`), and Blocks `99+` starting with `Logseq/MCP/Server/Survey/Report`. Create appears whenever no page has exactly the typed name ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L103-L104)); in the published build it creates a page only in the reader's memory.

### Five rows and Show more

- Each group shows 5 rows unless it is expanded, a filter is set, or the search is in the right sidebar ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L43), [L92–L102](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L92-L102)).
- The group that holds the highlighted row shows `Show more ⌘ ↓` (or `Show less ⌘ ↑`) in its header; `mod+Down` and `mod+Up` do the same, and clicking the group name toggles it ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L592-L604), [L697–L702](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L697-L702)).

### `/` filters

- Typing `/` shows a Filters group with six rows, each `Search only <kind> — Add filter to search`: pages, current page, blocks, commands, whiteboards, files ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L45-L57)). Text after the `/` narrows them by fuzzy match. Pages and Blocks that match `/` follow, so a bare `/` also lists blocks containing a slash.
- Choosing a filter removes the `/…` text and shows a `Search only: <kind>` chip with `×`. Escape on an empty input or Backspace on an empty input removes it ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L515-L532), [L764–L776](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L764-L776)).
- With a filter set, the group shows all its rows. The blocks filter also shows Current page.

### Matching

- **Pages:** a fuzzy match (Fuse, threshold 0.5) on the page name with brackets, slashes, underscores, parentheses and spaces removed, at most 100 results, then kept only when the query's characters appear in order in the name ([`search.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search.cljs#L41-L43), [L121–L157](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search.cljs#L121-L157), [`search/db.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search/db.cljs#L101-L120)). Aliases show `alias of <page>`.
- **Blocks:** a fuzzy match (Fuse, threshold 0.35) on the raw block text including property lines, at most 100 results ([`search/db.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search/db.cljs#L82-L93), [`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L280)).
- **Highlight:** each whitespace-separated query word is marked case-insensitively in the row text and its breadcrumb, as `span.ui__list-item-highlighted-span` on `rgb(198,212,249)`, 14px weight 500 ([`list_item`](https://github.com/logseq/og/blob/6e7afa8eb/deps/shui/src/logseq/shui/list_item/v1.cljs#L18-L41)).
- Rows are 14px weight 500 with a 20×20 icon box; the highlighted row has a grey background ([`list_item`](https://github.com/logseq/og/blob/6e7afa8eb/deps/shui/src/logseq/shui/list_item/v1.cljs#L57-L92)).
- Input changes are handled at most every 50 ms ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L664-L670)).

### Keyboard

- Down or Ctrl+N, Up or Ctrl+P move through all visible rows across groups and wrap ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L642-L716)).
- Enter opens a page, opens a block at `#ls-block-<uuid>` on its page (or zooms into it when a parent is collapsed), runs a command or applies a filter.
- Shift+Enter opens the page or block in the right sidebar; Mod+Enter adds the whole search to the right sidebar; Mod+C copies a block reference.
- Escape clears the input, then the filter, then closes. There is no Tab movement between groups.

### Performance in Logseq

Search runs against the whole graph, which the published site loads before any page draws (22.6 MB transferred, about 2 GB peak heap; [Parity performance](performance.md)). Both fuzzy indexes are built on the first search.

| Profile | Median per key (ms) | 95th percentile (ms) |
| --- | ---: | ---: |
| desktop | 776 | 1,836 |
| phone | 2,950 | 13,675 |

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/search.rs`, `search-ui.mjs`, `search.mjs` and the [static search contract](../static-search.md), checked against the whole-garden build:

- **Opening:** a header button and Cmd/Ctrl+K open a native `<dialog>` titled `Search garden` with a `Close` button and a visible label `Search published pages and notes`. The dialog is 680px wide with 20px padding and a 10px radius; the input is 16px with a border and no placeholder. Empty input shows `Type to search published pages and notes.`
- **Results:** one list of at most 20 results with a live status line (`20 results shown.`). Page results show the title; content results show the page title, the breadcrumb joined with `›`, and a bounded snippet. Ranking puts exact titles, exact aliases, prefixes and all-term title matches before content matches; ties sort by title. For `tmux` the list is 20 `tmux…` titles in alphabetical order and no content result.
- **Matching:** every query word must appear as a literal substring of one title, alias or bullet. There is no fuzzy match, so typos find nothing.
- **Missing:** groups, the five-row limit with `Show more`, the count badges, highlighting, `alias of`, the Current page group, `/` filters (`/` matches titles containing a slash: `10/3/3`, `12factor/app`), Recents, commands, the footer hints, Shift+Enter and Mod+Enter.
- **Not indexed:** property values, generated namespace titles and code fences.
- **Size and speed on the whole garden** (`parity:perf --search-site`):

| Profile | First open (ms) | Transferred (MB) | Decoded (MB) | Heap growth (MB) | Warm open (ms) | Median per key (ms) | 95th percentile (ms) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| desktop | 845 | 5.63 | 32.3 | **47.1** | 13 | 8.8 | 36.9 |
| phone | **29,449** | 5.63 | 32.3 | **74.7** | 30 | 39.7 | 85.1 |

Bold values exceed the `[search]` budgets; the corpus also exceeds `corpus_decoded_mb = 8` and `corpus_gzip_kb = 1000`. Page first paint is unaffected: 12 KB data and 9.7 KB script before the title.

- **The parity build:** `mise run parity:build` does not pass `--search`, so the committed `parity:perf` rows for `lsg` search time out instead of measuring.
- **Where the code lives:** the corpus is built inside the publisher's `site` module from each rendered page; the CLI and the language server cannot query it.

## Gaps

Each gap names a check the Apply step can run.

- 1. **Whole-garden budgets.** The first open on the phone profile stays under 10 s and heap growth under 30 MB on the whole encode garden, which one 32 MB corpus cannot meet. The index needs to be split so the first open loads a small page-title index and content shards load as queries need them. Check: `parity:perf --search-site` on the whole garden passes every `[search]` budget.
- 2. **The parity build enables search.** `parity:build` passes `--search`, so `parity:perf` records `lsg` search next to Logseq's. Check: the committed search rows have numbers for both builds.
- 3. **Groups.** Pages, then Current page, then Blocks, each with a header, a count badge (`99+` at 100) and empty groups left out. Check: `tmux` shows Pages before Blocks with `99+` on both; on a page whose own blocks match the query, those blocks appear under Current page and not under Blocks.
- 4. **Five rows and Show more.** Five rows per group; `Show more ⌘ ↓` / `Show less ⌘ ↑` on the highlighted group; `mod+Down` and `mod+Up`; clicking the group name toggles. Check: `tmux` Pages shows five rows, then all after `mod+Down`.
- 5. **Block rows.** A breadcrumb of the page and up to three parents above the block text. Check: a `tmux` block result's first line is its page title.
- 6. **Highlighting.** Each query word marked in titles, block text and breadcrumbs with Logseq's colour. Check: `tmux` rows mark `tmux`.
- 7. **Aliases.** An alias result shows `alias of <page>`. Check: `tmux.conf.local` shows `alias of tmux/Config/local`.
- 8. **Fuzzy matching.** Characters in order rather than whole literal words, so `tmx` still finds `tmux`. The ranking may differ from Fuse's as long as exact and prefix title matches lead. Check: `tmx` lists `tmux` first.
- 9. **`/` filters.** A Filters group for pages, current page and blocks, the `Search only: <kind>` chip, Escape and Backspace removal, and full groups while filtered. Whiteboards and files have no `lsg` content; the proposal decides whether those rows and Commands appear. Check: `/` lists the filters; choosing pages then typing `tmux` lists every `tmux` page.
- 10. **Dialog look.** 896px, 20px input without border, placeholder `What are you looking for?`, grey group headers, 14px rows with icons, the highlighted-row background, the footer tip and key hints. Check: computed styles against the values above.
- 11. **Keyboard.** Arrow keys and Ctrl+N/Ctrl+P across groups with wrapping; Enter opens; Escape clears input, then filter, then closes. Shift+Enter and Mod+Enter need the right sidebar (#119). Check: the browser test drives each key.
- 12. **Empty input.** Logseq shows Commands and Recents. The proposal decides which commands make sense in a published `lsg` site (those that navigate or toggle the theme) and whether Recents is kept per visit. Check: the decided groups appear on an empty input.
- 13. **Create.** Logseq offers `Create page` for unknown names, which a static site cannot do. The proposal decides whether to leave it out. Check: no Create group, or the decided equivalent.

## Where the work belongs

- **SDK:** the searchable records (titles, aliases, block text, breadcrumbs, destinations) come from the garden index that already holds pages, blocks and references, so the CLI and the language server can run the same query. Splitting the index into a title index and content shards, and keeping the shards stable when one page changes, is index work that a rebuild can update incrementally.
- **Publisher:** writing the shards, the dialog markup and styles, grouping, the five-row limit, highlighting, filters, keyboard handling and the budget rows.

## Fixture queries and pages

| Query or page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `tmux` | Many page and block results; an alias | Pages `99+`, Blocks `99+`; `tmux.conf.local` alias of `tmux/Config/local` | 20 titles, no blocks |
| `logseq query` | No page match; block results | Create, Blocks `99+` | 20 results, pages and blocks mixed |
| `/` | Filters | 6 filters, then blocks containing `/` | 20 titles containing `/` |
| `tmx` | Fuzzy match | not measured | not measured; literal matching predicts no `tmux` result |
| A page whose blocks match the query | Current page group | its matching blocks under Current page (no Current page group appeared for `tmux` on the `tmux` page) | no group |
| Whole encode garden | Corpus size and phone budgets | 22.6 MB graph before any page | 32.2 MB corpus, 29.4 s phone first open |

## Out of scope

- Opening results or the whole search in the right sidebar, which belongs with [Issue #119](https://github.com/codekiln/logseq-gardener/issues/119).
- Mod+C to copy a block reference, which depends on block references being copyable elsewhere in the published site.
- Whiteboards and files, which `lsg` does not publish.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
