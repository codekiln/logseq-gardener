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

The runner reuses the [mldoc baseline fixtures](../../../parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden) and adds [focused graph cases](fixtures/pages) for title overrides, shared aliases, publication properties, queries, and missing UUID references. Each syntax comparison uses a fresh mldoc process. The graph parser receives the files in a fixed order through `parse-graph`'s `:files` option. The snapshot includes both default filename interpretation and `:file/name-format :triple-lowbar`.

Graph output replaces generated UUIDs with deterministic labels while retaining explicit UUID targets, page names, file paths relative to the garden, parent and previous-sibling relationships, properties, aliases, and references. Built-in Logseq pages remain visible. Validator results contain assertion counts and the names of failing tests. Missing targets and the absent sketch asset intentionally cause validation failures; those outcomes are checked against the snapshot.

After an intentional change, run `mise run parser:comparison-update` and review the snapshot diff. These source-dependent research tasks run locally; existing GitHub CI continues checking the independent mldoc baseline.

## Reproduce public garden results

[corpora.json](corpora.json) pins the public repositories and their revisions. The wrapper extracts committed `pages/`, `journals/`, and graph configuration into an ignored directory, then runs lsdoc's upstream `graph-check.mjs` with fresh mldoc processes, journals enabled, and Markdown and Org enabled. It records the corpus hash and replaces the runner's incorrectly inferred lsdoc revision with the pinned source revision.

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

[sources.json](sources.json) pins lsdoc 0.5.8, Logseq 0.9.8's file parser, and graph-validator's last file-compatible first-parent revision before its database migration. Its declared dependency is the tested Logseq revision. The classpath uses only those archived sources and the npm-locked nbb-logseq 1.2.173 runtime. It omits upstream test-runner dependencies because the experiment invokes the parser and validator directly.

lsdoc is AGPL-3.0-only; Logseq is AGPL-3.0 with an additional permission; graph-validator's LICENSE.md is MIT, despite its package.json's ISC field. The installed mldoc npm package declares ISC; that declaration alone does not establish the license of its bundled parser implementation. License verification remains part of any adoption change. The docs garden is MIT. Encode Garden has no repository-wide license declaration; its complete source stays in the local archive. No upstream implementation is copied into tracked experiment files.

The extracted lsdoc Cargo manifest receives an empty `[workspace]` table to isolate it from gardener's workspace. Parser code and its Cargo.lock remain unchanged.
