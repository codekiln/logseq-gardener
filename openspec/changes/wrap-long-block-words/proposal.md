## Why

On a phone, a long page reference with no spaces, such as `[[Ableton/Live/Mode/Arrangement]]`, does not wrap. On `GitP/A/Session/26/10/24-Sat` at 390px the page lays out 449px wide and scrolls sideways. Logseq's block content breaks such words, so its pages fit the screen ([Issue #237 — Long page references overflow the screen on phones](https://github.com/codekiln/logseq-gardener/issues/237), part of [Issue #106 — Match Logseq's published site](https://github.com/codekiln/logseq-gardener/issues/106)).

## What Changes

- Outline blocks break a word that does not fit the width, as Logseq's `.block-content` does with `overflow-wrap: break-word` and `word-break: break-word` ([block.css](https://github.com/logseq/og/blob/master/src/main/frontend/components/block.css)).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-outline`: block content breaks long words so the outline fits the screen.

## Impact

- Publisher: one declaration in `crates/logseq-gardener-sdk/src/site/outline.css`. Linked references, hierarchy rows, breadcrumbs and search results already break long words.
- Performance: 22 bytes of inline CSS per page; no script change.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
