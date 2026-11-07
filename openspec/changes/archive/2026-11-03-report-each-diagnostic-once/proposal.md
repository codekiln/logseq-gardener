## Why

`lsg publish` reports a problem in a page's text a second time for each page that shows a copy of the block, with the showing page's file in front of it ([Issue #352 — Print each publish diagnostic once when --homepage is set](https://github.com/codekiln/logseq-gardener/issues/352)). For example, a block with a missing page link that also links to `lsg/Nowhere` is listed in the Linked References of `lsg`, so the report says `pages/lsg.md: pages/lsg___Broken.md: Unresolved or excluded page reference: lsg/Nowhere` as well as `pages/lsg___Broken.md: Unresolved or excluded page reference: lsg/Nowhere`. The same happens for embeds and query results. The repeats make the report longer and point the reader at a file that does not hold the problem.

The issue's title names `--homepage`, but the repeats appear with or without it: they come from pages that show copies of other pages' blocks.

## What Changes

- A diagnostic that comes from a copy of another file's block (in an embed, Linked References or query results) is reported under that file, without the listing page's file in front of it.
- Such a diagnostic is reported once, and not at all when that file already reports the same problem for its own page.
- The page's own diagnostics stay as they are, including repeats of the same message within one page.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-linked-references`: the requirement "Render each listed block once per build" reports a copy's diagnostics once, under the file the copied block comes from, instead of for every page that lists it.
- `selected-content-transclusion`: the requirement "Render selected page and block embeds" reports embedded content's diagnostics once, under the embedded file, and adds a scenario.

## Impact

- `crates/logseq-gardener-sdk/src/site/page_diagnostics.rs`: new; moves each borrowed diagnostic to its file and drops repeats.
- `crates/logseq-gardener-sdk/src/site/mod.rs`: the page loop records diagnostics through it.
- `SiteReport::diagnostics` and the `lsg publish` report: fewer entries when pages show other pages' blocks; a borrowed diagnostic's `source` is now the file that holds the problem. Published files do not change.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
