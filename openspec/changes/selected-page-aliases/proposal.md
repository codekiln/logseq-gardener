## Why

The published workshop summary shows alias headings as unresolved text even when their destination pages are included. Readers should reach those notes through the names used in the garden, as requested in [Issue #42 — selected page aliases](https://github.com/codekiln/logseq-gardener/issues/42) and the [publisher compatibility analysis](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Analysis___Codex___26___09___12___0733%20ET%20Specify%20which%20Logseq%20features%20the%20publisher%20supports.md).

## What Changes

- Add reusable SDK candidate lookup for page names and leading page aliases.
- Resolve unique names within the visible publication selection to existing routes while retaining source labels.
- Keep ambiguous names visible with diagnostics and verify withheld destinations remain absent.
- Extend the weekend trial with working workshop alias navigation.

## Capabilities

### New Capabilities

- `selected-page-aliases`: bounded Markdown page alias extraction and candidate resolution for publishing.

### Modified Capabilities

- `local-static-publishing`: selected page links also resolve unique aliases within the visible selection. The current requirement comes from the pending publisher, media, and outline-navigation changes.

## Impact

The SDK gains a page-name index used by HTML rendering. Namespace selection continues to use logical page titles. Unicode normalization adds a small Rust dependency. The CLI command and generated routes remain compatible.

## Citations

- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md) supports explicit candidate ambiguity.
- [My/Principle/CLI/Centricity/Offline Tools](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___CLI___Centricity___Offline%20Tools.md) supports local generation and verification.
