# Combined garden trial

The [weekend guide](../../../../../docs/weekend-trial.md) was exercised on October 8, 2026 with the publisher at [7d673fe — navigation validation](https://github.com/codekiln/logseq-gardener/commit/7d673fe1bec28f9a2da82ce5ec64fd1b05cff6e8). The source was the existing public encode garden, based on [8018e1e — public garden](https://github.com/codekiln/logseq-encode-garden/commit/8018e1e073f2495df4b252dc25eb46625d3e9bbf) with its working contents preserved. The trial implements the demonstration goal in [Issue #11 — selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11); [Issue #41 — reproducible weekend trial](https://github.com/codekiln/logseq-gardener/issues/41) tracks the guide and feedback.

## Generated output

Used the guide's combined selection: include `AI/ES/25/ws` and `GitP/A/Session`, exclude `AI/ES/25/ws/3` and `GitP/A/Session/26/09/24-Thu`, with triple-lowbar filenames and a fresh output directory. Generation completed with 31 pages, 24 copied assets, no withheld pages, and 489 skipped journals. The report contained 814 diagnostics, including references outside the selected namespaces and unsupported syntax. Completion with diagnostics is expected; the report is a compatibility investigation aid, not evidence that every retained page is fully rendered.

Every generated UUID link resolved to an ID in its generated target: 37 links passed. Every relative file link found an emitted destination. Artwork and audio elements appeared in the combined output. The excluded workshop subtree and September 24 session were absent from the index. All generated files were scanned for the excluded September recording URL, its distinctive session description, and the distinctive Session Assets warning; none appeared. The excluded workshop’s distinctive typo-bearing note and hosted slide URL were also absent. These are bounded exclusion checks, supported by the mixed-visibility fixture coverage recorded in the [CLI verification](../../../cli-static-publishing/verification.md). Selection can retain a mention of an excluded workshop already written inside an included summary.

SHA-256 snapshots of all ordinary files beneath `pages`, `journals`, and `assets` matched before and after generation, covering 7,265 source files. The source garden and its existing edits were preserved.

## Browser evidence

Earlier checks on this implementation's feature branches demonstrated the workshop summary's reference jumping to the MCP note and the November episode artwork loading and recording playing. Those observations are recorded in [navigation verification](../../verification.md) and [media verification](../../../selected-garden-media/verification.md). This combined run checked emitted links, destinations, media elements, exclusions, and source preservation; it does not claim another playback inspection.

## Product gaps

Workshop headings referencing `AI Engineer Summit 2025 NYC Workshops` and `AIES 25 WS 1 - Building Agents with MCP - Mahesh Murag` remain unresolved despite matching aliases in the selected source. [Issue #42 — selected publishing alias lookup](https://github.com/codekiln/logseq-gardener/issues/42) tracks that reading interruption.

Gitpa's [preparation script](https://github.com/codekiln/gitpa/blob/main/scripts/prepare_site.py) selects explicitly public production-note sections and expands selected episode media embeds before the [publish-spa workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml). The Rust trial supports direct artwork and audio links, and whole-page privacy. Private-outline selection, embeds, homepage queries, and journal naming remain gaps to evaluate against the user's next site. The trial demonstrates a locally readable selection; replacing every Logseq garden publishing behavior remains unproven.

`mise run docs:check` passed, including README commands and local documentation links. The documented combined publish command was run directly; the existing browser evidence supplies playback and jump-navigation inspection.
