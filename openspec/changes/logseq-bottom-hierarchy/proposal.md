## Why

Readers need Logseq's bottom Hierarchy view to browse complete namespace paths through every descendant level. The published site currently shows top breadcrumbs and only immediate children, which hides deeper paths and differs from the expected navigation.

## What Changes

- Replace top namespace breadcrumbs with a bottom Hierarchy view containing the current full path and every navigable descendant path.
- Render a flat list of full paths; make each available slash-separated segment link to its cumulative namespace destination.
- Keep the entire section expanded by default with an accessible collapse control.
- Preserve safe generated ancestors, private and alias protections, source-page counts, and source rendering.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-namespace-hierarchy`: Replace top breadcrumbs and immediate-child navigation with a bottom, expanded full-path list covering all descendant levels.

## Impact

The SDK hierarchy renderer, HTML shell, styles, tests, and publishing documentation change. Gitpa updates its renderer pin and styles to display the same structure. Existing routes, all-pages indexing, aliases, media, homepage selection, transclusions, and publication visibility retain their contracts.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
