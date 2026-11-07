## Why

A page property that links to a page through one of its aliases shows the target page's title, while Logseq 0.10.15 shows the name as written ([Issue #419 — Label property links to an alias with the name as written](https://github.com/codekiln/logseq-gardener/issues/419)). On the encode garden, `see-also:: [[Knowledge Gardener]]` on `Knowledge/Garden` reads `Knowledge/Garden/er` in lsg and `Knowledge Gardener` in Logseq. The page body already shows the name as written.

## What Changes

- A page link in a property value reads as the name the author wrote, so a link through an alias reads as the alias. It still opens the same page and still counts as a reference to it.
- A name written as the lower-cased title of the page it opens, such as `[[knowledge/garden/er]]` or `tags:: knowledge/tag page`, shows that page's title, as Logseq does.
- The `alias` and `aliases` rows keep their current labels.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-properties`: the requirement "Publish selected leading page properties safely" says how a property link is labelled.

## Impact

- `crates/logseq-gardener-sdk/src/site/properties.rs`: `value()` labels each page link with the name as written.
- Published sites with `:publishing/page-properties? true`: on the encode garden, 452 property links on 423 pages change their label, and 221 linked-reference fragments that show those rows change the same way. No link changes its target, and no row appears or disappears.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
