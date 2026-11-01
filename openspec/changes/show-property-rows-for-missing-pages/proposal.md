## Why

Logseq 0.10.15 shows a page property whose value links a page with no file, and the link opens that empty page. `lsg` leaves the whole row out of the Properties box ([Issue #409 — Show page property rows that link to a page with no file](https://github.com/codekiln/logseq-gardener/issues/409)). On the encode garden, 775 pages lose at least one row this way, mostly `tags` and `date-created`; [Knowledge/Garden](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Knowledge___Garden.md) loses `tags:: [[AI/Generated]]`. The page body already shows the same reference as an unlinked name.

## What Changes

- A reference in a page property to a name that no page or journal file in the garden has, as a title or an alias, shows the name as written, unlinked, in the same `page-reference unresolved` span the page body uses. The row stays, and the property's other values keep their links.
- A reference to a page that the garden has but the site leaves out, because the page is withheld, outside the selected namespaces or ambiguous, still drops the whole row, as before.
- The SDK's `Publication` gains `garden_names`: the lower-cased titles and aliases of every page and journal in the garden, published or not. The properties box uses it to tell a page with no file from a page the site leaves out.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-properties`: the requirement "Publish selected leading page properties safely" shows references to pages with no file instead of omitting their rows.

## Impact

- `crates/logseq-gardener-sdk/src/publication.rs`: new `Publication::garden_names` field.
- `crates/logseq-gardener-sdk/src/site/properties.rs` and `site/mod.rs`: the properties box renders names with no page as unlinked names.
- `crates/logseq-gardener-sdk/tests/site_properties.rs`: a test built from the issue's example page.
- Published sites with `:publishing/page-properties? true` gain rows. Sites without that setting are unchanged.
- Docs: the Rows section of lsg/Ref/Published Pages.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
