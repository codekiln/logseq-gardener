# Verification

Measured on 2026-10-24 against the `gh-pages` build of garden commit `0ce74d6`, with an `lsg` build of the whole encode garden (`mise run parity:build`, 1,042 roots, 6,984 pages) from this branch. Both builds were served locally and opened in Playwright's Chromium on the `tmux` page with the left sidebar opened by `t l`.

## Frame

| Check | Logseq | lsg |
| --- | --- | --- |
| Header height, 1280px | 48px | 48px |
| Header height, 390px | 42px | 48px |
| Favorites listed | 19 | 19, in `:favorites` order, `github/cli/extension/gh-dash` shown as `GitHub/CLI/Extension/gh-dash` |
| Sidebar width, 1280px | 246px | 246px |
| Column shift when the sidebar opens, 1280px | 123px (235 to 358, from the analysis) | 123px (203 to 326) |
| Overlay width, 390px | 289px (74vw) | 289px (74vw) |
| Page width at 390px | 394px (4px sideways scroll) | 390px |
| Main column, 1280px | 810px content | 810px content plus 32px padding each side |
| Light background and text | `#fbfcfd`, `#11181c` | `rgb(251, 252, 253)`, `rgb(17, 24, 28)` |
| Dark background and text | `#151718`, `#ecedee` | `rgb(21, 23, 24)`, `rgb(236, 237, 238)` |

Screenshots in `verification/`: `desktop-light-logseq.jpg` and `desktop-light-lsg.jpg` side by side, `desktop-dark-lsg.jpg`, and `phone-light-logseq.jpg` and `phone-light-lsg.jpg` with the overlay open.

Explained differences:

- Logseq's sidebar also shows Journals, Whiteboards and Flashcards, and its header Search and Toggle right sidebar. They arrive with #110, #123, #118 and #119 (Resolved Questions 1 and 2).
- Logseq draws a page icon beside each favorite and a star beside FAVORITES; `lsg` shows the titles only, which keeps the per-page frame small.
- Logseq's header is 42px tall below 640px; `lsg` keeps 48px as the spec states.
- Body type falls back to the system font where Source Sans 3 is not installed (Resolved Question 9).
- Page content (title size, bullets, property blocks) differs as before; #111 to #115 cover it.

## Size and performance

`mise run parity:budgets` passes: `lsg-shell.js` is 4,572 characters (budget 6 KB), fixture pages stay within the first-paint and script budgets, and no page or the shell script changes size when 2,000 unrelated pages are added. On the encode garden the header and sidebar markup is 2.9 KB per page with 19 favorites (budget 6 KB).

`mise run parity:perf --builds lsg` rewrote the `lsg` rows in `docs/parity/performance.md`. Data before the title grew by about 3 KB gzip per page and script before the title from 8.4 KB to 13.5 KB, because the browser fetches the deferred `lsg-shell.js` before the title shows. Every fixture page stays within budget; time to the title is unchanged within run-to-run variation (for example `tmux` desktop 34.7 ms, phone 236 ms).

## Behaviour

- Rust tests (`crates/logseq-gardener-sdk/tests/site_shell.rs`, `garden_settings` unit tests) cover header order, Home on `index.html`, All pages with and without `--homepage`, favorites order, alias labels, duplicates, private and missing favorites, current-page marks, hidden controls without JavaScript and the shared script.
- Node tests (`tests/shell.test.cjs`, `tests/theme.test.cjs`) cover the sidebar toggle and saved state, the phone overlay closing by shade, link and Escape, the More menu's keyboard behaviour, Toggle theme, Use system theme, `t l`, `t t`, `t w`, `g h`, `g a`, field and modifier exclusions, and blocked storage.

## Gitpa

Gitpa's `site/navigation.js` replaces the first `<nav>` and the `<main>` of the page with those of the next page, then calls `window.logseqGardenerInitTheme(importedNav)`. The first `<nav>` is now the sidebar's navigation, so the swap updates the favorites' current-page mark; the header stays in place, and its controls keep working because `lsg-shell.js` handles events on `document`. Gitpa's own Appearance selector keeps working through `logseqGardenerInitTheme`. After a swap the header keeps the Home button of the first page, which is visible on Gitpa's home page; this was read from the code, and a Gitpa build was not run in this change.
