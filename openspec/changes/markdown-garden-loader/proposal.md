## Why

Rust publishers need to read the garden's Markdown notes before they can resolve page names or render a site. A read-only SDK loader supplies source text, file locations, parsed structure, and references for the current [static site goal](https://github.com/codekiln/logseq-gardener/issues/11).

## What Changes

- Pin lsdoc to the revision covered by the merged parser comparison and use it for Markdown loading.
- Load visible Markdown files under `pages/` and `journals/`, preserve their original source, and return documents in deterministic order.
- Reject visible symlinks and unreadable or invalid UTF-8 inputs instead of silently omitting notes.
- Add fixtures and a runnable public-garden example, with an explicit supported scope.

## Capabilities

### New Capabilities

- `markdown-garden-loading`: Read Markdown source files and retain parser structure and reference sets through the SDK.

### Modified Capabilities

None.

## Impact

Adds an SDK module, a pinned parser dependency, and a garden-reading example under [Issue #14 — Load Markdown sources](https://github.com/codekiln/logseq-gardener/issues/14). Page identity and publication filtering remain subsequent work under [Issue #4 — Page lookup](https://github.com/codekiln/logseq-gardener/issues/4). The CLI remains unchanged. Broader corpus validation continues in [Issue #3 — Parser comparison](https://github.com/codekiln/logseq-gardener/issues/3).

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md): build and verify the reader independently of indexing and rendering.
