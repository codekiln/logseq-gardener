## Context

The renderer stores outline bullets in a flat sequence and ignores parsed properties after visibility and anchor extraction. Gitpa's episode preparation also strips collapsed properties. Together these behaviors make all eleven authored collapsed Dialogue sections on the October 20 episode expand on the website.

Logseq parses collapsed as a boolean, stores true as the block's collapsed state, and hides descendants until the reader opens the outline control. [The source reference](references/logseq-collapse.md) records the inspected parser and renderer paths.

## Goals / Non-Goals

**Goals:** Honor authored closed outline parents with accessible controls; preserve nested states, full content, publication visibility, media, transclusions, and block navigation; verify the episode in a mobile and keyboard-accessible browser view.

**Non-Goals:** None.

## Decisions

### Group each authored parent with its descendants

Recognize collapse only from the parsed property block attached to an outline parent. A true flag with descendants creates a native details element without the open attribute. Its summary exposes the parent's label; its body contains the original descendant subtree. Determine the boundary from outline levels so a following sibling stays outside. False or absent flags retain expanded rendering. A leaf retains its normal visible content without an empty disclosure.

Use a bounded traversal consistent with the existing renderer limits. Nested true flags create independently closed disclosures. Opening an outer parent leaves the nested parent's authored state intact.

### Keep content and safety rules intact

Render the complete retained subtree through the existing escaping, asset, link, and transclusion paths. Collapsing controls initial presentation; it does not remove content or alter source-page selection. The existing whole-page private-property check still runs before rendering, including properties below a collapsed ancestor.

Only attached parsed outline metadata supplies a collapse flag. Page-level properties and property-like text inside code or literal examples cannot collapse unrelated content. Keep summaries valid HTML; render flow content outside the summary when a rich parent label includes an embed or another block-level construct.

### Preserve reachable anchors

Keep original outline anchors and selected block-reference destinations. Navigating to a descendant inside closed disclosures reveals its containing disclosures. Prefer native browser fragment behavior and verify both initial URL fragments and later fragment changes; add only the small reveal support needed when native behavior is insufficient. Other closed branches keep their state.

### Use native controls and retain metadata at integration

Details/summary supplies keyboard interaction and state without a separate application framework. Keep focus indicators visible and long summary labels readable on narrow screens. Gitpa's preparation must retain collapsed metadata while keeping its existing publication-selection handling; the shared SDK owns rendering behavior.

## Risks / Trade-offs

- Incorrect flat-outline boundaries could hide following siblings → cover nested, skipped-depth, and sibling fixtures.
- Closed content could be mistaken for private content → retain full safe content and assets and test private descendants separately.
- Summary markup could contain invalid flow content → test rich labels and transclusions in and below collapsed parents.
- A deep-link target could stay hidden → exercise initial fragments and fragment changes in a browser.

## Migration Plan

Implement the SDK behavior and focused tests, document it, and run the repository checks. Retain collapse metadata in Gitpa's prepared episode source and generate a preview using the updated renderer. Inspect eleven initially closed Dialogue sections, their contents after keyboard expansion, nested states, fragment links, mobile wrapping, bottom Hierarchy, and continuing recording playback. Merge and deploy after validation. Track post-merge spec synchronization and archival in the cleanup child.

## Resolved Questions

### 1 - Does a collapsed outline remove its descendants from publication?

The published HTML retains its safe descendants and media. A closed native disclosure controls initial visibility, and the existing privacy rules still decide whether the page is published.

### 2 - What happens to false, absent, or leaf collapse flags?

False or absent flags keep the outline expanded. A leaf stays visible without an empty disclosure.

## Open Questions

None.
