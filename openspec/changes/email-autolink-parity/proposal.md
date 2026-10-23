## Why

Email autolinks such as `<reader@example.org>` appear as unsupported code in published pages. Logseq 0.10.6 renders these addresses as mailto links, so readers should retain that navigation when publishing with Gardener.

## What Changes

- Render parsed email autolinks as escaped mailto anchors.
- Preserve literal code, safe link labels, and visible fallbacks for malformed email data.
- Verify the behavior through renderer and publication tests and document support.

## Capabilities

### New Capabilities

- `email-autolinks`: Email address navigation in published HTML, including escaping and link-label behavior.

### Modified Capabilities

None.

## Impact

The shared SDK renderer, its tests, and local publication documentation change. The parser, public APIs, dependencies, and publication selection stay unchanged.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing.md)
- [Logseq 0.10.6 email rendering](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/block.cljs#L1596-L1599)
