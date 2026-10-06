## Context

The SDK currently exposes version information. Merged experiments favor lsdoc for read-only Markdown parsing, and current Logseq OG fixture relationships have been compared. The publishing priority now requires usable garden data, while broader corpus validation remains open. [Issue #14 — Load Markdown sources](https://github.com/codekiln/logseq-gardener/issues/14) is independent of the unmerged namespace-selection PR.

## Goals / Non-Goals

**Goals:** Return Markdown documents with original source, relative file location, parsed blocks, and parser references in deterministic order.

**Non-Goals:** Logical page identity, configured filename decoding, alias resolution, publication-property evaluation, and site rendering remain follow-up capabilities.

## Decisions

### Pin the tested parser for a limited Markdown reader

Use lsdoc at `32e63ef095c711d6d9947257bf5fd07d540fa59d`, the [tested revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d). The [merged findings](../parser-comparison-evidence/experiments/parser-comparison/findings.md) support read-only Markdown syntax and reference extraction; the [current OG comparison](../current-og-relationships/experiments/file-relationships/README.md) supplies later relationship evidence. The final broad adoption decision remains in Issue #3. This change records supported reader scope without claiming complete graph compatibility. Updating the parser or using mldoc would require new integration evidence.

### Preserve source alongside parser structure

Expose `Garden`, `GardenDocument`, and `DocumentKind` from an SDK `garden` module. Each document retains its unmodified source string, path relative to the supplied root, page/journal directory kind, and lsdoc `Projection`. Re-export the parser AST module so Rust callers can inspect the pinned syntax types. The SDK owns loading while the pinned parser owns syntax. Returning just rendered HTML would lose the source and relationship information needed by later publishing work.

### Traverse only garden note directories

Require a directory root containing at least one `pages/` or `journals/` directory. An absent counterpart is allowed. Recursively load lowercase `.md` files, skip names starting with `.`, reject visible symlinks and special files, and sort relative paths before parsing. Errors identify the attempted path and stop the whole load. Org support and filename interpretation wait for their own SDK scope. An empty valid note directory returns an empty garden; an unrelated directory returns an error.

### Treat loading as a local snapshot

The loader performs reads only. Tests compare source before and after loading, including CRLF and trailing whitespace. The example reports document and reference counts, without printing note contents. Publication code must later resolve identities and evaluate visibility before using any loaded content.

## Risks / Trade-offs

- Parser syntax types become part of this early SDK API → pin the revision and document the API as initial, with changes reviewed alongside parser updates.
- Parsing supplies no normal error result for malformed Markdown → test representative source and use the parser's existing permissive Markdown behavior; do not claim strict Markdown validation.
- Broader publishing compatibility is incomplete → demonstrate the reader on the public encode garden and retain Issue #3 for the outstanding corpus and final compatibility decision.
- Retaining all source and parsed documents uses memory proportional to the garden → measure the actual garden before introducing persistent caching.

## Resolved Questions

### 1 - Can a limited reader proceed while broader corpus validation remains open?

Yes. The merged syntax and source-preservation comparisons cover the pinned parser on the public garden and fixtures. This reader preserves raw parser results and leaves Logseq graph semantics to subsequent changes.

### 2 - Should an unrelated directory look like an empty garden?

No. Require at least one visible note directory; empty note directories are valid.

## Open Questions

None for the reader. Publication rendering and graph identity remain follow-up work.
