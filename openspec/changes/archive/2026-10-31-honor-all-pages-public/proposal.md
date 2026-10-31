## Why

Logseq publishes only pages marked `public:: true` unless `logseq/config.edn` sets `:publishing/all-pages-public? true` or the older `:all-pages-public? true`. `lsg publish` ignores both settings and publishes every selected page that lacks `public:: false`, so a garden that keeps Logseq's default gets pages from `lsg` that Logseq would keep private ([Issue #268](https://github.com/codekiln/logseq-gardener/issues/268)).

## What Changes

- Read `:publishing/all-pages-public?` and `:all-pages-public?` from `logseq/config.edn` in the SDK's garden settings. Either key set to `true` turns the setting on, as in Logseq's publishing export.
- When the setting is off or absent, publish a page or journal only when its leading properties set `public:: true`, or when it shares an alias with such a page. Count every other selected page as withheld, with a diagnostic.
- Keep namespace selection and `public:: false` as they are: a page is published when it is selected, public under the setting, and has no `public:: false`.
- A garden folder without `logseq/config.edn` keeps publishing every selected page except those marked `public:: false`.
- **BREAKING** for gardens whose `config.edn` leaves the setting off: their sites shrink to the pages Logseq would publish.

## Capabilities

### New Capabilities

### Modified Capabilities

- `local-static-publishing`: Page selection applies the graph-wide all-pages-public setting and `public:: true` alongside namespace selection.
- `published-journals`: Journals follow the same public-page rule.

## Impact

The SDK's garden settings, the `publishing` module and site page selection. Test gardens that write a `config.edn` and expect every page published now set the key. The encode garden sets `:publishing/all-pages-public? true`, so its site is unchanged. Gitpa sets it to `false` and selects only `public:: true` pages already, so its site is unchanged when it moves to a release with this change.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
