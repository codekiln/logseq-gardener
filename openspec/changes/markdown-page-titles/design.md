## Context

The loader in [PR #15 — Markdown garden sources](https://github.com/codekiln/logseq-gardener/pull/15) supplies source paths and lsdoc syntax. Publishers need the corresponding page titles before applying the namespace selection in [PR #13 — Namespace selection](https://github.com/codekiln/logseq-gardener/pull/13).

The [saved relationship findings](../current-og-relationships/experiments/file-relationships/findings.md) establish title overrides and configuration-dependent names. The associated Logseq OG revision is `6efedb75588763af256bc7dfd0ed5526dc91fe7c`; its [title extraction](https://github.com/logseq/logseq/blob/6efedb75588763af256bc7dfd0ed5526dc91fe7c/deps/graph-parser/src/logseq/graph_parser/extract.cljc#L30) and [filename decoding](https://github.com/logseq/logseq/blob/6efedb75588763af256bc7dfd0ed5526dc91fe7c/deps/graph-parser/src/logseq/graph_parser/util.cljs#L127) guide this API. A title retains capitalization; a canonical lookup key requires additional normalization.

## Goals / Non-Goals

**Goals:** Supply page titles through a read-only API, reproduce the recorded filename and property behavior, and demonstrate extraction across the pages in [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden).

**Non-Goals:** Load graph configuration, format journal dates, resolve aliases, and render the selected garden. These remain subsequent steps toward publishing.

## Decisions

### Require the caller to choose the filename format

Expose `page_title(&GardenDocument, FilenameFormat) -> Result<String, PageTitleError>`. The format enum has `Legacy` and `TripleLowbar` variants, with no implicit default. Inferring the format from underscores would misidentify legacy files; requiring the caller to read the graph's setting avoids that ambiguity while configuration loading remains under [#4 — SDK page lookup](https://github.com/codekiln/logseq-gardener/issues/4).

### Read title overrides from the first parsed properties block

Use only `parsed.blocks.first()` when it is `Block::Properties`. Property keys match without ASCII case, and the last title wins. A later drawer or bullet property retains its block meaning. Text scanning would mistake such block properties or literal code for page titles. Match Logseq's earlier `pages/contents.` special case before looking at properties, interpreting native path separators as `/`.

### Preserve the observed decoding order

Use the final filename stem, preserving directory location separately. Legacy replaces dots with slashes, then percent-decodes the whole string only when every escape and the resulting UTF-8 are valid. TripleLowbar replaces `___` with `/`, decodes each ASCII `%HH` independently, preserves invalid or non-ASCII byte escapes, and removes empty slash segments. This deliberately retains the OG behavior where `%C3%A9` remains encoded in TripleLowbar but becomes `é` in Legacy. A general URL decoder would change these names.

### Reject inputs whose title this API cannot supply

Require a relative path of normal components beneath `pages/` with a lowercase `.md` extension and a UTF-8 stem. Return a path-bearing error for journals, malformed paths, and empty or whitespace-only titles. Empty-title rejection is SDK validation for publishing; OG accepts an empty override. Preserve all other title characters, including capitalization and boundary slashes in property values. Canonical lookup normalization belongs to the later index.

## Risks / Trade-offs

- Configuration loading remains manual → the example requires a format argument and documents the public garden's active setting.
- An extracted title does not establish a unique lookup destination → keep aliases, collisions, fileless names, and canonical keys in the lookup work.
- Fixture agreement does not prove complete Logseq compatibility → test the saved page-title cases and source-derived decoding edges, and state the scope of the real-garden demonstration.

## Migration Plan

Add the module alongside the loader; existing API callers remain unchanged. Stack the PR on the loader branch and retain that dependency until the loader is merged with user authorization.

## Resolved Questions

### 1 - Which source defines the supported title rules?

The Logseq OG revision recorded by the saved relationship experiment defines the rules. The local Logseq checkout's current DB-graph source is a different revision and is not the comparison target.

### 2 - How will journals be handled?

Return a clear unsupported-input error. [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden) formats journal titles as `yyyy-MM-dd EEE`. The saved Logseq OG experiment used a different default date format, so publishers need to read each graph’s journal settings before deriving journal titles. Journal configuration and formatting require their own lookup task.

## Open Questions

None for this bounded change.
