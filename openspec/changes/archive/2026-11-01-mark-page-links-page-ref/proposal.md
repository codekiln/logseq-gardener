## Why

Shift-click on some page links still opens a new browser window on the Gitpa site instead of adding a card to the right sidebar ([Issue #311 — Mark every page link with page-ref so Shift-click opens it in the right sidebar](https://github.com/codekiln/logseq-gardener/issues/311)). Since [#297](https://github.com/codekiln/logseq-gardener/pull/297) the sidebar recognizes a page link at any address by its markup, but Gardener still renders five kinds of page link as a bare `<a>`. Gitpa rewrites `p-*.html` addresses into readable routes, so those links match neither the markup nor the address. In Logseq every page link is drawn by `page-cp`, which renders `a.page-ref`.

## What Changes

- Page-valued text properties, including `prev::`, `next::` and `tags::`, render their page links with the class `page-ref`.
- Each row of Pages tagged with renders with `page-ref`.
- The page title of each Linked and Unlinked References group renders with `page-ref`, as query result groups already do.
- A page embed's header renders its page link with `page-ref`.
- A plugin-derived row renders a link to a whole page with `page-ref`. A link to a block of a page stays a plain `<a>`, as breadcrumb crumbs do.

None of these links has a style keyed on `page-ref`, so they look the same, and a plain click still navigates.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-right-sidebar`: Gardener marks every page link it renders with page-link markup, so Shift-click adds a card for it on a host that rewrites page addresses.

## Impact

- `crates/logseq-gardener-sdk/src/site/properties.rs`, `references.rs`, `render.rs` and `derived.rs`: the five links gain `class="page-ref"`.
- Tests: `tests/site_properties.rs`, `tests/site_references.rs`, `tests/site_embeds.rs` and `tests/site_build_plugins.rs` assert the class on each kind.
- Docs: `docs/local-site.md` lists the page links Gardener marks, so a host author knows only links the host writes itself need the markup.
- Scripts: no change. `lsg-shell.js` and `lsg-sidebar.js` already treat `a.page-ref` as a page link.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
