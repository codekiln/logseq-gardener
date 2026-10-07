## Why

The user needs to try a Rust-generated site from selected Logseq namespaces by the coming weekend. A local publisher makes that workflow testable against [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden) and exposes product gaps before distribution work.

## What Changes

- Combine the reviewed loader, page titles, and namespace policy in a reusable SDK publisher and runnable Rust example.
- Write a navigable index and standalone HTML pages with outlines, headings, common inline markup, code, and selected-page links.
- Apply exclusions before rendering; withhold files containing parsed `public:: false`, and show placeholders plus local diagnostics for unsupported embeds and unresolved references.
- Copy supported assets referenced by selected rendered content, validating their paths and excluding unrelated files.
- Validate excluded content with fixtures and inspect a real-garden site locally.

## Capabilities

### New Capabilities

- `local-static-publishing`: Generate a selected, locally readable HTML garden with constrained references and assets.

### Modified Capabilities

None.

## Impact

Implements [#17 — Local HTML site](https://github.com/codekiln/logseq-gardener/issues/17). The branch builds on [PR #18 — Page titles](https://github.com/codekiln/logseq-gardener/pull/18) and reuses the selector from [PR #13 — Namespace selection](https://github.com/codekiln/logseq-gardener/pull/13) without merging either PR. A runnable SDK example supplies the local command while the `lsg` command interface remains a later product task.

The [project brief's publishing goal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md#3-static-knowledge-garden-publishing) calls for independently useful HTML documents. The [project proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md) requires filtered embeds and assets and visible unsupported-feature reporting. The user's October 6 instruction prioritizes direct Rust HTML generation; that is the current direction over the proposal's earlier Quartz trial.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md): demonstrate a local publishing workflow before adding deployment and broader graph features.
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md): document the initial visibility policy and report unsupported references.
