## Context

The [page outline analysis](../../../docs/parity/page-outline.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `09cf7fb`) with an `lsg` build from `2cead35`. The content of every outline matches; the look and the reader controls do not. Logseq draws each block as `div.ls-block` with a bullet column, content and a `block-children` container with a left border. `lsg` writes nested `<ul><li>` lists, and an authored collapsed block becomes `<li class="outline-collapsed"><details class="outline-disclosure"><summary>…</summary><ul>…</ul></details></li>` (`blocks` in `crates/logseq-gardener-sdk/src/site/render.rs`). The renderer works out collapse, heading level and task marker inline from the parser's flat bullet list.

Logseq can zoom into a block only after loading the whole graph: a 15.8 MB `db.gz` and about 35 MB of script. Its zoom links (`#/page/<uuid>`) use a random UUID for every block without `id::`, so they change on every build. Zoom is the part of this feature where copying Logseq's design would cost most, so it runs on the page already loaded.

The `tmux` page is the size limit to watch: 1,061 blocks and 94 KB gzip against the 100 KB budget for data before the title is visible.

## Goals / Non-Goals

**Goals:**

- Readers of the encode garden see Logseq's title, bullets, guide lines, headings and task markers on every page and in the journals feed.
- Readers can collapse any block with children and zoom into any block, with addresses they can share and that survive rebuilds of unchanged content.
- First paint stays independent of garden size: the outline adds CSS and a few bytes per block, and its script loads after `load`.
- The CLI and the future language server get block depth, ancestry, markers, headings and identifiers from the same SDK operations the publisher uses.

**Non-Goals:**

- Rendering `:LOGBOOK:` drawers and other inline markup like Logseq, which [Issue #112 — Match Logseq's inline markup](https://github.com/codekiln/logseq-gardener/issues/112) covers.
- The reference-count badge on referenced blocks, part of [Issue #114 — Match Logseq's block references and block and page embeds](https://github.com/codekiln/logseq-gardener/issues/114).
- Logseq's outline look inside Linked References contexts, part of [Issue #115 — Match Logseq's linked references, filters, unlinked references and tagged pages](https://github.com/codekiln/logseq-gardener/issues/115).

## Decisions

**Where the work lives.** The SDK gains a public `outline` module:

- `Outline::from_blocks(&[Block])` reads the parser's flat bullet list for one page and returns its blocks in document order. Each `OutlineBlock` holds its depth (1 at the top level), the index of its parent, its `TaskMarker` (`Todo`, `Doing`, `Now`, `Later`, `Wait`, `Waiting`, `InProgress`, `Done`, `Canceled`, with `CANCELLED` read as `Canceled`), its priority letter, its heading level, whether it is authored collapsed, whether it has children, its source span, and its `BlockId`.
- The heading level is the `#` count, clamped to 1–6. `heading:: true` gives `min(depth + 1, 6)`, as Logseq's `block.cljs` does; `heading:: <n>` for 1–6 gives `n`.
- A block is authored collapsed when its attached properties have `collapsed:: true` and it has children. This moves the rule that `render.rs` applies today into the SDK without changing it.
- `BlockId` is `Uuid(String)` for a valid `id::` UUID (lower-cased, as `selected-block-navigation` normalizes it), otherwise `Path(Vec<u32>)`: the block's 1-based position among its siblings at each level, written `3.1.2`. `BlockId::parse` reads either form; `Outline::find(&BlockId)` returns the block; `Outline::ancestors(index)` returns the parents from the nearest up.

The publisher renders the markup, CSS, script, checkboxes and zoomed view. It reads collapse, heading level, marker and priority from `Outline`.

**Markup.** Each block's `li` gains a bullet element as its first child, `<a class="bullet" href="#" aria-label="Open block"></a>`, the same bytes for every block so it compresses to almost nothing. A block with children whose list is shown gets no extra markup; the existing `ul` carries the guide line. Root lists of a page and of a feed journal get `class="outline"`, so the new CSS leaves Linked References contexts and Markdown lists inside blocks unchanged. Authored collapsed blocks keep `<details class="outline-disclosure">`, with the bullet inside the `summary` before the content. Task blocks render `<input type="checkbox" disabled>` (checked for DONE) and `<span class="task-marker">TODO</span>` before the text; DONE and CANCELED put the text in `<span class="task-done">`. A priority renders `[#A]`, linked to page `A` when it is published. Headings keep their current `h1`–`h6` elements inside the `li`, now for `heading::` too. Page properties are followed directly by the outline; the empty paragraph a blank line produced is no longer written.

**CSS.** The page stylesheet gains, under `.outline`: no list markers, a 29px indent per level, a 2px left border in a new `--lsg-guide` colour on child lists, 2px block padding and a 1.6 line height, a 6px dot centred in a 14px column for `.bullet`, a hover disc with the dot scaled to 1.2, and a filled 14px disc for the bullet of a closed `details`. The native disclosure triangle is hidden. Headings use Logseq's live values (h1 32px/300, h2 24px/300, h3 19.2px/600, h4 16px/600, h5 13.28px/600, h6 12px/600, a 1px bottom rule under h1 and h2) and align with the bullet. The page title is 36px, weight 300, 24px below. Task labels are 85%, weight 650, opacity 0.7; struck-through text is opacity 0.7. Colours come from theme tokens so both themes match Logseq's dark and light values.

**Reader collapse.** `lsg-outline.js` adds one arrow element and moves it to the block row under the pointer when that block has children. Clicking the arrow, or the 4px strip over a child list's guide line, toggles the block: an authored `details` opens or closes, and any other `li` gets or loses `class="folded"`, which hides its child list and draws the closed bullet. Folding lives only in the reader's tab. Without JavaScript, authored collapsed blocks still open with their bullet row as the `summary`. `fragments.js` removes `folded` from every ancestor of a linked block, as it opens authored disclosures today.

**Block zoom.** A bullet click, or Enter on a focused bullet, sets the address to `#zoom/<id>`, where `<id>` is the block's UUID when its `li` holds a `block-<uuid>` anchor and otherwise its position path counted from the DOM, the same path `BlockId::Path` gives. On `hashchange` and on load with a `#zoom/` hash, the script:

- adds `data-zoomed` to `main`, which CSS uses to hide the title, page properties, the hierarchy section, Linked References and every outline item that is neither the target nor inside it;
- opens every `details` and removes `folded` inside the target, keeping their previous state to restore;
- writes a breadcrumb before the outline: a link to the page (no hash), then up to three nearest ancestors as text links to their own `#zoom/` addresses, with `⋯` after the page when more ancestors are hidden, separated by chevrons at opacity 0.5. Each ancestor's text is its content without child lists, cut to 80 characters.

Clearing the hash, Back, or the page link restores the page view. An identifier that matches no block leaves the page view and logs nothing. The same view works in a journal page. In the journals feed, a bullet opens the journal's own page at the block's `#zoom/` address.

**Loading.** `lsg-outline.js` is written once to the site root. `lsg-shell.js` adds it after the `load` event on every page, as it adds `lsg-journals.js`, so it is outside the 16 KB pre-title script budget. A page opened at a `#zoom/` address shows the whole page until the script runs; on the `tmux` page that is the first frame after `load`.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on the encode garden, desktop and phone profiles:

| Measure | Budget |
| --- | --- |
| `lsg-outline.js`, uncompressed | 8 KB |
| Outline CSS added to every page, gzip | 1.5 KB |
| `tmux` page, gzip | under 100 KB (existing budget), at most 2 KB above today |
| Data and script before the title is visible | existing budgets (100 KB, 16 KB) |
| Bullet click to zoomed view painted, `tmux` | 50 ms desktop, 150 ms phone |
| Peak JavaScript heap on `tmux` after zooming in and out | existing 10 MB budget |

`parity:budgets` in CI gains the script size check and the `tmux` size check on the fixture garden's largest page. `parity:perf --record` adds the zoom row and commits it to `docs/parity/performance.md`.

## Risks / Trade-offs

- [A position path changes when the author inserts a block above it, so a shared zoom link can point at a different block after an edit] → Blocks with `id::` use their UUID, which never moves; Logseq's own links break on every build, so the path is still more stable than the original.
- [The per-block bullet element pushes `tmux` towards its 100 KB limit] → The element is byte-identical for every block, and the `tmux` budget check fails the build if it adds more than 2 KB gzip.
- [The hidden native triangle removes a visual hint that a block opens] → The closed bullet's filled disc is Logseq's hint, and the `summary` keeps its keyboard and focus behaviour.
- [Gitpa replaces `<main>` content] → The script uses event delegation on `document` and reads the hash on each `hashchange`, so swapped content needs no setup.

## Resolved Questions

### 1 - What identifier does a zoom address use for a block without `id::`?

Decided by the agent without codekiln: the block's 1-based position among its siblings at each level, such as `3.1.2`. Logseq assigns a random UUID on every build, so its links never survive a rebuild; a position path survives every rebuild of unchanged content, needs no extra bytes in the page because the script counts it from the DOM, and the SDK computes the same value for the CLI and language server.

### 2 - Does block zoom get its own HTML file per block?

Decided by the agent without codekiln: no. The zoomed view hides the rest of the loaded page and builds the breadcrumb from the DOM. One file per block would multiply the site's file count by the block count for a view the reader reaches only from the page itself.

### 3 - Can readers tick task checkboxes?

Decided by the agent without codekiln: no; checkboxes are disabled. Logseq's published site lets a reader tick one, but the change is lost on reload and suggests the reader edited the page. The analysis recorded the same choice.

### 4 - Do Linked References contexts get Logseq's bullets in this change?

Decided by the agent without codekiln: no. Only root outlines marked `.outline` (a page and a feed journal) get the new look. #115 redesigns the Linked References section and its contexts, and styling them now would be redone there.

### 5 - Is reader collapse saved between visits?

Decided by the agent without codekiln: no. Logseq's published site keeps it only in the open tab, and saving it would need per-page browser storage for a state readers rarely expect to persist.

### 6 - What does `heading:: true` map to?

Decided by the agent without codekiln: `h` of the block's depth plus one, capped at 6, as Logseq's `block.cljs` computes it. No encode-garden page uses it, so the SDK unit tests are its only check.

### 7 - Do journal-feed articles zoom?

Decided by the agent without codekiln: no. Feed bullets get the look but open the journal's own page at the zoom address instead of zooming in the feed; the feed appends batches, and zooming there would have to restore a long, growing list.

### 8 - How large may `lsg-outline.js` be?

Decided by the agent without codekiln during the apply step: 8 KB uncompressed, up from the proposed 4 KB. The readable script with collapse, zoom, breadcrumb and restore is 6.9 KB; it loads after `load`, so it adds nothing before the title, and shrinking it by hand would make it harder to read than the repository's other scripts.

### 9 - Which pages load `lsg-outline.js`?

Decided by the agent without codekiln during the apply step: every page, after `load`. Gitpa swaps outlines into pages that started without one, and the browser caches the one file after its first use.

### 10 - What does the zoom fixture on `rw-cli` use as its address?

Decided by the agent without codekiln during the apply step: the position path `1.1.1.2`. The UUID `6add3ffb-e52b-4e8f-908a-9958186e4dfb` in the analysis is one Logseq generated for that build; it is not in `pages/rw-cli.md`, so no static build can reproduce it. Zooming `rw-cli` at `#zoom/1.1.1.2` shows the breadcrumb Logseq shows.

### 11 - What about `collapsed:: true` written after a block's children?

Decided by the agent without codekiln during the apply step: left for a follow-up. Two of `rw-cli`'s four collapsed blocks put the property after their first child, at the parent's indentation; Logseq attaches it to the parent, and the parser attaches it to the child, which has no children, so `lsg` shows those two blocks open. Fixing it means attaching properties by indentation in the parser boundary, which is wider than this change.

## Open Questions

None.
