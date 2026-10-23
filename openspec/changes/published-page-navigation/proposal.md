## Why

Readers cannot tell a generated namespace destination from a published source page. Authored page sequences also lose their Previous and Next links during publication, making episode and manual navigation harder.

## What Changes

- Label generated destinations as “Namespace” in their page content, the page index, and hierarchy links, with a link style that differs beyond color.
- Publish accessible Previous and Next links from explicit leading page properties, using canonical selected source routes.
- Keep private, excluded, ambiguous, and self targets out of sequence navigation.

## Capabilities

### New Capabilities

- `authored-page-sequences`: Safe Previous and Next navigation from leading page properties.

### Modified Capabilities

- `published-namespace-hierarchy`: Visible distinction between published source pages and generated namespace destinations.

## Impact

The shared Rust publisher gains destination labels and sequence navigation. Existing source selection, aliases, media, embeds, and homepage validation retain their behavior. Gitpa must retain and supply episode sequence properties separately.

## Citations

- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
