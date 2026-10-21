## Why

Gitpa's homepage depends on Logseq page and block embeds, media, and query results. Publishing it through logseq-gardener needs a selected homepage and safe transclusion so the site's existing content and layout remain recognizable.

## What Changes

- Render page and block embeds from selected visible content, including aliases, nested content, and assets relative to their original source.
- Bound recursive expansion and keep repeated transclusions free of duplicate HTML IDs.
- Add an SDK homepage option and `lsg publish --homepage` while preserving a separate page index and ordinary page routes.
- Exercise the publisher against the Gitpa homepage. Gitpa evaluates its Logseq Datalog queries through its nbb-logseq bridge before publication; the generic publisher keeps diagnostics for unsupported queries.

## Capabilities

### New Capabilities

- `selected-content-transclusion`: Render selected page and block embeds with visibility, resource, and identity protections.

### Modified Capabilities

- `local-static-publishing`: Permit bounded selected-content transclusion and publish a selected page at the homepage while retaining page navigation.
- `cli-static-publishing`: Accept and document a homepage selector through the SDK-backed publishing command.

## Impact

The SDK's site renderer, publication options, asset collection, CLI argument handling, offline help, tests, and publishing documentation change. Gitpa consumes the resulting release through its existing bridge and theme. Existing callers retain the current index homepage when they omit the new option.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
