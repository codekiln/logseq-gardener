## Why

A block whose property name Logseq rejects, such as one that starts with the code `` `logseq-entity:: [[Logseq/Entity/Asset]]` identifies a local asset. ``, shows Logseq's warning "Invalid property names" on Logseq's published site. On an `lsg` site the same block is empty, so a reader cannot tell why ([Issue #439 — Show Logseq's warning on a block whose property name Logseq rejects](https://github.com/codekiln/logseq-gardener/issues/439)). Logseq's build of the encode garden shows the warning on 13 blocks of 11 pages, such as the block under "Frontmatter and body" on `Logseq/Entity/Asset`.

Since [PR #440 — drop the property names Logseq rejects](https://github.com/codekiln/logseq-gardener/pull/440), the SDK drops these properties and their references. It does not yet say which names it dropped, so the publisher has nothing to show.

## What Changes

- The SDK reports the property names Logseq rejects in each properties block, written as Logseq writes them in the warning: lower-cased, with spaces and underscores turned into hyphens, each name once, in the order the block gives them.
- The publisher renders Logseq's warning in place of each such properties block, with Logseq's markup, text and styles: a `div.invalid-properties` holding "Invalid property names:", a button for each name and a `code` element that says which names are valid.
- On the encode garden, the same 13 blocks on the same 11 pages show the warning as in Logseq's build, with the same names and the same markup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-garden-loading`: a new requirement, "Report the property names Logseq rejects", says which names the SDK reports for a properties block and how it writes them.
- `published-page-outline`: a new requirement, "Show Logseq's warning for rejected property names", says where the publisher shows the warning and what it looks like.

## Impact

- `crates/logseq-gardener-sdk/src/property_names.rs`: `rejected_property_names`, the SDK call the publisher uses, and the property-line reader that `references.rs` already used, now shared.
- `crates/logseq-gardener-sdk/src/references.rs`: reads property lines with the shared reader. Its behaviour is unchanged.
- `crates/logseq-gardener-sdk/src/site/render.rs` and `site/inline.css`: the warning's markup and styles. The styles go into the site's shared stylesheet; pages load no new script.
- Published sites: pages with such a block gain the warning. Every other page is unchanged apart from the shared stylesheet.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
