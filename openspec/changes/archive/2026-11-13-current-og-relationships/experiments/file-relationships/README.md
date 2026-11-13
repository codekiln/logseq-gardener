# Logseq OG relationship comparison

This experiment runs current and historical Logseq file parsers on the shared Markdown fixtures. Read [the findings](findings.md) for the results and their implications for page lookup.

## Run the comparison

The existing [logseq/logseq ghq checkout](https://github.com/logseq/logseq) must contain both revisions in [sources.json](sources.json). From the gardener repository root:

```sh
mise run parser:og-check
```

The task installs locked npm dependencies and extracts committed graph-parser, database, and common source into ignored `node_modules/sources/` directories. Setup leaves the upstream checkout unchanged and fails if a required revision is unavailable. It performs no clone or fetch.

After an intentional source or fixture change:

```sh
mise run parser:og-update
```

Review the diff in [relationships.json](relationships.json), including input hashes and relationships, before accepting the update. These research commands run locally; project CI continues checking the independent mldoc baseline.

## Inputs and saved results

[compare.mjs](compare.mjs) reads the [baseline garden](../../../parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden) and [focused relationship fixtures](../../../parser-comparison-evidence/experiments/parser-comparison/fixtures). It creates default and triple-lowbar-configured gardens from those notes in sorted file order. Each source revision runs in a fresh nbb-logseq process through [graph.cljs](graph.cljs).

The saved results contain page names, original names, relative file locations, aliases, namespace parents, publication properties, journal flags and dates, note content, parents, previous siblings, and references. Generated block UUIDs become deterministic labels; explicit UUID properties and unresolved UUID reference targets remain visible. Built-in pages remain in the results. The earlier snapshot comparison omits the newly recorded journal flag and date fields, then compares the same page and block fields as the earlier experiment.

The current and historical runs share mldoc 1.5.7, the version declared by current Logseq OG. The historical source declares a compatible range beginning at 1.5.1. Comparing the historical run with the earlier mldoc 1.5.9 snapshot separately checks whether this runtime change affects the recorded fixture results. The locked nbb-logseq version is 1.2.173; it provides the ClojureScript runtime and database dependencies.

The historical graph-validator remains part of the [earlier evidence](../../../parser-comparison-evidence/experiments/parser-comparison/findings.md). This experiment compares the selected database fields directly and runs no validator assertions against current Logseq OG.

## Sources and licenses

[sources.json](sources.json) identifies the source revisions. The [current Logseq OG file branch](https://github.com/logseq/logseq/tree/6efedb75588763af256bc7dfd0ed5526dc91fe7c) and [historical Logseq source](https://github.com/logseq/logseq/tree/82cf4d3c65acbf230a3170640fe271dd74095067) use AGPL-3.0 with an additional permission. The installed nbb-logseq package declares MIT. The mldoc npm package declares ISC; that declaration alone does not establish the license of its bundled parser implementation. Upstream implementations stay in ignored extracted directories. Parser adoption still requires the license review tracked in [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3).
