## Context

The parser already supplies an Email node with local_part and domain fields. The renderer currently converts it to an unsupported literal. [Logseq 0.10.6’s email renderer](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/block.cljs#L1596-L1599) joins the local part and domain into a mailto link.

## Goals / Non-Goals

**Goals:** Preserve email navigation in published content while retaining the renderer's escaping, literal-code behavior, and prohibition on nested anchors.

**Non-Goals:** None.

## Decisions

Handle Email in the existing inline renderer. Reuse text and attribute escaping, and encode URI delimiter bytes in the destination so address data cannot become mail headers or fragments. Keep the source address as the visible label. The parser remains responsible for email syntax; minimal structural checks protect SDK callers constructing AST data directly.

Use the existing label context to render addresses as text within an outer anchor. Creating anchors unconditionally would produce invalid nested links. Code nodes continue through their existing literal branch.

## Risks / Trade-offs

- Address delimiters can change mailto semantics → percent-encode URI delimiter bytes and test the resulting destination.
- Unexpected SDK AST values can create invalid destinations → preserve the existing fallback for malformed components.
- Synthetic tests can miss parser behavior → include publication coverage using real Markdown parsing.

## Migration Plan

Published HTML gains mailto anchors when regenerated. Reverting this renderer change restores the existing literal fallback; source gardens require no edits.

## Resolved Questions

### Email links in Logseq 0.10.6

The locally available Logseq 0.10.6 Email renderer displays the joined address as a mailto link. The fixture reproduces Gardener's current unsupported literal for the same source.

## Open Questions

None.
