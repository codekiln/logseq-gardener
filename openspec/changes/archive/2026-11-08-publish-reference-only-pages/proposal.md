## Why

In Logseq, `[[Deno Deploy]]` makes a page even when no file holds it, and publish-spa publishes that page with the blocks that name it. `lsg` 0.0.38 renders the same link as plain text: on the encode garden, 3,872 linked or tagged names stay unresolved, 9,366 diagnostics in all ([Reference-only pages analysis](../../../../docs/parity/reference-only-pages.md), [Issue #351 Publish pages that exist only as link targets, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/351)). Readers of the garden lose those links, and 376 of the names already have a generated namespace page that a link in a block's text cannot reach.

## What Changes

- The SDK finds the reference-only pages of a publication: every name that a published block writes as a page reference, a tag or a page-valued property, together with that name's namespace parents, when no page file, alias or generated namespace page answers to it. A name whose file is withheld or outside the selection is left out and stays unresolved.
- Each reference-only page takes the title Logseq gives it: the spelling of the first reference Logseq reads, with boundary slashes dropped and a date name titled as its journal.
- The page-name index that links and tags use resolves reference-only pages and generated namespace pages, so `[[Deno Deploy]]`, `#company` and `[[AI]]` become page links for the CLI, the language server and the publisher alike.
- `lsg publish` writes one static page per reference-only page: its title, then Pages tagged with, Linked References, Hierarchy and Unlinked References, as Logseq shows a page with no blocks. A page whose address collides with another page or a reserved file is skipped with a diagnostic.
- Links and tags to a reference-only page look like any page link. Links to generated namespace pages keep their dotted underline.
- All pages lists reference-only pages with their backlink count, the journals feed lists reference-only journals, search finds reference-only pages by title, and the graph view, Page graph and right sidebar open them. `not published` remains for names with no published page.
- The `lsg publish` report counts reference-only pages.
- A reference-only page's Unlinked References hold at most one fragment file, about 50 pages; the heading counts every block and the section says how many pages it shows.
- Search title index entries leave out the page address when the browser can compute it from the title. Search format version 4 replaces version 3. **BREAKING** for hosts that read the title index.
- The publication view and `lsg-site.json` list reference-only pages with the kind `reference`.
- New budgets: a reference-only page's size, the site's size, page HTML and `_refs/` limits for the larger site, and the existing first-paint, search, phone memory and build-time budgets applied to the new pages.

## Capabilities

### New Capabilities

- `reference-only-pages`: the SDK operation that finds the reference-only pages of a publication, titles them, gives them routes and keeps the set current as files change.
- `published-reference-only-pages`: the published page for each reference-only page, how links reach it, how other views list and open it, its diagnostics and its budgets.

### Modified Capabilities

- `local-static-publishing`: page links resolve to reference-only and generated namespace pages; unresolved links remain only for withheld, unselected, ambiguous or skipped names.
- `published-namespace-hierarchy`: links and tags in page text resolve to generated namespace pages.
- `published-page-properties`: page-valued property values link to reference-only and generated namespace pages.
- `garden-graph`: a published reference-only node carries its page's route.
- `published-graph-view`: clicking a reference-only node with a route opens its page.
- `page-list`: entries of kind `Reference` for reference-only pages.
- `publication-view`: view pages of kind `reference`.
- `published-unlinked-references`: the one-file limit on a reference-only page's Unlinked References.
- `static-search-contract`: reference-only titles in the corpus, title entries without computable addresses, format version 4.
- `published-site-size`: new site, page HTML and `_refs/` budgets.
- `published-journals`: the journals feed lists reference-only journals dated on or before the build date.
- `cli-static-publishing`: the publish report counts reference-only pages.

## Impact

- SDK: a new `reference_only` module beside `references`, `title_spellings` and `namespaces`; `publication.rs` routes reference-only pages beside `namespace_destinations` and fills the page-name index with them and with generated namespace pages; `page_list.rs`, `graph.rs`, `search.rs` and the publication view gain the new kind.
- Publisher: `site/render.rs` stops marking these links unresolved; the page template reuses the generated page's sections without the Namespace notice; `site/references.rs` writes one Unlinked References file for these pages; `site/search.mjs`, `site/search.rs` and `site/graph.js` follow the new data.
- Search hosts: format version 4; a page published before the change asks the reader to reload, as for version 3.
- Parity harness: `scripts/parity/budgets.toml` gains a `[reference_only]` table and new `[site]` values; `tests/fixtures/parity-garden` gains fixture pages.
- Output: about 3,800 more pages on the encode garden, about 65 MB more output, measured in the Apply step.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
