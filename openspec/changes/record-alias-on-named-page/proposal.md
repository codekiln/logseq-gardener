## Why

When one page lists another page that has its own file as an alias, Logseq records the alias on both pages, and `lsg` records it only on the page that lists it ([Issue #453 — Record an alias on the page it names too, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/453)). With `pages/brew.md` holding `alias:: [[Homebrew]]` and `pages/Homebrew.md` holding its own text, Logseq gives `Homebrew` the alias `brew`; the SDK gives it none, and `mise run docs:parser-check` reports 17 such differences on the encode garden.

On published pages the gap shows up as broken links and missing references. `lsg` treats `[[Homebrew]]` as ambiguous, because one page has the title and another lists it as an alias, so the link renders as plain text with a diagnostic: 37 such diagnostics on the encode garden. Logseq keeps one page per name, so the link opens `Homebrew`, and each page's Linked References also list the blocks that refer to the other page.

## What Changes

- The SDK records an alias on the page it names, as Logseq's file parser does: the named page gets the listing page and the listing's other names. The parser check compares these aliases.
- A name that is a page's title opens that page, although other pages list it as an alias. When the titled page holds no text and exactly one page lists the name, the name opens that page instead, as Logseq's links do.
- A page's Linked References also list the blocks that refer to the other pages of its alias group, with Logseq's muted `Alias` label on the group of such a page, and its Unlinked References look for every name of the group.
- The parser check's deliberate mismatch moves from `fixtures/alias-of-another-page`, which now agrees, to `fixtures/alias-namespace-parents`: an alias in a namespace, whose parents Logseq creates as pages and the SDK does not. [Issue #462](https://github.com/codekiln/logseq-gardener/issues/462) tracks that difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `selected-page-aliases`: "Preserve page name candidates" lets a title win over another page's alias, and a new requirement, "Record an alias on the page it names", gives each page Logseq's aliases.
- `published-linked-references`: a new requirement, "List the references of a page's alias group".
- `published-unlinked-references`: a new requirement, "Look for every name of a page's alias group".

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: `AliasGroups::stored_aliases` and `AliasGroups::group_pages` are new; `PageNameIndex::candidates` prefers a title.
- `crates/logseq-gardener-sdk/src/references.rs`: `ReferenceIndex::linked_with_aliases` is new. `ReferenceIndex::linked`, which queries and other callers use, is unchanged, because Logseq's `page-ref` query rule ignores aliases.
- `crates/logseq-gardener-sdk/src/site/`: the publisher reads alias groups for the Linked and Unlinked References sections and adds the `Alias` label and its style.
- `crates/logseq-gardener-sdk/tests/relationships/mod.rs`: the parser check reads each page's aliases from `AliasGroups::stored_aliases`.
- `scripts/logseq-parser/`: the fixture `alias-namespace-parents` replaces `alias-of-another-page`, and its test follows.
- `lsg lookup` keeps listing every page that matches a name, with how it matched.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
