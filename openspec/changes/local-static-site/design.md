## Context

The loader, page-title API, and namespace selector have tested implementations in open PRs. [The publishing brief](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md#3-static-knowledge-garden-publishing) asks for independently useful HTML pages, while [the proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md) asks for filtered references, embeds, and assets. The user prioritizes trying direct Rust HTML generation before distribution.

The [gitpa workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml) passes its graph directory to publish-spa. Its upstream namespace proxy imports Markdown into that graph. This publisher instead selects documents at generation time using the shared SDK syntax and titles.

## Goals / Non-Goals

**Goals:** Provide a repeatable local command, readable selected-page HTML, constrained asset copying, explicit unsupported-feature reporting, and evidence that excluded content stays out of output.

**Non-Goals:** Full Logseq query execution, embed transclusion, complete alias/journal lookup, incremental builds, and deployment. These remain useful follow-ups after the local demonstration.

## Decisions

### Publish selected pages into a fresh directory

Expose `publish_site(root, output, format, selection)` through the SDK, returning page and asset counts plus source-associated diagnostics. An example accepts explicit garden root, output path, filename format, and repeated include/exclude flags. Reuse the selector source from its reviewed PR rather than redefine its semantics.

The output directory must be absent and its parent must exist. Resolve the parent to its real path and reject destinations beneath the source garden; create the new output directory without replacing anything. Prepare titles, rendered documents, and supported asset contents before creating output. Input failures leave output absent. A write failure can leave a partial new directory and is reported; rebuilds use another fresh directory. This avoids stale excluded files remaining from a prior publication.

### Withhold files with explicit private properties

Apply namespace inclusion and exclusion to display titles. Also withhold an entire file if any parsed properties block contains `public:: false`, including nested outline properties. This is a conservative initial policy: a private nested block withholds its whole page. Literal property-like text in code is ordinary content. Namespace selection supplies publication intent for pages lacking an explicit private property; `public:: true` does not override exclusions. Journals are skipped with a count because configured journal naming remains separate work.

### Resolve links only against selected titles

Build a destination map from selected titles using Unicode lowercase comparison. Reject duplicate selected lookup names and generated route collisions. Canonical normalization, aliases, and fileless graph identities remain under [#4 — SDK lookup](https://github.com/codekiln/logseq-gardener/issues/4). Unresolved and excluded page links retain labels from selected source and get local diagnostics. Block references and macros, including embeds and queries, receive visible placeholders and diagnostics; the publisher performs no transclusion.

### Write standalone HTML with escaped source content

Render outlines with nesting, headings, paragraphs, emphasis, code, quotes, tables, and allowed links from the parsed AST. Escape text and attributes; source HTML and Hiccup remain escaped literal content with diagnostics. Keep unsupported syntax visible where possible and report it. Allow ordinary HTTP, HTTPS, and mailto anchors, while remote images are placeholders with diagnostics. Generated pages use relative links and embedded CSS, so a browser can read the index and pages directly from disk.

Assign each selected title a deterministic FNV-1a route such as `p-<digest>.html`, checking collisions before writing. Fixed-length filenames handle long garden titles and avoid path traversal. Output titles and index labels come only from selected documents.

### Copy referenced assets from the garden assets directory

Resolve a local URL relative to its source document after strict percent decoding. Normalize traversal lexically and require the result beneath `assets/`. Reject absolute paths, URL schemes, query/fragment suffixes, symlinks in every asset component, special files, and unsupported extensions. Copy supported raster images, PDF, and common audio/video files into deterministic asset routes. Read only assets requested by rendered selected content; excluded pages and unsupported embeds cannot request copies. Asset errors become selected-source diagnostics and placeholders.

## Risks / Trade-offs

- A private child withholds more than its own text → document the whole-file policy and defer precise block visibility until graph relationships are ready.
- Lowercase title matching covers a subset of Logseq resolution → document alias, Unicode-normalization, journal, and block-reference limitations and show unresolved references.
- Unsupported syntax reduces fidelity → inspect representative real pages and keep a local diagnostic report for follow-up product work.
- Filesystem state can change during generation → guarantees assume no concurrent mutation of source, output parent, or newly created output; writes never intentionally replace an existing directory.

## Migration Plan

Add SDK modules and a Rust example alongside the current APIs. Keep existing PRs open and make this PR depend on page-title and namespace-selector work. Users can try the worktree's example before a merge. Source gardens remain the input of record.

## Resolved Questions

### 1 - What proves the initial publisher works?

Generate a selected namespace from the current public garden, inspect the index and a representative page in a browser, follow a generated link, and verify fixture exclusions and source preservation. Record the command and supported limitations before calling the local workflow usable.

### 2 - How will unsupported embeds and queries appear?

Show escaped selected-source syntax or a readable placeholder and report the unsupported construct locally. Resolve neither queries nor transclusions in this demonstration, so excluded destinations contribute no content.

## Open Questions

None for this bounded local demonstration.
