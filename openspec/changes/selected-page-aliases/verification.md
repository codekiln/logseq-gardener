# Selected page alias navigation

The workshop summary's schedule and MCP headings now navigate to their selected pages in the in-app browser. The schedule link opened `AI/ES/25/ws`; the MCP link opened `AI/ES/25/ws/1/Building Agents with Model Context Protocol`. The source alias text remained the visible link label.

## Reproduce

From this branch, build and run the [weekend trial](../../../docs/weekend-trial.md). For a workshop-only check:

```sh
cargo run --bin lsg -- publish --graph "$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)" --output /tmp/fresh-workshop-alias-site --filename-format triple-lowbar --include AI/ES/25/ws --exclude AI/ES/25/ws/3 --format json
```

The destination must be absent. Serve that destination on loopback and follow the schedule and MCP alias headings in the workshop summary.

## Evidence

The local public garden was at `8018e1e073f2495df4b252dc25eb46625d3e9bbf`. Generation produced eleven pages and copied twenty-four assets. Every emitted local page or outline link resolved: sixty-two links, including the schedule alias and both occurrences of the MCP alias. The index contained no excluded workshop destinations; the excluded workshop subtree had no output routes. Fingerprints of all source files under pages, journals, assets and logseq were identical before and after generation.

SDK tests reproduce `Other Name` ambiguity from the saved OG fixtures and cover last leading properties, nested properties, comma-containing names, code and quoted literals, title overrides, Unicode normalization, alias/title collisions, source labels, deterministic output and source preservation. A selected alias shared with excluded and private pages resolved to the selected page; withheld names, text, assets and URLs remained absent from generated output. Normalized duplicate selected titles failed before output creation.

`mise run ci` passed. The normalization dependency source was inspected from Cargo's local cache. Its dependency records were regenerated as required by the repository checks.

## Remaining behavior

Unavailable and ambiguous names keep visible labels and local diagnostics. Journals, fileless names, transclusion, queries and precise private-outline publication remain outside this slice. Namespace exclusions do not redact text already present in retained source pages.

## Cleanup

[Issue #44 — sync and archive selected aliases](https://github.com/codekiln/logseq-gardener/issues/44) waits for implementation integration and the publisher, media and outline-navigation specification cleanup before synchronizing this delta and archiving the change.
