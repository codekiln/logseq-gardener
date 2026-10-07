## Context

The publisher plans selected visible pages before rendering. Its flat Logseq outline AST contains a bullet followed by a property group. Logseq OG normalizes `custom-id` and `custom_id` to the built-in `id` property and parses the ID as a UUID; this change accepts the ordinary hyphenated UUID spelling used by the public garden.

The workshop summary references nested notes in the MCP workshop. A local page-and-anchor link gives the reader a useful destination while retaining the existing publication boundary.

## Goals / Non-Goals

Goals: readable navigation to unique explicit IDs on rendered outline items, selected-source diagnostics for unavailable references, and unchanged source gardens.

Non-Goals: inline block transclusion, inferred IDs, and query expansion.

## Decisions

Build the UUID index after namespace selection and private-page withholding. Duplicate IDs among published targets disable that UUID's navigation. Each outline item can emit an anchor only when its explicit ID resolves uniquely to that item. An excluded page cannot add labels, links, assets, or ambiguity to the index.

Associate an immediately following property group with the preceding Markdown outline item, matching the pinned parser's AST representation. Root page properties and IDs on unsupported constructs receive no outline target. Accept `id`, `custom-id`, and `custom_id`, with the parser's normalized property keys and Logseq's custom-ID precedence. Normalize UUID case for matching and fragment generation.

Default labels use a bounded plain-text projection of the selected target's inline content. Rendering a label does not follow references, copy assets, or evaluate macros. Existing source labels retain precedence. Empty or unsupported target labels fall back to the UUID spelling. The anchor belongs on the owning list item so fragment navigation reveals the referenced note.

## Risks / Trade-offs

A reference is navigational instead of Logseq's inline replacement text. The guide states this limit. Only explicit IDs on Markdown outline items are targets; unavailable references retain their source text and a diagnostic. Duplicate IDs remain unavailable until the source is corrected. Selected target text is escaped and capped to keep links readable.

## Migration Plan

Merge after selected garden media and publishing integration. Existing CLI arguments and SDK publication calls continue to work. Reverting the change restores literal reference fallbacks.

## Resolved Questions

### 1 - What should a reader see for a reference without an explicit label?

Use bounded plain text from the selected target's outline item. Public workshop references otherwise appear as opaque UUIDs. The label never expands nested children or another document.

## Open Questions

None.
