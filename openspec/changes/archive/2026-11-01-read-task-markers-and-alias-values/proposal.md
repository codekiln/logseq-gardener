## Why

The SDK gives a garden different links than Logseq does for task markers and for the values of `alias::` and `tags::` ([Issue #361 — Read task markers and alias values as Logseq does](https://github.com/codekiln/logseq-gardener/issues/361)). A block that starts with `TODO` refers to the page TODO in Logseq and to nothing in the SDK. `alias:: Name` makes the block refer to Name in Logseq and is plain text in the SDK. `alias:: #[[Name]]` gives Logseq the alias `Name` and the SDK the alias `#[[Name]]`. On the encode garden, Logseq's parser check reports 1,104 block references that Logseq reads and the SDK does not.

## What Changes

- A block that starts with a task marker, such as `TODO` or `DONE`, refers to the marker's page, and a block with a priority, such as `[#A]`, refers to the priority's page, as in Logseq.
- The SDK reads the values of `alias::`, `aliases::` and `tags::` with one rule, Logseq's: each `[[Name]]`, `#name` and `#[[Name]]` names a page, a nested reference `[[Outer [[Inner]] Name]]` names `Outer [[Inner]] Name` only, and the plain text around them names a page for each comma-separated part. Each name is a page alias where the property is the page's `alias::`, and a reference of the block in every case.
- The parser check's deliberate mismatch moves from the alias fixture, which now passes, to a nested reference in another property's value, which [Issue #378](https://github.com/codekiln/logseq-gardener/issues/378) tracks.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `selected-page-aliases`: the requirement "Extract leading page aliases" reads tags and nested references in the alias value as Logseq does.
- `reference-index`: the requirement "Answer linked references from one shared index" adds task markers, priorities and the plain names in `alias::`, `aliases::` and `tags::` values to a block's references.

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: a new public operation, `linkable_property_names`, reads an `alias::`, `aliases::` or `tags::` value; page aliases use it.
- `crates/logseq-gardener-sdk/src/references.rs`: block references add the task marker and priority, and use `linkable_property_names` for those three properties, in place of the private `tag_names`.
- Published sites: the page graph, linked references, tagged pages and backlink counts change where a garden uses task markers, priorities or plain names in these properties. On the encode garden, five published pages change their reference sections and the diagnostic count stays at 35,671.
- `scripts/logseq-parser/`: the fixture `tag-alias` becomes `nested-property-reference`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
