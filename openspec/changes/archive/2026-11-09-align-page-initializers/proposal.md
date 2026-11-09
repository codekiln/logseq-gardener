## Why

After a site plugin's host shows another page in place, an open Page graph card in the right sidebar keeps drawing the previous page. The "Page initializers" requirement of `published-plugin-host` also lists media, slides, flashcards, graph and sidebar wiring as initializers, but media, slides, flashcards and most of the sidebar listen on the document and need none. A browser check on 2026-11-09 found each of them working after a same-document navigation; only the Page graph card failed.

## What Changes

- The right sidebar registers a page initializer: each open Page graph card draws the new page, and a card in a closed sidebar draws it when the sidebar opens.
- Drawing the Page graph card again ends the previous drawing's layout worker.
- The "Page initializers" requirement names the behaviour that registers an initializer, and states that media, View as slides, flashcard reviews and the rest of the right sidebar keep working after a same-document navigation.

## Capabilities

### New Capabilities

### Modified Capabilities

- `published-plugin-host`: "Page initializers" lists the registered initializers, adds the Page graph card, and adds scenarios for the features that work without one.

## Impact

`crates/logseq-gardener-sdk/src/site/host.mjs`, `sidebar.js` and `graph.js`; unit tests in `tests/sidebar.test.cjs` and `tests/graph.test.cjs`; a browser check in `scripts/plugins/browser.py`, run by `mise run plugins:browser-test`; and the navigation steps in `lsg/Ref/Plugin Host API`. The host module grows by one line and stays under its 4 KB gzip budget.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
