## Verification

`lsg` now writes every `{{query}}` and `#+BEGIN_QUERY` block as Logseq's result box, evaluated during the build. Eight of the analysis's nine result counts match Logseq exactly on the garden at `083106d`; `(between -30d today)` finds 75 blocks where Logseq finds 76, for the reason under [Remaining difference](#remaining-difference). The browser runs no query: opening the collapsed 603-result tmux query draws its first group in 20 ms on desktop and 618 ms on the phone profile, where Logseq needs 19.8 s and 185 s before the page shows anything.

Measured on 2026-10-28. Logseq's results come from the `gh-pages` build `8132c95` of garden commit `083106d` (the build the [analysis](../../../docs/parity/queries.md) measured), read from its `static/js/db.gz` with the `query_dsl.cljs` rules applied in Python; they reproduce every count the analysis measured in the browser. `lsg`'s results come from a release build of this branch over an archive of the same commit with one page, `Parity/Queries`, holding the test queries (`scripts/parity/testdata/queries-page.md`), built with `SOURCE_DATE_EPOCH` on 2026-10-27.

### Result counts

| Query | Logseq | `lsg` |
| --- | --- | --- |
| `(and [[yazi]] [[card]])` | 18 results, 13 groups | 18 results, 13 groups |
| `(page-tags [[CLI/Tool]])` | `Live query for pages`, 60 results, no toggle | the same |
| `(and [[yazi]] [[Rust]])` | 5 results | 5 results |
| `(and (task TODO DOING) [[tmux]])` | 2 results | 2 results |
| `(between -30d today)` | 76 results on 29 journals | 75 results on the same 29 journals |
| `"lightweight database"` | 1 result | 1 result |
| `(page-property logseq-entity [[Logseq/Entity/Proxy/Page]])` | `Live query for pages`, 6 results | the same |
| `#+BEGIN_QUERY` `[[tmux]]`, `:collapsed? true` | closed, no count; 603 results on 259 pages when opened | the same |
| `{{query [[tmux]]}}` | first groups `2026-09-19 Sat`, `2026-09-15 Tue`, `2026-09-11 Fri` | the same |
| `{{query (and [[tmux]]}}` | `Query Error:` box | `Query Error:` box and one diagnostic |

Four SDK rules were needed to reach these counts, each matching Logseq's database: a page's leading properties are a block of their own and no other block's ancestor; text before the first bullet, such as `# [[Filed]]` on a journal, is the parent of the bullets indented under it, unless page properties come first; page properties and `tags::` come only from properties that open the page, with the last value of a repeated key winning (so `starship` and three pages whose `tags::` sit under a heading are not tagged `CLI/Tool`); and a tag with no page file, such as `#card`, matches the blocks that write it.

The Datalog test queries (`:inputs [:current-page]`, `[:find ?name …]` with `:view :pprint`, an unknown predicate) render the decided `Query failed:` line, as Resolved Question 2 says.

### Remaining difference

`2026-10-14 Wed` has `# [[Filed]]` without a bullet after a bullet. Logseq reads it as a new top-level block; the outline parser that Linked References also use reads it as part of the bullet above, so `(between -30d today)` misses that one top-level block.

### Styles and behaviour

Computed in Chromium at 1280×800 on `Parity/Queries` (light theme), against the analysis's measured values:

| Element | Logseq | `lsg` |
| --- | --- | --- |
| Header `.custom-query > .th` | 12px, opacity 0.9, margin `4px 2px`, padding `16px 24px`, `rgb(248,249,250)`, radius 8px, ring `0 0 0 2px rgb(233,236,239)`, 52px tall | the same, 52px tall |
| Group `.custom-query-page-result` | margin `8px 0`, padding 8px, `rgb(248,249,250)`, radius 4px, the same ring | the same |
| Group title | 16px, `rgb(16,107,163)` | the same |
| `Query Error:` heading | 13.28px, weight 600, `rgb(235,144,145)` | the same |
| Clause row | `( AND [[yazi]] [[card]] )` | the same text, without the `+` button |

In the browser, on the fixture garden's `Tools/Queries` and on `Parity/Queries`:

- `Switch to table view` on a block query fetched its table from one fragment and showed the same blocks as rows; a second click restored the list without a request.
- Clicking a table's column title sorted its rows and set `aria-sort`; a second click reversed them.
- Opening the collapsed tmux query set `603 results` and drew its first groups from the first fragment; scrolling loaded the rest. After every fragment on the page had loaded, 568 groups were in the document.
- `tmux` links neither `lsg-queries.js` nor `lsg-queries.css`; only `Parity/Queries` and `Logseq/Queries` link them in the whole-garden build.
- `Logseq/Queries` shows the `Live query` header with no count, no results and no literal `{{query`. `` `{{query}}` `` in inline code on `Logseq/Flavored Markdown` and `{{query}}` in a fenced block on `LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation` stay literal.
- The garden's two default journal queries are Datalog: the build lists two `Default journal query skipped` diagnostics and draws no default query box.

### Sizes and time

- `mise run parity:budgets` on the fixture garden: `lsg-queries.js` 1,960 bytes (budget 2,000), `lsg-queries.css` 581 bytes gzip (budget 1,000); `Tools/Queries` within the first-paint budgets and unchanged with 2,000 added pages.
- Whole garden: the `Parity/Queries` page with nine boxes is 46.8 KB gzip; the largest query fragment is 49.9 KB gzip (budget 100 KB).
- `mise run parity:perf --queries --baseline-lsg <origin/main build>` on the garden at the latest run: title 121 ms desktop and 374 ms phone with 57,390 bytes before the title; open 20 ms desktop and 618 ms phone (budgets 300 and 1,500); phone heap 1.2 MB after every fragment (budget 20); whole-garden build 14.48 s against 14.25 s (+1.6%, budget 10%). The rows are in [Parity performance](../../../docs/parity/performance.md#queries).
