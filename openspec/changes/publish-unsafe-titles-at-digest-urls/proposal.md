## Why

`lsg publish --readable-urls` stops the whole encode garden build on three pages whose titles hold a literal slash, stored as `%2F` in the file name ([Issue #363 — Publish pages with unsafe titles at their digest URL under --readable-urls](https://github.com/codekiln/logseq-gardener/issues/363)). The slash splits `Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?` into a segment that ends with a space, `If I rename a CLI session with `, and the requirement "Safe and distinct page paths" makes any such title fail the build. The garden cannot leave the pages out either, because `--exclude` refuses the same titles. This blocks [codekiln/logseq-encode-garden#295 — Serve pages at their Logseq namespace URL](https://github.com/codekiln/logseq-encode-garden/issues/295).

## What Changes

- With `--readable-urls`, a page or generated namespace page whose title cannot form a safe path keeps its `p-<digest>.html` file at the output root. Every link, search result, sidebar card, graph node and `lsg-site.json` entry uses that file. Every other page keeps its folder.
- The build reports one diagnostic for each such page, naming the title, the problem and the file, and exits 0.
- The collision check compares only pages that get folders.
- The readable variants of the graph and search scripts accept a `p-<digest>.html` route next to folder routes.
- Path collisions and reserved root names still stop the build before output creation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readable-page-urls`: "Readable page layout" adds the digest file for a title that cannot form a safe path; "Safe and distinct page paths" turns the unsafe-title failure into that fallback and a diagnostic; "Readable routes in search, sidebar and graph" and "Plugin host navigation in readable builds" accept the page's digest file as its route.

## Impact

- `crates/logseq-gardener-sdk/src/publication.rs`: `route_for` falls back to the digest route and returns the reason; `select_with_urls` records the notice and checks collisions among folder routes only.
- `crates/logseq-gardener-sdk/src/site/readable-graph.js` and `readable.rs`: the readable graph script computes the digest file for a title that cannot form a safe path and finds the current page from it; the readable search engine accepts both route kinds.
- Tests: `crates/logseq-gardener-sdk/tests/site_readable.rs`, `tests/publish.rs` and `scripts/readable/browser.py`.
- Docs and help: the Readable URLs section of lsg/Ref/Publish Options, the `lsg publish help` behavior text and the recorded help reviews.
- Rust callers: `select_with_urls` no longer returns `SelectError::PageUrl(PageUrlError::Unsafe)`; it returns a `Notice` instead. `page_urls::readable_url` still returns the error for such a title.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
