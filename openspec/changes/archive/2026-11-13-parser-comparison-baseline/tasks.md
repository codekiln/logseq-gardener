## 1. Fixture garden

- [x] 1.1 Add short synthetic Markdown files for aliases, page references, namespaces, nested blocks, tags, media, and reference-like code text.
- [x] 1.2 Document how the cases relate to the local Logseq source and identify its tested revision.

## 2. Reference output

- [x] 2.1 Pin the Logseq-used `mldoc` version in an isolated npm experiment with a lockfile.
- [x] 2.2 Add a runner that checks raw parse and reference output and detects changed Markdown fixtures.
- [x] 2.3 Generate and inspect the committed baseline output.

## 3. Verification and handoff

- [x] 3.1 Run the experiment check and strict OpenSpec validation.
- [x] 3.2 Document the remaining candidate-parser, graph, and text-preservation work under [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3).
- [x] 3.3 Run the baseline check through `mise run ci` in GitHub Actions and verify the full local task.
