## Why

Slashes sit too close to linked namespace segments in the bottom Hierarchy list. A small gap makes the paths easier to scan.

## What Changes

- Add a logical horizontal margin of 0.2em on each side of existing hierarchy slash separators.
- Preserve the separators as plain text and retain natural path wrapping.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-namespace-hierarchy`: Give slash separators visible spacing within hierarchy paths.

## Impact

One declaration in the shared default stylesheet affects every generated site. Existing hierarchy markup, navigation, and publication selection remain unchanged.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing.md)
