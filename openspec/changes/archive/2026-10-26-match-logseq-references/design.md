## Context

The [references analysis](../../../../docs/parity/references.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `c622a4d`) with an `lsg` build of garden commit `0ce74d6` from `lsg` `3f8e7c2`. Today `crates/logseq-gardener-sdk/src/site/backlinks.rs` builds the reference index inside the publisher, renders every group into the page, and `backlinks.js` filters them with one select per related page. On `mise` the section is 449 KB of a 511 KB page, and on `Answer/AI` it is 99% of 2.2 MB.

Logseq computes the same sections in the browser after downloading the whole graph (`db.gz`, 19.8 MB). `lsg` computes them at build time and sends a page with only the section headers and counts; the bodies arrive when a reader opens a section.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's four sections below the outline, with its counts, order, groups, breadcrumbs and filter dialog.
- The CLI and the language server get the same reference answers as the publisher from one SDK index.
- A page's first paint stays within the `[desktop]` and `[phone]` budgets whatever its reference counts, and opening a section stays within a phone memory budget.

**Non-Goals:**

- Block-level linked references (the count beside a referenced block and the references under a zoomed block), which belong with block zoom.
- Shift-click on a group title or crumb to open it in the right sidebar, which belongs with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).

## Decisions

**Where the work lives.** A new SDK module, `references`, owns every relationship; the publisher only renders.

| Operation | Answer |
| --- | --- |
| `ReferenceIndex::build(garden, selection)` | The index over selected visible documents, built from one contribution per document so a changed file replaces only its own contribution. |
| `linked(page) -> Vec<RefBlock>` | Blocks whose references include the page, an alias of it, or a namespace child of it, outside the page itself, deduplicated per block. Page properties count as one block. |
| `path_refs(block) -> PageSet` | The page identities of the block, its ancestors and its page, used for filter matching. |
| `related(blocks, exclude) -> Vec<(PageId, count)>` | Page identities referenced by the blocks, their ancestors or their children, with counts, most frequent first, ties by title. |
| `starting_filter(page) -> Filter` | The includes and excludes from the page's `filters::` value, keeping only names that resolve to published pages. |
| `tagged_with(page) -> Vec<PageId>` | Pages whose `tags::` name the page or an alias, sorted by lower-cased title. |
| `unlinked(page) -> Vec<RefBlock>` | Blocks whose text mentions the page title or an alias under Logseq's match rule, outside the page itself. |

Namespace-parent expansion happens when a document's contribution is built, from the name the author wrote: a reference to `A/B/C` adds `A/B` and `A`, even when `A/B/C` has no page, as Logseq's [`graph_parser/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L358-L367) does. Generated namespace pages receive these references too.

**Unlinked mentions in one pass.** The index builds one case-insensitive [Aho–Corasick](https://docs.rs/aho-corasick) automaton over every selected title and alias, then scans each block's text once with overlapping matches, with the logbook removed. A match counts when the character before it is not a letter, digit or `#` and the text before it is not `[[`, and the character after it is not a letter or digit, which is Logseq's rule from [`model.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1320-L1347). Matches inside another page's link text count, as in Logseq. One pass costs about the size of the garden's text; a scan per page would cost that times 6,990.

**Header and count.** The page's HTML holds each section's header and nothing else:

```html
<section class="references" data-refs="linked" data-src="_refs/p-…-linked-1.html">
  <details class="references-section">
    <summary><span class="ref-chevron"></span><h2>4 of 589 Linked References</h2>
      <a class="filter" title="Filter" data-state="include" href="…">…funnel svg…</a></summary>
    <noscript><a href="_refs/p-…-linked-1.html">Show linked references</a></noscript>
  </details>
</section>
```

The count uses blocks, before filters. With a starting filter the build computes `F` too, so the first paint already reads `4 of 589`.

**Fragments.** For each page the publisher writes, into one flat `_refs/` directory:

- `<page>-linked-<n>.html`: groups in order, about 50 groups per file. The first file also carries the filter data as a JSON script: a page-local table of related page titles and routes, and per block its fragment file, its path-reference indexes and, for a block that is not inside another referring block, its descendants' references.
- `<page>-unlinked-<n>.html`, the first of which carries the total. A page with no mentions has no file.

Opening a section fetches the first file; later files load as the reader scrolls near the end, like Logseq's `lazy-visible`. An active filter fetches only the files that hold matching groups. Each fragment is also a readable HTML page, with `<base href="../">` and a link to the next one, so the no-script link works. Fragments use the same block rendering, anchor and withholding rules as page content.

**Groups and breadcrumbs.**

```html
<div class="references-blocks-item">
  <div class="foldable-title"><span class="ref-chevron"></span><a href="…">2026-10-19 Mon</a></div>
  <div class="breadcrumb block-parents">…crumb › crumb › crumb…</div>
  <div class="blocks-container">…referring block and children…</div>
</div>
```

Journals come first, newest first, then other pages by lower-cased title; within a group, top-level blocks come first, then document order. A breadcrumb shows the nearest three ancestors, with `⋯` replacing the rest; each crumb is the ancestor's first line through the inline renderer and links to that ancestor's anchor on its page. The page itself is not a crumb. The existing de-duplication of overlapping outlines within a group stays.

**Filter dialog.** `references.js` replaces `backlinks.js` and loads after the page's `load` event through `lsg-shell.js`, like the outline script. Clicking the funnel opens a native `<dialog>` with the hint text, Includes and Excludes rows, a search input and chip buttons with superscript counts. A chip responds to click and Enter by including, to shift-click and Shift+Enter by excluding, and to a second activation by removing; each chip's accessible name states its current state. The funnel's `data-state` drives Logseq's colours. Chips and counts are recomputed from the blocks the current filter keeps. Filter changes are kept in `sessionStorage` under a key scoped to the site's base path and the page, so going back to a page within a visit restores them; when storage is unavailable, they last for the page view.

**Styles.** A new `site/references.css`, inlined with the other page styles, holds the measured light values from the analysis through theme tokens in `site/theme.css`. Dark values reuse the existing dark tokens and are checked against the `gh-pages` build in dark mode during Apply.

**Budgets.** Measured by the [comparison harness](../../../specs/parity-comparison-harness/spec.md) on the whole encode garden:

| Measure | Budget |
| --- | --- |
| First paint of `mise`, `Claude/Code`, `Answer/AI`, `note/my` | within the existing `[desktop]` and `[phone]` `data_before_title_kb` of 100 KB |
| `references.js`, minified | 6 KB, deferred, shared by every page |
| References styles on every page, gzip | 1 KB |
| Each fragment file, gzip | 100 KB |
| First group visible after opening a section | 300 ms desktop, 1,500 ms phone |
| Peak phone heap after opening Linked and Unlinked References on `mise` and scrolling to the end | 20 MB |
| Filter change on `mise`, desktop | 50 ms |
| Whole-garden build time | at most 25% above the build without this change |
| Search latency per keystroke | unchanged; references add nothing to the search corpus |

`scripts/parity/budgets.toml` gains a `[references]` table: `parity:budgets` checks the script, style and fragment sizes in CI, and `parity:perf` marks the timing, heap and build values in `docs/parity/performance.md`.

## Risks / Trade-offs

- [Short page names such as `AI` or `Go` match very often, so unlinked fragments could add a lot of output and build time] → The build-time and fragment budgets fail CI first; Apply records the ten largest unlinked totals and the total fragment size in `verification.md`.
- [Namespace-parent counting makes namespace roots' sections much larger] → Bodies leave the page, so first paint is unaffected; fragment files keep each fetch bounded.
- [Moving bodies out of the page makes linked references depend on a fetch when scripts run] → Each fragment is a working HTML page reachable by a plain link, and fragment navigation to a referring block still lands on the source page.
- [Dark values are not measured] → Apply checks them against the `gh-pages` build in dark mode before merging.

## Resolved Questions

### 1 - How are groups of non-journal pages ordered?

Decided by the agent without codekiln: by lower-cased title after the journals. Logseq uses database insertion order, which a build cannot reproduce; title order is stable and easy to scan.

### 2 - Do property names count as references and filter options?

Decided by the agent without codekiln: no. Logseq's file graphs turn property names such as `card-last-interval` into pages, so they appear as chips, but they are storage details with no published page, and a chip for them leads nowhere. Page-reference property values still count.

### 3 - How long does a reader's filter change last?

Decided by the agent without codekiln: for the visit, through `sessionStorage` scoped to the site and the page. Logseq's published build keeps changes in memory for the session and never writes them back to the file; a static site needs storage to carry them across page loads.

### 4 - Does the Unlinked References header show its count before it is opened?

Decided by the agent without codekiln: no. It reads `Unlinked References` until opened, then `N Unlinked References`, as in Logseq. The count arrives with the first fragment, so the page carries nothing extra.

### 5 - Where do the section bodies live?

Decided by the agent without codekiln: in fragment files fetched when a section opens. Logseq renders them in the browser from the whole graph; inlining them would keep `Answer/AI` at 2.2 MB, and the epic requires first paint independent of reference counts.

### 6 - What is the keyboard alternative to shift-click on a chip?

Decided by the agent without codekiln: Enter includes, Shift+Enter excludes, and activating an active chip again removes it, with each chip's accessible name stating its state. This mirrors the pointer gestures without new controls.

### 7 - Where does a breadcrumb crumb lead?

Decided by the agent without codekiln: to the ancestor's anchor on its page, where block zoom shows it on its own. Logseq opens `#/page/<uuid>`, which needs its client-side router.

### 8 - Does the zero-reference heading stay?

Decided by the agent without codekiln: no. A page with no linked references has no Linked References section, as in Logseq. This reverses the existing spec's zero-heading scenario and removes the heading from 3,529 pages.

### 9 - How are fragment files laid out?

Decided by the agent without codekiln: one flat `_refs/` directory with `<page>-linked-<n>.html` and `<page>-unlinked-<n>.html`, the filter data inside the first linked file, and no file for a page without mentions. The first layout (a directory and a JSON file per page) wrote 34,000 files in 10,700 directories, and creating them took most of a 14-second build; this layout writes 13,600 files and one request opens a section.

### 10 - Do names inside code count as unlinked mentions?

Decided by the agent without codekiln: yes, as in Logseq, which removes only the logbook before matching. With code removed `tmux` showed 609 unlinked references; with it kept `tmux` shows Logseq's 856.

### 11 - How does a copied block open its source?

Decided by the agent without codekiln: copied blocks keep their bullets, as Logseq's do, and a referring block's bullet links to the block's anchor on its page. The `↗` link beside each referring block is gone.

### 12 - When does `lsg-references.js` load?

Decided by the agent without codekiln: after the page's `load` event, through `lsg-shell.js`, as the outline script does. Inlined or deferred, it would count against the 16 KB first-paint script budget on search-enabled pages. A host that swaps content calls the same initializer as before.

### 13 - How are filter options counted?

Decided by the agent without codekiln: as Logseq's `reference.cljs` counts them. Each referring block whose parent is not itself a referring block adds its path references once and each descendant's references, repeats included; a referring block inside another adds nothing of its own. On `mise` this gives `mise/Config` 103 and `Gard` 66 against Logseq's 106 and 66.

### 14 - Does an alias reference name a namespace?

Decided by the agent without codekiln: no. Logseq expands only the name the author wrote, so `[[mise File Tasks]]`, an alias of `mise/Task/File`, does not refer to `mise`. Page properties resolved through the property view keep the expansion from their page titles, leaving out the page's own alias. With this rule `mise`, `tmux` and `McKinsey` match Logseq's block and group counts exactly.

## Open Questions

None.
