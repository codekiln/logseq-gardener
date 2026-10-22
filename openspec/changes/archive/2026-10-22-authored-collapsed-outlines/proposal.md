## Why

Published notes should respect the author's collapsed outlines so readers can choose when to open long details. The October 20 episode contains eleven collapsed Dialogue sections, but the published site expands every transcript.

## What Changes

- Honor an outline parent's attached `collapsed:: true` with an initially closed native disclosure.
- Preserve nested collapse states, sibling boundaries, full content, media, transclusions, anchors, and existing privacy checks.
- Keep absent or false collapse flags expanded and literal property examples unchanged.
- Support keyboard disclosure controls, readable mobile layouts, and navigation to descendants inside closed outlines.

## Capabilities

### New Capabilities

- `authored-outline-collapse`: Render authored collapsed outlines with accessible controls and preserved publication semantics.

### Modified Capabilities

None.

## Impact

The SDK outline renderer, HTML presentation, focused tests, and publishing documentation change. Gitpa must retain authored collapse metadata in its prepared episode source before consuming the renderer update. Its existing visibility selection remains in place. Source gardens and publication identity remain unchanged.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
