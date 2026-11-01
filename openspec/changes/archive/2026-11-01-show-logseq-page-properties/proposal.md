## Why

Logseq 0.10.15 shows `alias::`, `public::` and most other page properties in a page's properties box, and `lsg` leaves them out ([Issue #385 — Show alias, title and public in the page properties box as Logseq does](https://github.com/codekiln/logseq-gardener/issues/385)). On the docs site, the `lsg` page shows only its `tags` row, while Logseq shows `alias: Logseq Gardener` above it. The requirement "Publish selected leading page properties safely" omits "internal metadata" without saying which keys that means, and the publisher reads it as a longer list than Logseq hides.

## What Changes

- The page properties box hides exactly the keys Logseq 0.10.15 hides from page properties: its hidden built-in keys, the card scheduling keys, `title`, `filters` and `icon`, and the keys in `:block-hidden-properties`. `alias`, `aliases`, `public`, `search-exclude`, `custom_id`, `updated_at`, `file`, `file-path`, `file-name` and keys that start with `logseq-proxy-`, `publishing/` or `logseq.`, other than Logseq's own hidden `logseq.` keys, now show when their values are safe.
- Each name in an `alias::` or `aliases::` value links to the page it names, labelled with the name as written, as Logseq shows it.
- The SDK gains `page_properties::is_hidden_page_property`, which the publisher calls.
- lsg/Ref/Published Pages lists the hidden keys, and the board lsg/Ref/Logseq Published Site/Frontmatter and Properties no longer says that `title`, `public` and `alias` get no row.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-properties`: the requirement "Publish selected leading page properties safely" names the keys it hides and shows alias values as links.

## Impact

- `crates/logseq-gardener-sdk/src/page_properties.rs`: new public module with `is_hidden_page_property`.
- `crates/logseq-gardener-sdk/src/site/properties.rs`: the properties box uses that function and renders alias values; query tables keep their current column rule.
- `crates/logseq-gardener-sdk/tests/site_properties.rs`: a test for the shown and hidden keys; the existing hidden-key test uses `hl-type` in place of `logseq-proxy-source`.
- Published sites with `:publishing/page-properties? true` gain rows, such as `alias` on the docs site's `lsg` page. Sites without that setting are unchanged.
- Docs: lsg/Ref/Published Pages and lsg/Ref/Logseq Published Site/Frontmatter and Properties.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
