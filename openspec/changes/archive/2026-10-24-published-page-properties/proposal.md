## Why

Published pages currently hide authored leading properties, including useful linked metadata. Readers need a compact property section and a property backlink that opens the actual source metadata.

## What Changes

- Add opt-in linked page properties with configurable native disclosure.
- Exclude internal/control metadata and omit unsafe or unavailable reference values.
- Link property backlinks to the visible property section.
- Preserve generic authored prev/next links while avoiding duplicate sequence links when properties display them.

## Capabilities

### New Capabilities

- `published-page-properties`: safe visible leading properties, configuration and stable fragment navigation.

### Modified Capabilities

- `published-linked-references`: property source links target visible properties when available.
- `authored-page-sequences`: configurable generic sequence presentation alongside visible properties.

## Impact

Shared configuration, leading property rendering, sequence navigation and property backlink source links. Source documents remain unchanged. Existing sites retain their current display until enabled.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
