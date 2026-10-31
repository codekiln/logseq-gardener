## 1. Shell and sidebar scripts

- [x] 1.1 In `shell.js`, intercept Shift-click on `.page-reference a`, `a.page-ref`, `a.tag`, `a.block-ref-link` and any address holding `#zoom/` inside the main column or the right sidebar, beside the existing bullet and `p-*` targets
- [x] 1.2 In `sidebar.js`, add a helper that turns an address into a route relative to the site root and one that turns a route back into an address; use them for the current page, fetches, breadcrumb crumbs, the Slides title, Open as page and zoom, and mark the breadcrumb's page link and the Slides title with `page-ref`
- [x] 1.3 In `target()`, recognize page links by markup or `p-*.html` address, take routes from the resolved address, and find cards with `section.lsg-card`

## 2. Tests

- [x] 2.1 In `tests/shell.test.cjs`, check that Shift-click on each page-link form at a `/gitpa/` address reaches the sidebar, and that plain clicks, other links and links outside the main column do not
- [x] 2.2 In `tests/sidebar.test.cjs`, cover a host serving `/gitpa/` readable routes: page cards from a page reference, a query result and a tag; a block card from a bullet; a Block references card; Open as page and crumb addresses; a link inside a flashcard block; and restoring the cards on another page

## 3. Verification

- [x] 3.1 Build a Gitpa preview with this branch's `lsg` through Gitpa's `scripts/build_site.py`, serve it, and check Shift-click and plain click in a browser; record the result in `verification.md`
- [x] 3.2 Run `mise run ci`
