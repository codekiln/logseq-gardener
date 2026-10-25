## Why

Published search data needs an accessible visitor interface that works across ordinary pages and hosts with persistent navigation. A lazy shared dialog lets visitors search without adding the corpus to normal page loading.

## What Changes

- Add a labelled header search button and Cmd/Ctrl+K shortcut.
- Add a persistent native dialog with safe result links, keyboard navigation, focus restoration and loading/empty/error/retry states.
- Load and validate revision-specific artifacts on first open; expose idempotent SPA initialization and route adaptation.

## Capabilities

### New Capabilities

- `accessible-static-search`: Shared lazy search dialog, keyboard/focus behavior, publication loading and host integration.

### Modified Capabilities

None.

## Impact

Search-enabled generated HTML gains one small shared controller and themed dialog styles. Existing pure index/evaluator APIs remain the search contract. Implements [issue95 — accessible search dialog](https://github.com/codekiln/logseq-gardener/issues/95) under [issue93 — static search](https://github.com/codekiln/logseq-gardener/issues/93). Representative performance validation remains issue96.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
