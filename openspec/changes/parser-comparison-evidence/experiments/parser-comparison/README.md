# Parser comparison

This experiment compares lsdoc with mldoc on shared Markdown fixtures and pinned public gardens. It also records how Logseq's file parser turns the fixtures into pages, aliases, references, and nested notes. Read [the findings](findings.md) for the recommendation and its limits.

## Run the fixture check

The following ghq checkouts must contain the revisions in [sources.json](sources.json):

- [martinkoutecky/lsdoc](https://github.com/martinkoutecky/lsdoc)
- [logseq/logseq](https://github.com/logseq/logseq)
- [logseq/graph-validator](https://github.com/logseq/graph-validator)

From the gardener repository root:

```sh
mise run parser:comparison-check
```

The task installs the locked npm dependencies, extracts pinned source into ignored `node_modules/sources/`, builds lsdoc, and compares [fixtures.json](fixtures.json). Source archives leave each checkout unchanged. A missing repository or revision fails setup; the task never clones or fetches.

[compare.mjs](compare.mjs), run by `mise run parser:comparison-check`, compares the parsers on the [mldoc baseline test notes](../../../parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden) and [additional graph test notes](fixtures/pages). The additional notes cover title overrides, shared aliases, publication properties, queries, and missing UUID references. Each syntax comparison starts a fresh mldoc process. Logseq's graph parser receives the notes in a fixed order. The saved results include both default filename interpretation and the configuration that turns triple underscores into namespace separators.

Graph output replaces generated UUIDs with deterministic labels while retaining explicit UUID targets, page names, file paths relative to the garden, parent and previous-sibling relationships, properties, aliases, and references. Built-in Logseq pages remain visible. Validator results contain assertion counts and the names of failing tests. Missing targets and the absent sketch asset intentionally cause validation failures; those outcomes are checked against the snapshot.

After an intentional change, run `mise run parser:comparison-update` and review the snapshot diff. These source-dependent research tasks run locally; existing GitHub CI continues checking the independent mldoc baseline.

## Reproduce public garden results

[garden.mjs](garden.mjs), run by `mise run parser:garden-comparison`, extracts the committed notes and graph configuration from the revisions in [corpora.json](corpora.json). It compares Markdown and Org pages and journals using [lsdoc's garden comparison script](https://github.com/martinkoutecky/lsdoc/blob/32e63ef095c711d6d9947257bf5fd07d540fa59d/tools/graph-check.mjs), starting a fresh mldoc process for each comparison. The report records the input hash and the pinned lsdoc revision.

```sh
mise run parser:garden-comparison docs
mise run parser:garden-comparison encode
```

The matching ghq gardens must contain the recorded commits. Reports contain measurements, source-file locations, and anonymized inputs rechecked by the upstream runner. Timings and report timestamps vary between runs. The corpus hash identifies the source inputs; deterministic fixture results are checked separately.

`regressions.json` contains small synthetic reproductions of the findings. The fixture check records the intentional math difference and tests Org reference extraction with the format argument supplied.

## Source preservation and memory

The Rust probe in `probe/` calls lsdoc's stable `parse_outline` API, checks that each range slices valid UTF-8, and reassembles the retained source around the accepted header lines. It includes Unicode, CRLF, an unterminated final line, and reference-like text inside a fence. The snapshot also records mldoc's `parseAndExportMarkdown` output with property retention enabled under the configuration in `oracle.mjs`. Source reassembly is a test of retaining original text; it is not an AST serializer or an editing implementation.

On macOS, after reproducing the docs report:

```sh
cd openspec/changes/parser-comparison-evidence/experiments/parser-comparison
mise exec -- node memory.mjs
```

`memory.json` records peak resident memory from `/usr/bin/time -l` for separate whole-corpus processes. The measurement includes input, syntax output, serialization, and the language runtime. It uses the docs corpus and one run per parser; it is not a parser-only allocation measurement.

## Sources and licenses

[sources.json](sources.json) records the tested versions of lsdoc, Logseq's file parser, and graph-validator. We use the last graph-validator revision that supports file gardens and the Logseq 0.9.8 revision it depends on. The experiment runs those archived sources with the locked nbb-logseq dependency, which runs ClojureScript in Node.js, and invokes the parser and validator directly.

lsdoc is AGPL-3.0-only; Logseq is AGPL-3.0 with an additional permission; graph-validator's LICENSE.md is MIT, despite its package.json's ISC field. The installed mldoc npm package declares ISC; that declaration alone does not establish the license of its bundled parser implementation. License verification remains part of any adoption change. The docs garden is MIT. Encode Garden has no repository-wide license declaration; its complete source stays in the local archive. No upstream implementation is copied into tracked experiment files.

The extracted lsdoc Cargo manifest receives an empty `[workspace]` table to isolate it from gardener's workspace. Parser code and its Cargo.lock remain unchanged.
