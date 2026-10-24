## Context

The [app shell analysis](../../../docs/parity/app-shell.md) compared Logseq 0.10.9's published build of the encode garden with an `lsg` build on 2026-10-24. Logseq shows a 48px header, a 246px left sidebar with five items and 19 favorites, a More menu, and a 810px column in its indigo palette. `lsg` writes `<nav>{links}{Appearance selector}</nav><main>…</main>` with a 55rem column and its own palette (`html_document` in `crates/logseq-gardener-sdk/src/site/mod.rs`).

Several frame items lead to features that other issues under [Issue #106 — Match Logseq's published site](https://github.com/codekiln/logseq-gardener/issues/106) add: search, Graph view, the right sidebar, Journals, Flashcards and View as slides. This change builds the frame and the controls that have a destination today, and gives each later issue a place to add its control.

Two consumers depend on the current markup. Gitpa (Codex's work) inserts its own navigation and calls `window.logseqGardenerInitTheme(root)`. Sites with `--stylesheet` override the `--lsg-*` colour variables.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's header, left sidebar, favorites and More menu on every page, in Logseq's colours and widths, at desktop and phone widths.
- Every control on the page works; none leads to a missing file.
- The frame adds a fixed cost per page that does not grow with the garden.

**Non-Goals:**

- Drag-resizing the left sidebar between 240 and 460px, as Logseq allows. It is a convenience for long titles; the fixed 246px width covers the encode garden's favorites.
- Shipping Source Sans 3. Matching Logseq's typeface is part of the look, but it adds a font download before first paint; see Resolved Question 9.

## Decisions

**Where the work lives.** The SDK gains a public `garden_settings` module. `GardenSettings::read(root)` returns `favorites: Vec<String>` (raw `:favorites` entries in order) and `features: FeatureFlags` (`journals`, `flashcards`, `whiteboards`, each defaulting to `true`). It reuses the bounded EDN reader in `site/config.rs`, which moves next to it so both the site and the new module call one reader. A second operation, `resolve_favorites(settings, lookup)`, resolves each entry through the same case-insensitive title and alias lookup that page links use (the [selected-page-aliases](../../specs/selected-page-aliases/spec.md) rules) and returns, in order, the target route and original title for each resolved entry plus a diagnostic for each unresolved one. Duplicates that resolve to the same page keep the first position. The CLI and the future language server can call both. The publisher only renders the resolved list.

**Markup.** `html_document` writes, in order: `<header class="lsg-header">`, `<aside id="lsg-left-sidebar" class="lsg-left-sidebar">`, a `<div class="lsg-sidebar-shade">`, then `<main>`. Header buttons are native `<button>` elements with `aria-label`, `aria-controls` and `aria-expanded`; Home is a link to `index.html` with `aria-label="Home"`, omitted from `index.html` itself. The sidebar holds a `<nav aria-label="Navigation">` list with All pages, then a `FAVORITES` heading and a list of favorite links; the link for the current page gets `aria-current="page"` and the highlighted style. All pages points at `pages.html` when `--homepage` is set and at `index.html` otherwise. The More menu is a `<button aria-haspopup="menu">` with a `role="menu"` list of `role="menuitem"` buttons. Icons are inline SVG paths from Tabler Icons, the set Logseq uses, with the MIT notice recorded per [docs/licensing.md](../../../docs/licensing.md).

**Slots for later controls.** The header and sidebar render from an ordered list of items in the publisher. Each later issue adds its item to the list (Search after the sidebar toggle; Graph and Toggle right sidebar on the right; Journals, Flashcards and Graph view in the sidebar before All pages; View as slides at the top of More). An item appears only when its feature is published and, for the sidebar items, its `:feature/enable-*` flag is true.

**Sidebar state.** The open state is saved in `localStorage` under `logseq-gardener-left-sidebar` (`"open"` or absent). The inline theme snippet in `<head>` also reads this key and sets `data-lsg-left-sidebar="open"` on `<html>` before paint, so a page that opens with the sidebar open does not shift sideways after load. Below 640px the saved state is ignored on load and the overlay starts closed, as in Logseq, because an open overlay would cover the page a reader just opened. Choosing a link in the overlay closes it. Unavailable storage keeps the state for the current page only, matching the theme rules.

**Theme.** `t t` and Toggle theme switch between light and dark and save the choice under the existing key `logseq-gardener-theme`. When the current appearance comes from the system, the toggle saves the opposite of what the system shows. Use system theme appears in More only while a light or dark choice is saved and removes it. The default page no longer renders the Appearance `<select>`; `theme.js` still initialises any `[data-lsg-theme-select]` a host inserts, so Gitpa's control and `window.logseqGardenerInitTheme(root)` keep their behaviour.

**Colours and layout.** `theme.css` keeps the `--lsg-*` variable names, so custom stylesheets keep working, and takes Logseq's indigo values as defaults: background `#fbfcfd`/`#151718`, sidebar `#f8f9fa`/`#1a1d1e`, surface `#f1f3f5`/`#202425`, border `#e6e8eb`/`#2b2f31`, muted text `#687076`/`#9ba1a6`, text `#11181c`/`#ecedee`, link `#3451b2`/`#849dff`. New variables `--lsg-sidebar` and `--lsg-header` name the sidebar and header backgrounds. `--lsg-warning` and `--lsg-focus` keep values with at least 4.5:1 contrast on the new backgrounds; the apply step checks every pair. The main column is `max-width:810px` (1280px when `data-lsg-wide="true"`), with `padding-inline:16px` below 640px and `32px` from 640px. Body text uses `font-family:"Source Sans 3","Source Sans Pro",system-ui,sans-serif` at 16px with line height 1.5, Logseq's base size. The page title's size and weight belong to [Issue #111 — Match Logseq's page title, outline, headings, collapse and block zoom](https://github.com/codekiln/logseq-gardener/issues/111).

**Keyboard shortcuts.** One `keydown` listener in `lsg-shell.js` handles two-key sequences (`t l`, `t t`, `t w`, `g h`, `g a`) typed within one second, ignoring keys while focus is in an input, textarea, select or editable element, or while a modifier other than Shift is held. Escape closes the More menu and the overlay sidebar and returns focus to the button that opened it.

**Scripts and styles.** The shell's styles (about 3 KB) go inline with the existing page styles, because the header and sidebar must lay out before first paint. The shell's behaviour (sidebar, More menu, shortcuts, wide mode) is one file, `lsg-shell.js`, written once to the site root and loaded with `<script defer src="lsg-shell.js">`. The theme snippet stays inline in `<head>` because it must run before paint. The existing inline scripts (`fragments.js`, `backlinks.js`) stay as they are; moving them is a separate optimisation with its own risk for Gitpa's fragment swaps. Without JavaScript the header buttons and More menu stay hidden (as the Appearance selector does today), Home stays visible, and the sidebar stays closed.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on every fixture page, desktop and phone profiles:

| Measure | Budget |
| --- | --- |
| `lsg-shell.js`, uncompressed | 6 KB |
| Inline shell styles plus header and sidebar markup, per page | 6 KB uncompressed with the encode garden's 19 favorites |
| Script before the title is visible | 16 KB (existing budget; today 8.4 KB) |
| Data before the title is visible (gzip) | 100 KB (existing budget) |
| Peak JavaScript heap | 10 MB (existing budget) |
| Page size change when 2,000 unrelated pages are added | none (existing `parity:budgets` check) |

Search latency is unaffected; this change adds no search. `parity:budgets` in CI gains the `lsg-shell.js` size check, and the fixture garden's `config.edn` gains three favorites (one through an alias, one naming a missing page) so CI exercises the frame. `parity:perf --record` rewrites `docs/parity/performance.md` with the new rows.

## Risks / Trade-offs

- [A saved open sidebar shifts the main column on every load at desktop width] → The `<head>` snippet applies the state before paint.
- [Gitpa replaces `<main>` content and may duplicate or drop the header] → The header and sidebar sit outside `<main>`; `lsg-shell.js` binds its handlers once with event delegation on `document`, so swapped content needs no reinitialisation. The apply step checks a Gitpa build.
- [Favorites change the HTML of every page when `config.edn` changes] → Expected: the favorites list is part of the frame. Its size depends on the config, which the budget covers.
- [Logseq's fonts differ from the fallback] → Screenshots will show the type difference; the parity notes record it as explained.
- [Single-key shortcuts conflict with screen reader quick keys] → Shortcuts run only outside form fields and editable content, as in Logseq; screen readers in browse mode capture keys before the page sees them.

## Resolved Questions

### 1 - What do header controls without a destination do until their issues land?

Decided by the agent without codekiln: they are left out. Search, Graph and Toggle right sidebar are added by #118, #120 and #119 together with what they open. A disabled control or placeholder page would show readers a feature that does nothing, and the analysis requires that no control lead to a missing file.

### 2 - Which left sidebar items appear now?

Decided by the agent without codekiln: All pages, then the favorites. Journals, Flashcards and Graph view are added by #110, #123 and #120, each before All pages in Logseq's order and hidden when its `:feature/enable-*` key is `false`. Whiteboards is left out: `lsg` publishes no whiteboards, and the encode garden has no `whiteboards/` directory, so the item would open an empty list.

### 3 - What does All pages open before #117 adds Logseq's table?

Decided by the agent without codekiln: the existing page index, `pages.html` with `--homepage` and `index.html` without. #117 replaces the index with the table at the same address.

### 4 - What happens to a favorite that names no published page?

Decided by the agent without codekiln: it is left out of the list and reported as a diagnostic in the publish report. Logseq links it to an empty page; `lsg` publishes only selected pages, so a link would lead to a missing file.

### 5 - Which More menu items does `lsg` offer?

Decided by the agent without codekiln: Toggle theme and Use system theme now, View as slides with #121. Export page and Export graph are left out: exporting the graph as EDN or JSON would ship the whole graph, which the performance section of #106 rules out, and a reader can already save the static page.

### 6 - How does System appearance fit Logseq's single Toggle theme?

Decided by the agent without codekiln: Toggle theme switches light and dark as in Logseq; Use system theme appears in More while a light or dark choice is saved. The published-system-themes spec, the existing storage key and the starting System choice stay, and the menu shows only Logseq's item to a reader who has not chosen.

### 7 - Does the starting appearance follow the system or default to light as Logseq does?

Decided by the agent without codekiln: it follows the system. The published-system-themes spec already requires this, and a reader whose system is dark sees a dark site with no flash of light colours.

### 8 - Does the left sidebar start open, as in the reference prototype, or closed, as in Logseq's source and live build?

Decided by the agent without codekiln: closed, with the state saved across pages. The source and live build win over the boards, as the reference README says.

### 9 - Does `lsg` ship Source Sans 3?

Decided by the agent without codekiln: no; the font stack names Source Sans 3 first and falls back to the system font. A font file would add to the data loaded before first paint on every first visit and needs a licence notice. If codekiln wants the typeface, a subset WOFF2 file loaded with `font-display:swap` is the follow-up.

### 10 - Do the shell's styles and script stay inline?

Decided by the agent without codekiln: the styles stay inline because the header must lay out before first paint; the behaviour moves to one shared `lsg-shell.js` so its size is paid once per visit. The analysis asked for shared files; the existing inline scripts move only if a later change measures a gain.

### 11 - Where do favorites and feature settings come from?

Decided by the agent without codekiln: a public SDK operation that reads `config.edn` and resolves favorites through the title and alias lookup, so the CLI, the language server and the publisher give the same answer, as the shared-engine comment on #106 asks.

### 12 - Does this change set the page title's size and weight?

Decided by the agent without codekiln: no; #111 owns the page title. This change sets the frame, body type and widths.

## Open Questions

None.
