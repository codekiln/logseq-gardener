# mldoc syntax baseline

This experiment records how `mldoc` parses six small Markdown files. The files are synthetic, so the comparison can be shared without copying anyone's notes. They cover aliases and tags, page and block references, nested blocks, namespace-like filenames, a journal, an image, an embed, and reference-like text inside a code fence.

The local [`logseq/logseq` source at `63b76c5`](https://github.com/logseq/logseq/tree/63b76c5db4e7b710551c8c1ba7a4f00e5b94bb0b/deps/graph-parser) declares `mldoc` `^1.5.9` in `deps/graph-parser/package.json`. Its `mldoc_test.cljs`, `text_test.cljs`, and `block_test.cljs` cover property values, page references, and nested names. This experiment pins `mldoc` 1.5.9 exactly so a later parser can be tested against one stable syntax reference. The cases are new examples, not copies of Logseq test data. The source revision identifies the code consulted while choosing them; it does not mean that Logseq's graph-parser ran here.

Logseq's checked-out `LICENSE.md` is AGPLv3 with an additional permission. The installed `mldoc` 1.5.9 package declares ISC in its `package.json`; its exact npm archive is fixed by `package-lock.json`. No upstream code or third-party notes are copied into this experiment.

## Run

From the repository root:

```sh
mise run parser:baseline-check
```

The task uses Node 24.14.0 from `mise.toml`, installs the locked experiment dependencies, and checks the snapshot. GitHub Actions runs the same check through `mise run ci`.

`compare.mjs` reads every Markdown file under `fixtures/garden`, calls `mldoc`'s `parseJson` and `getReferences` with the same Markdown configuration used by Logseq's wrapper, and compares the result with `mldoc-1.5.9.json`. The snapshot contains each file's SHA-256, raw syntax tree, and raw reference output. A changed input or parser result fails the check. After an intentional fixture or version change, run `mise run parser:baseline-update` from the repository root and review the JSON diff.

## What this does not establish

The raw `mldoc` result does not resolve aliases, decide whether a page exists, build Logseq's graph, or prove that a note can be rewritten without changing unrelated text. A Markdown image path in the fixture is intentionally not backed by an asset; this baseline checks parsing, not asset loading. The namespace-like filename is recorded as input, but `mldoc` receives file contents and does not interpret that filename.

We will test `lsdoc`, `mldoc`, Logseq's graph parser, and graph-validator against the same examples and public gardens under [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3). That comparison will record differences, performance, and text-preservation results before choosing the SDK parser. [Issue #4 Add the first read-only garden command through the SDK](https://github.com/codekiln/logseq-gardener/issues/4) follows the parser decision.
