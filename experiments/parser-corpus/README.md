# Parser comparison on an unlicensed corpus

This one-time experiment compares parsers on [Pengx's knowledge garden](https://github.com/pengx17/knowledge-garden) at the revision in [sources.json](sources.json). That garden has no license, so this repository keeps only the script and [aggregate results](results.json). [Parser adoption](../../docs/parser-adoption.md) explains what the results mean.

## What it compares

[compare.mjs](compare.mjs) reads the garden's committed `pages/`, `journals/`, `assets/` and `logseq/config.edn` at the pinned revision and runs four checks:

1. **Syntax and references.** lsdoc's garden runner compares lsdoc with mldoc 1.5.9 on every note, starting a fresh mldoc process for each note. It applies the same Org reference repair as the [earlier comparison](../../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/README.md).
2. **Embedded media and publication properties.** The script counts images, asset links, page and block embeds, media macros, media HTML and page or block properties in each parser's output, then checks that the counts agree file by file.
3. **Logseq relationships.** The current Logseq OG file-graph parser builds the graph with the garden's own configuration, using the [relationship script](../../openspec/changes/current-og-relationships/experiments/file-relationships/graph.cljs) and runtime from the earlier OG comparison. For each file, the script compares the pages that Logseq links with the page references lsdoc reports, and sorts the differences into categories.
4. **Gardener end to end.** `lsg publish --journals` publishes every root namespace, and the script counts the published pages and diagnostics by kind.

## Run it

```sh
cargo build --release --bin lsg
node experiments/parser-corpus/compare.mjs --scratch /private/tmp/parser-corpus \
  --lsdoc-repo PATH --logseq-repo PATH
```

`--scratch` must be outside this repository and the corpus checkout. Every per-file output, including the runner's report, goes there and must not be committed. The script never clones or fetches; it extracts sources with `git archive`, which leaves each checkout unchanged. Without the repository options, it finds lsdoc and Logseq through ghq. The recorded run used the lsdoc revision from the local Cargo git cache and the `version/file` branch of the local [logseq/og](https://github.com/logseq/og) checkout.

The run takes a few minutes, mostly to build lsdoc. It is not part of CI.
