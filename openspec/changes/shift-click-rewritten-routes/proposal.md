## Why

Shift-click on a page link opens a new browser window on the Gitpa site instead of adding a card to the right sidebar ([Issue #292 — Open page links in the right sidebar on Shift-click when a host rewrites page addresses](https://github.com/codekiln/logseq-gardener/issues/292)). Gitpa rewrites Gardener's `p-*.html` page addresses into readable routes such as `/gitpa/GitP/A/Session/26/10/29-Thu/` after the build, and the right sidebar recognizes a page link only by a `p-*.html` address.

## What Changes

- Shift-click recognizes a page link by its markup as well as by its address: a link inside `.page-reference`, an `a.page-ref` (query results, All pages, reference titles) or an `a.tag`. A block reference (`a.block-ref-link`) is recognized by its class at any address. Links to `p-*.html` keep working as they do now.
- A card's route is the link's address as the browser resolves it, relative to the site root, so a card for a rewritten route loads, survives a reload from any page and opens with Open as page.
- A block card for a bullet on the current page takes its route from the current address the same way, so a page served at a readable route no longer stores `index.html`.
- Links the sidebar writes into cards (breadcrumb crumbs and the Slides card title) point at the card's page from the site root, so they work from a page at a nested address.
- Shift-click on a link inside a flashcard block (`li.lsg-card`) adds its card instead of stopping on an error, because the sidebar finds its own cards by `section.lsg-card`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-right-sidebar`: Shift-click recognizes page links by their markup, and cards keep the route of the address a host serves the page at.

## Impact

- `crates/logseq-gardener-sdk/src/site/shell.js`: the Shift-click check lists the page-link markup and block references.
- `crates/logseq-gardener-sdk/src/site/sidebar.js`: `target()` recognizes page links by markup and stores routes relative to the site root; breadcrumb, Slides title, Open as page and zoom addresses resolve from the site root.
- Tests: `tests/shell.test.cjs` and `tests/sidebar.test.cjs` cover a host that serves pages at readable routes under `/gitpa/`; `tests/mini-dom.cjs` learns the `*=` attribute selector.
- Docs: `docs/local-site.md` tells host authors which links keep Shift-click after they rewrite page addresses.
- Budgets: both scripts stay within the `[shell]` and `[sidebar]` budgets in `scripts/parity/budgets.toml`.
- Gardener's own sites keep the same saved routes (`p-*.html`, `index.html`), so saved sidebar state stays valid.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
