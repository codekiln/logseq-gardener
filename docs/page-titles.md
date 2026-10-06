# Derive Markdown page titles

Rust publishers can derive page titles from loaded Markdown page documents. A title can come from a leading `title::` property or from the filename. The caller chooses the filename format explicitly; `pages/Names___Nested.md` yields `Names/Nested` with TripleLowbar and `Names___Nested` with Legacy.

## Try the SDK

Run the summary example from this repository:

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example page_titles -- /path/to/garden triple-lowbar
```

Use `triple-lowbar` when the graph's `logseq/config.edn` sets `:file/name-format :triple-lowbar`; use `legacy` for the default filename format. The example derives all page titles, counts titles containing `/`, and reports how many journals it skipped. It stops at the first title error. It prints counts and leaves garden files unchanged.

A Rust caller can retain each title alongside its source location:

```rust
use logseq_gardener_sdk::garden::{DocumentKind, load_garden};
use logseq_gardener_sdk::page_titles::{FilenameFormat, page_title};

let garden = load_garden("/path/to/garden")?;
for document in &garden.documents {
    if document.kind == DocumentKind::Page {
        let title = page_title(document, FilenameFormat::TripleLowbar)?;
        // Keep document.relative_path as the source location for title.
    }
}
```

## Supported title rules

The API follows the Logseq OG source revision recorded in the [relationship experiment](../openspec/changes/current-og-relationships/experiments/file-relationships/README.md):

- A relative path starting with `pages/contents.` (using native path separators) yields `Contents`, including `pages/contents.extra.md`.
- A title in the first root parsed properties block overrides the filename. Property keys match without ASCII case; the last title property wins. Later drawers and bullet properties retain the filename title.
- Filename titles use the final stem. Directory names remain file locations rather than page namespaces.
- Legacy replaces dots with `/`, then decodes the entire percent-encoded UTF-8 string. A malformed escape or invalid UTF-8 preserves the whole dot-replaced string.
- TripleLowbar replaces each non-overlapping `___` with `/`, decodes individual ASCII percent escapes once, and removes empty slash segments. Non-ASCII byte escapes remain encoded, so `%C3%A9` remains `%C3%A9`; Legacy decodes it to `é`.

`PageTitleError` includes the relative path and a reason for unsupported journals, invalid page paths, and empty titles. Empty or whitespace-only title rejection is SDK validation for publishing; Logseq OG itself accepts an empty title override. The API preserves capitalization and other title characters. It derives a display title, while canonical lookup keys require separate normalization.

## Next publishing steps

[Issue #16 — Markdown page titles](https://github.com/codekiln/logseq-gardener/issues/16) supplies a prerequisite for [Issue #4 — SDK page lookup](https://github.com/codekiln/logseq-gardener/issues/4). Lookup still needs configuration loading, aliases and collision handling, journal titles, and names without source files. [Issue #11 — Static publishing](https://github.com/codekiln/logseq-gardener/issues/11) continues with namespace selection, links and assets, and HTML output. Title extraction does not determine publication visibility.
