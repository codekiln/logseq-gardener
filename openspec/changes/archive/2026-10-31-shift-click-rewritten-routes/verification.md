# Verification: shift-click-rewritten-routes

Shift-click adds right sidebar cards on a Gitpa build whose pages are served at readable routes, and the cards keep working after navigation.

Checked on 2026-10-31 with a Gitpa preview built from [codekiln/gitpa `69a3a54` build: install released Logseq Gardener v0.0.1](https://github.com/codekiln/gitpa/commit/69a3a5400) by `python3 scripts/build_site.py --renderer <this branch's lsg> --output …/gitpa`, served at `http://127.0.0.1:8792/gitpa/` and opened in a 1024×768 browser tab with an empty `logseq-gardener-right-sidebar` entry.

| Action | Result |
| --- | --- |
| On `/gitpa/GitP/A/Session/26/10/29-Thu/`, Shift-click the inline page reference `Microfreak/Preset/016 Voidom` | Page card opens; route `Microfreak/Preset/016%20Voidom/`; URL unchanged |
| Shift-click the `00:00 enter` bullet on the same page | Block card `GitP/A/Session/26/10/29-Thu › Hallowed Speaking › Chapters`; route `GitP/A/Session/26/10/29-Thu/`, block `1.4.1` |
| Open `/gitpa/Ableton/Clip/How To/Export Multiple Clips to Wav Files/` | Both cards restored with their content |
| Shift-click the tag `#Ableton` | Page card `Ableton`, route `Ableton/` |
| On All pages (`/gitpa/pages-26.html`), Shift-click the `a.page-ref` row `GitP/A/Session/24/11/19-Tue` | Page card with the session's content, route `GitP/A/Session/24/11/19-Tue/` |
| Card menu, Open as page | Browser opens `/gitpa/GitP/A/Session/24/11/19-Tue/` |
| From that page, read the block card's breadcrumb | Links point at `/gitpa/GitP/A/Session/26/10/29-Thu/`, `…/#zoom/1` and `…/#zoom/1.4` |
| Shift-click the `Chapters` crumb | Block card for block `1.4` of `GitP/A/Session/26/10/29-Thu/` |
| Plain click on the `GitP` page reference | Browser navigates to `/gitpa/GitP/` |

Before this change, `lsg-shell.js` intercepted Shift-click only for an `href` starting with `p-`, so each Shift-click above on a rewritten link fell through to the browser, as [Issue #292](https://github.com/codekiln/logseq-gardener/issues/292) reports from the live site.

Not covered by this change, seen in the same preview:

- Gitpa's homepage list of recent sessions is plain `<a>` rows that Gitpa writes itself, so Shift-click on them still opens a new window. Adding `class="page-ref"` to those rows in Gitpa's `install_homepage_query` would let the sidebar recognize them.
- Page-valued text properties such as `prev::` and `next::` render as plain `<a>` in `.page-property`, so on a rewriting host they also still open a new window.

`tests/shell.test.cjs` and `tests/sidebar.test.cjs` cover the same forms with a fixture host at `/gitpa/`; the new tests fail against the previous scripts. `lsg-shell.js` is 6,506 bytes after minifying (budget 6,900) and `lsg-sidebar.js` about 6.2 KB gzip (budget 6.5).
