## Why

Published namespace pages need links to their parents and children so readers can move through related notes. A leaf such as an Auster preset contains its own Notes; the missing navigation makes its surrounding garden hard to discover.

## What Changes

- Add linked namespace ancestors and a Hierarchy section listing immediate children on published pages.
- Generate intermediate namespace pages from selected visible page titles when those ancestors have no published source page.
- Include generated namespace destinations in the all-pages index while preserving source-page counts, ordinary routes, and explicit page/alias lookup.
- Keep navigation accessible and readable on narrow screens.

## Capabilities

### New Capabilities

- `published-namespace-hierarchy`: Build visible namespace ancestry, intermediate destinations, and child navigation.

### Modified Capabilities

- `local-static-publishing`: Include generated namespace destinations in publication alongside selected source pages while preserving visibility and reporting semantics.

## Impact

The SDK site publisher gains a namespace navigation model, generated navigation pages, and HTML/CSS for breadcrumbs and children. SDK and CLI tests and local publishing documentation cover the resulting routes, source-page count, and index. Gitpa can consume the index to map these destinations into its public URL layout. Existing source rendering, transclusions, media, aliases, and homepage selection retain their behavior.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
