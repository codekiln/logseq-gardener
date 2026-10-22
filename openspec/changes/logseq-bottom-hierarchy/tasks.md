## 1. Bottom hierarchy rendering

- [x] 1.1 Remove top namespace breadcrumbs and emit a bottom Hierarchy section with the current full-path row first and every navigable descendant at all levels in normalized order.
- [x] 1.2 Render flat full-path rows with independent cumulative-prefix segment links, escaped labels, slash separators, and protected gaps as plain text.
- [x] 1.3 Add an expanded native details/summary disclosure within the labelled Hierarchy section and wrap/focus styles for mobile and keyboard use; update publishing documentation.

## 2. Verification

- [x] 2.1 Test the exact A/B/C, D, E, E/F example, deeper descendant levels, every prefix link, namespaced leaf rows, bottom placement, sibling boundaries, disclosure state, and privacy/alias gaps while preserving routes, indexes, counts, media, and transclusions.
- [x] 2.2 Run repository checks and inspect the integrated Gitpa bottom hierarchy at desktop and mobile widths, including leaf ancestor navigation, all descendant rows, and retained audio playback.
- [x] 2.3 Review artifacts and implementation against the requirements and writing preferences, record source and browser evidence, and link the implementation PR to cleanup child #61.
