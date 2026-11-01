## Context

GitHub Pages serves a site of at most 1 GB. Publishing the encode garden writes 1,217 MB in 46,767 files. Logseq's own build of the same garden, on its `gh-pages` branch, is 228 MB in 506 files, because Logseq sends the whole graph as one database file and renders every page in the browser. `lsg` sends each page as static HTML, which is what makes its pages show their titles in tens of milliseconds where Logseq takes about 20 seconds ([lsg/Ref/Logseq Compatibility/Performance](https://github.com/codekiln/logseq-gardener/blob/main/docs/pages/lsg___Ref___Logseq%20Compatibility___Performance.md)). Static HTML takes space for every page, so this change stops repeating in each page what all pages share.

Measured on 2026-11-01 with `lsg` at `4573370` on encode garden commit `4d37b9ff`, Apple M2 Pro, with `--journals --search` and every top-level namespace included, as `parity:publish-bench` publishes it:

| Part | Files | Size |
| --- | ---: | ---: |
| Page HTML at the site root | 13,929 | 552 MB |
| `_refs/` Linked References fragments | 13,586 | 220 MB |
| `_refs/` Unlinked References fragments | 3,865 | 223 MB |
| `_refs/` Hierarchy fragments | 35 | 5 MB |
| `assets/` | 111 | 165 MB |
| Search files | 1,640 | 29 MB |
| `_graph/` | 13,528 | 19 MB |
| Everything else | 73 | 4 MB |

Inside the 552 MB of page HTML:

- 334 MB is the same two `<style>` blocks in every page's head: 18 KB of page styles and 5.8 KB of search styles.
- 64 MB is inline scripts: the theme script (3 KB a page, 41 MB in all), the fragment-reveal script at the end of the body (11 MB) and the search start-up (9 MB).
- 68 MB is the header and left sidebar, and 81 MB is the page content inside `<main>`.

Inside the 443 MB of Linked and Unlinked References fragments, 336 MB is copies of referring blocks. Only 114 MB of those copies are distinct: a block that links to, or mentions, several pages is copied into each page's fragments. The fragments are HTML and compress well: stored gzip-compressed, the same files take 80 MB.

`assets/` already holds only the files that rendered content links to, so its 165 MB is the garden's own media.

Moving the repeated styles and most inline scripts into shared files and compressing the Linked and Unlinked References fragments brings the encode garden to about 475 MB: 173 MB of page HTML, 80 MB of compressed fragments, 5 MB of Hierarchy fragments and 216 MB for everything else. This projection was computed by applying those three changes to the measured site.

[Issue #349 · Rewrite only changed pages when one block changes](https://github.com/codekiln/logseq-gardener/issues/349) asks that a one-block edit change only the files whose content changes. Today every page changes after any edit, because every page names the content-hashed search manifest. This change adds files that every page names, so their names must not undo #349's work.

## Goals / Non-Goals

**Goals:**

- Publish the encode garden in under 550 MB with no post-processing, so it fits GitHub Pages with room to grow.
- Keep every page's look and behaviour as Logseq's published site shows it, and keep the existing first-paint, search and memory budgets.
- Check the site size in CI on the pinned encode garden commit, beside the publication-speed rows.
- Name shared files so that a garden edit never changes them.

**Non-Goals:**

- Store each referring block's copy once across all reference fragments. After compression this would save an estimated 50 MB more, at the cost of one request per source page when a section opens.
- Shrink the header and left sidebar each page carries, about 5 KB a page and 68 MB on the encode garden.
- Rewrite only the files an edit changes, which [Issue #349](https://github.com/codekiln/logseq-gardener/issues/349) covers.

## Decisions

### Each inline style block becomes a shared file at the same place in the head

Every page's head carries one large `<style>` block of built-in styles, then the optional `custom.css` link, then, depending on the page, the search styles, the flashcard widget styles and build plugins' derived-row styles. The publisher writes each of these blocks once, as a file at the site root, and links it from the position the inline block held:

| Inline block today | Shared file | Pages that link it |
| --- | --- | --- |
| Built-in page styles: base, theme, navigation, outline, inline markup, code, transclusion, references and shell | `lsg-site.css` | every page |
| Search styles, `data-lsg-search-styles` | `lsg-search.css` | every page, when search is published |
| Flashcard widget styles | `lsg-flashcards-widget.css` | pages with a flashcard widget |
| Derived-row styles | `lsg-derived.css` | pages with derived rows |

Keeping each block's position keeps the order in which the styles apply. A garden stylesheet given with `--stylesheet` still comes after the built-in page styles and before the search styles, so it overrides and is overridden by the same rules as today.

Pages in folders, published with `--readable-urls`, already set `<base href>` to the site root, so the links use the same root-relative form as `lsg-shell.js`.

Alternatives considered:

- One stylesheet holding every block. It would save a request on pages with flashcards or derived rows, but `custom.css` would then come after the search styles, changing which rule wins where both style the same element.

### Only the scripts that must run before paint stay inline

The theme script does two jobs. Before the page paints, it applies the saved left sidebar, wide mode, right sidebar width and theme, so the page does not flash in the wrong layout. After that, it wires the theme menu items and exposes `window.logseqGardenerInitTheme`. The first job stays inline, minified; the second moves to `lsg-theme.js`, loaded with `defer`.

The fragment-reveal script stays inline at the end of the body, minified. It opens the collapsed blocks around the element a link's fragment names before the page paints, so a deferred file would let the page paint once with the target hidden.

The search start-up moves to `lsg-search-boot.js`, loaded with `defer` from the place the inline script held. Deferred scripts run before `DOMContentLoaded`, so the file registers the same `DOMContentLoaded` handler and loads the search controller after the second animation frame, as today. It still finds the manifest through the page's `<link rel="lsg-search">`.

The inline scripts `lsg` writes on every page then total at most 1.5 KB. Start-up scripts that only some pages carry, for dialogue timestamps and page properties, are unchanged.

### Linked and Unlinked References fragments are stored compressed

The publisher writes each Linked and Unlinked References fragment gzip-compressed, at level 6, with no file name or time in the gzip header, so the same content always produces the same file. Fragments are compressed on several threads during the write phase. File names keep their current form with `.html.gz` added: `_refs/<page>-linked-<n>.html.gz` and `_refs/<page>-unlinked-<n>.html.gz`.

A compressed fragment holds the section data and the groups. The document wrapper that made each fragment a readable page, its head, heading and link to the next file, is dropped, since a reader without scripts can no longer open the file.

`lsg-references.js` fetches a fragment as binary data. When the data starts with the gzip header, it decompresses it with `DecompressionStream`, as the graph view does with `_graph/default.bin`; otherwise a host has already decoded it, and the script reads it as text. It then parses the HTML as today. The right sidebar loads references through the same code.

Hierarchy fragments stay readable HTML pages. They total 5 MB, and they keep the Hierarchy section's script-free fallback.

Alternatives considered:

- Store each referring block's copy once, in a file per source page, with fragments naming the copies they show. Uncompressed, it reduces 336 MB of copies to 114 MB; compressed, it would save an estimated 50 MB more than compressing the current fragments. Opening a section would then fetch one file per source page, up to 50 for a full fragment, and the largest source page's copies are 2.5 MB. Compression alone meets the size budget with one request per fragment.
- Compute Unlinked References in the browser from the search index. The search records hold plain text, so the results would not render blocks as Logseq does.
- Name the files `.bin`, as the graph view's data files are named. `.html.gz` says what the file holds and opens with `gunzip`; the gzip-header check handles hosts that decode `.gz` files on the way.

### Files every page names have fixed names

Every file that every page names, `lsg-site.css`, `lsg-search.css`, `lsg-theme.js` and `lsg-search-boot.js`, has a fixed name, as `lsg-shell.js` and `lsg-references.js` already have. Their contents depend only on the `lsg` version, never on the garden, so a garden edit leaves them and every page's link to them unchanged. An `lsg` upgrade that changes only styles rewrites one file and no pages.

GitHub Pages lets browsers cache files for 10 minutes. For that long after an `lsg` upgrade, a reader can see a new page with an old cached stylesheet, as can already happen with `lsg-shell.js`.

Fragment names stay derived from the target page and position, and identical content compresses to an identical file, so a fragment changes only when its groups change. Together with [Issue #349](https://github.com/codekiln/logseq-gardener/issues/349)'s stable search manifest, a one-block edit can then change only the pages whose content changes and the fragments that list the edited block.

Alternative considered:

- Name shared files by a digest of their contents, as `lsg-search-ui-<digest>.mjs` is named. The name would change on every upgrade that touches the file, so every page would change with it.

### The publisher packages; the SDK's answers are unchanged

The SDK's reference index still decides which blocks refer to and mention each page, their order and their filters; the page catalog, search records and graph are unchanged. Everything in this change happens in the publisher in `crates/logseq-gardener-sdk/src/site/`, after those answers exist: where styles and scripts are written, and how fragments are stored. The public SDK API and `lsg publish`'s options and reports are unchanged.

### Budgets

The encode garden at the commit CI pins:

| Measure | Budget | Before | Projected |
| --- | ---: | ---: | ---: |
| Whole site | 550 MB | 1,217 MB | 475 MB |
| Page HTML at the site root | 200 MB | 552 MB | 173 MB |
| `_refs/` | 100 MB | 448 MB | 85 MB |

Logseq's `gh-pages` build of the same garden is 228 MB. The budgets leave room for the garden to grow by about 15% before CI flags it, and the 1 GB limit is far beyond that.

Each page, on the parity fixture garden:

- No inline `<style>` element written by `lsg`.
- At most 1.5 KB of inline script written by `lsg` on every page.
- Each `_refs/` fragment at most 100 KB as stored, the current gzip budget for fragments.

Reading pages, measured by `parity:perf` on the fixture pages, in both builds:

- First visit: at most 100 KB gzip of data and 16 KB of script before the title, and the title visible within 300 ms on desktop and 1,500 ms on the phone profile. These are the current budgets; the shared files now count toward them.
- A second page in the same visit loads no stylesheet or script from the network before its title.
- Search: warm keystrokes at most 50 ms p95 on desktop and 100 ms on the phone profile; a keystroke that fetches a shard at most 150 ms and 1,000 ms. These are unchanged.
- Peak JS heap on the phone profile: at most 10 MB after a page loads, and at most 20 MB after opening both reference sections on `mise` and scrolling to the end, decompression included. These are unchanged.
- Opening a reference section: first group within 300 ms on desktop and 1,500 ms on the phone profile, unchanged.

Building: a whole encode garden build at most 10% slower than the build before this change. Compressing 443 MB costs about 6 seconds of processor time at level 6, spread across threads, and the build writes about 360 MB less.

### Where the measurements run

`parity:publish-bench` already publishes the pinned encode garden commit in CI and records the output's file count and size. Its rows gain the file count and size of each part of the site: page HTML, `_refs/`, `assets/`, search files, `_graph/` and everything else. `--check` fails when the encode garden's site, page HTML or `_refs/` exceeds its budget in the new `[site]` table of `scripts/parity/budgets.toml`. The check uses the size of each run's output, so it needs no recorded row from the same processor.

`parity:budgets` checks the per-page budgets on the fixture garden in CI. `parity:perf` measures the reading budgets for both builds, and `parity:compare` with `parity:perf` runs once on the whole encode garden against Logseq's `gh-pages` build for the pull request, recording both sites' sizes on the Performance page.

## Risks / Trade-offs

- [A page paints before the stylesheet arrives on a slow first visit] → A linked stylesheet in the head blocks rendering, as the inline styles did, so the page never paints unstyled. The first visit costs one more request; `parity:perf` measures title time on the phone profile against the unchanged budget.
- [A host decodes `.gz` files and sends plain HTML] → The script checks for the gzip header before decompressing.
- [A browser lacks `DecompressionStream`] → The graph view already requires it. In such a browser the opened reference section says its references could not be loaded, and the page itself is unaffected.
- [Search engines no longer index `_refs/` pages] → Those pages repeated text from the pages that hold it. Every block remains indexed on its own page.
- [A reader without scripts loses the reference fragment link] → Pages, the Hierarchy section and All pages keep their script-free fallbacks. Logseq's published site shows nothing without scripts.
- [Pages and shared files mismatch for up to 10 minutes after an `lsg` upgrade] → The same holds for `lsg-shell.js` today. Content-hashed names would change every page on every such upgrade.

## Migration Plan

The change ships in an `lsg` release. A garden that republishes with it gets the new files; old `_refs/*.html` addresses stop resolving, and nothing outside the site links to them. After the encode garden publishes with that release, a pull request in logseq-encode-garden removes the `share_repeated_styles` step from `mise-tasks/site/build`. Rolling back means publishing with the previous release.

## Resolved Questions

### 1 - Which parts of the site should shrink?

Decided by the agent without codekiln: the repeated head styles, the repeated inline scripts and the Linked and Unlinked References fragments. Together they are 840 MB of the 1,217 MB. `assets/` holds only linked media, and search files and `_graph/` together are 48 MB.

### 2 - Should the inline style blocks become one shared stylesheet or one file each?

Decided by the agent without codekiln: one file each, linked where the inline block was. A garden's `custom.css` sits between the built-in page styles and the search styles today, so merging them would change which rule wins on some elements and could change the look.

### 3 - Should shared files that every page names be content-hashed or keep fixed names?

Decided by the agent without codekiln: fixed names, with contents that depend only on the `lsg` version. A content-hashed name would change every page whenever the file changes, which works against [Issue #349](https://github.com/codekiln/logseq-gardener/issues/349). With fixed names, a garden edit never touches them, and an upgrade that only changes styles rewrites one file. The cost is up to 10 minutes of stale cache after an upgrade on GitHub Pages, which `lsg-shell.js` already accepts.

### 4 - Which scripts stay inline?

Decided by the agent without codekiln: the part of the theme script that applies the saved layout and theme, and the fragment-reveal script. Both must run before the page paints, or the page would paint once in the wrong layout or with the linked block hidden. The theme controller and the search start-up run after paint, so they move to deferred shared files.

### 5 - Should reference fragments be compressed, or should each block's copy be stored once?

Decided by the agent without codekiln: compressed. Compression takes the fragments from 443 MB to 80 MB and keeps one request per fragment. Storing each copy once saves an estimated 50 MB more after compression but needs up to 50 requests when a section opens, which risks the 1,500 ms phone budget for the first group.

### 6 - How should compressed fragments be named and read?

Decided by the agent without codekiln: `.html.gz`, read as binary and decompressed with `DecompressionStream` when the data starts with the gzip header. The name says what the file holds, and the header check works on hosts that decode `.gz` files themselves.

### 7 - Should reference sections keep their script-free fallback?

Decided by the agent without codekiln: no. A compressed file cannot be read as a page, and keeping an uncompressed copy of the first fragment would keep most of the size, since the median section has one fragment. Logseq's site needs scripts for everything, the page content stays readable without them, and the fragment pages repeated text that search engines already find on each block's own page.

### 8 - Should Hierarchy fragments, page graph neighbourhood files and search files be compressed too?

Decided by the agent without codekiln: no. Together they are 53 MB, the budget is met without them, and Hierarchy fragments keep their script-free fallback. Search files belong to [Issue #349](https://github.com/codekiln/logseq-gardener/issues/349)'s renaming work.

### 9 - Which compression level, and does it slow the build?

Decided by the agent without codekiln: gzip level 6 on several threads. Level 6 compresses the fragments to 80 MB in about 6 seconds of processor time; level 1 is about 2.5 times faster and 20% larger. Spread across threads and offset by writing 360 MB less, the build should stay within 10% of today's, which the benchmark checks.

### 10 - What should the size budgets be?

Decided by the agent without codekiln: 550 MB for the whole encode garden site, 200 MB for page HTML and 100 MB for `_refs/`. The projection is 475 MB, so the garden can grow by about 15% before CI flags it, and the site stays far below GitHub Pages' 1 GB. Part budgets point a failure at the part that grew.

### 11 - Where should the size checks run?

Decided by the agent without codekiln: the site-size check in `parity:publish-bench --check`, which already publishes the pinned encode garden commit in CI and records the output size; the per-page checks in `parity:budgets` on the fixture garden. Size does not depend on the processor, so the check compares each run with the budget directly.

### 12 - Should the header and left sidebar move out of each page?

Decided by the agent without codekiln: no. They are part of the first paint, and loading them from a shared file would show the page without its frame first. At 68 MB they are not needed to meet the budget.

### 13 - What belongs in the SDK and what in the publisher?

Decided by the agent without codekiln: everything in this change belongs to the publisher. The SDK's reference index, page catalog, search records and graph give the same answers; the publisher decides where styles and scripts are written and how fragments are stored. No SDK API changes.

## Open Questions

None.
