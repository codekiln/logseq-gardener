# Verification: mark-page-links-page-ref

Shift-click adds a right sidebar card for each kind of page link this change marks, on a Gitpa build whose pages are served at readable routes.

Checked on 2026-11-01 with a Gitpa preview built from [codekiln/gitpa `65d7bf6` build: install Logseq Gardener v0.0.4](https://github.com/codekiln/gitpa/commit/65d7bf69f) by `python3 scripts/build_site.py --renderer <this branch's lsg> --output …/gitpa`, served at `http://localhost:8793/gitpa/` and opened in a 1280×800 browser tab with an empty `logseq-gardener-right-sidebar` entry.

| Action | Result |
| --- | --- |
| On `/gitpa/GitP/A/Session/26/10/26-Mon/`, Shift-click the `prev` property value `GitP/A/Session/26/10/24-Sat` | Page card `GitP/A/Session/26/10/24-Sat`, route `GitP/A/Session/26/10/24-Sat/`; URL unchanged |
| On `/gitpa/Ableton/`, Shift-click the Pages tagged with row `Ableton/Arrangement View/Clip/How To/Play in a loop` | Page card with that page's content, route `Ableton/Arrangement%20View/Clip/How%20To/Play%20in%20a%20loop/`; URL unchanged |
| On `/gitpa/Logseq/Frontmatter/`, Shift-click the page embed header `Logseq/Frontmatter/tags` | Page card with that page's content, route `Logseq/Frontmatter/tags/`; URL unchanged |
| Plain click on the same embed header | Browser opens `/gitpa/Logseq/Frontmatter/tags/` |
| On `/gitpa/Microfreak/`, expand Linked References and Shift-click the first group title | A page card is added and the URL is unchanged, but the card reads "This page is not published" (see below) |

Each of these links renders with `class="page-ref"` in the preview. Before this change each was a bare `<a>` at a readable route, so Shift-click fell through to the browser and opened a new window, as [Issue #311](https://github.com/codekiln/logseq-gardener/issues/311) reports.

The group title's card is empty because of Gitpa, not this change: Gitpa rewrites `p-*.html` links only in top-level pages, so the `_refs/` fragment keeps `p-2b893bd4a4255811.html`, which resolves against `/gitpa/Microfreak/` once `lsg-references.js` moves it into the page. A plain click on the same title leads to a 404 on the live site too. [codekiln/gitpa#124 Links in Linked References lead to a 404 at readable routes](https://github.com/codekiln/gitpa/issues/124) tracked it, and [codekiln/gitpa#125](https://github.com/codekiln/gitpa/pull/125) fixed it: with both changes in the same preview, the group title's card showed its page and a plain click opened it.

Gitpa uses no build plugins, so plugin-derived rows were checked only by `tests/site_build_plugins.rs`, which also asserts that a derived link to a block stays a plain `<a>`.
