# Architecture

Logseq Gardener separates the Rust API from the command-line interface. Other Rust applications can depend on `logseq-gardener-sdk` without installing `lsg`.

## Packages and modules

| Component | Responsibility |
| --- | --- |
| `logseq-gardener-sdk` in `crates/logseq-gardener-sdk/` | The reusable Rust API. It exposes the SDK version, a logical namespace-selection policy, a read-only Markdown loader, and configured page-title extraction. It can [publish selected pages as local HTML](local-site.md). Complete page lookup and alias resolution remain planned. |
| `logseq-gardener` at the repository root | The CLI package, which builds `lsg` and depends on the SDK. |
| `src/cli.rs` | Validates arguments and dispatches help and version requests. |
| `src/help.rs` | Holds the help document and selects its sections for text or JSON output. |
| `src/output.rs` | Writes results and diagnostics to their respective output streams and handles write failures. |

The CLI handles terminal input and output. Garden loading belongs in the SDK so the CLI and other Rust applications can call the same code.

## Garden files and parsing

Markdown files hold the garden's notes. Any future index must be rebuildable from those files. An editing command should be able to change a note without reformatting unrelated text.

The limited Markdown reader uses the tested `lsdoc` revision; official `mldoc` provides the syntax reference, and Logseq OG's graph parser and graph-validator provide references for relationships between notes. The [parser comparison findings](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/findings.md) favor lsdoc for read-only Markdown parsing on the tested fixtures and public gardens. The [page-title API](page-titles.md) derives titles with an explicit filename format. The SDK still needs canonical lookup keys, aliases, journal identities, UUID resolution, and fileless targets. Retained source text and source ranges belong alongside parsed structure so future edits can preserve formatting.

The [current Logseq OG relationship comparison](../openspec/changes/current-og-relationships/experiments/file-relationships/findings.md) reproduces the earlier fixture relationships under default and configured filename interpretation. The results preserve the recorded page lookup rules for titles, aliases, namespaces, journals, and fileless targets. Pengx's publishing corpus and the final adoption decision remain part of [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3).

## Publishing

The SDK owns namespace selection so Rust publishers and the CLI can share it. [Publishing](publishing.md) describes the current static site priority, the runnable selection example, and the remaining garden-loading and rendering work.
