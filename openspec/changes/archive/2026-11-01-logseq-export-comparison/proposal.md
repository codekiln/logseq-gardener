## Why

The parser check from [PR #358](https://github.com/codekiln/logseq-gardener/pull/358) compares how Logseq's file parser and the SDK read a garden's files. It cannot see what a reader sees on a page. For [Issue #369](https://github.com/codekiln/logseq-gardener/issues/369#issuecomment-6561215901), Logseq 0.10.15's own static export of the docs graph was compared with the `lsg` site by hand-run scripts, and the comparison found [Issue #385 Show alias, title and public in the page properties box as Logseq does](https://github.com/codekiln/logseq-gardener/issues/385). [Issue #388 Compare Logseq's own export of a garden with the lsg site](https://github.com/codekiln/logseq-gardener/issues/388) asks for that comparison as a repeatable mise task.

## What Changes

- A new task, `mise run parity:logseq-export`, builds Logseq's static export of a garden, the docs graph by default, with Logseq 0.10.15's `logseq.publishing/export`, the code `logseq/publish-spa` wraps. The export code comes from tag `0.10.15` of logseq/logseq, from a local ghq checkout or fetched from GitHub, checked against a pinned digest and cached. The built publishing frontend comes from a folder passed with `--static-dir`, from Logseq.app on macOS, or from the official 0.10.15 release archive, and must match a pinned digest.
- The task builds the same garden with the checkout's `lsg`, with the options `docs:site` uses for the docs graph or the `lsg publish` options passed to the task for another garden.
- It serves both builds from one local server and reads every page in headless Chromium: the rendered text of the blocks and the rows of the page properties box. It reports, per page, the words only one build shows and the property rows only one build shows, writes the report as JSON and Markdown under `target/parity/<UTC time>/`, and exits 1 when any page differs.
- The comparison leaves out Logseq's reference-count badge, waits for Logseq's lazily rendered blocks, and reads text the way the browser lays it out, so a word that syntax highlighting splits across elements stays one word. Unit tests cover the word and property comparison, and small HTML fixture pages check the page reading in Chromium.
- The task stays out of `mise run ci`, like the other browser parity tasks. Its unit tests run in `mise run test`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parity-comparison-harness`: new requirements "Build Logseq's own export of a garden", "Build the same garden with lsg for the export comparison" and "Compare rendered pages with Logseq's export"; "Test the harness logic in CI" also covers the export comparison.

## Impact

- `.mise/tasks/parity/logseq-export`: the new task.
- `scripts/parity/logseq_export.py`: builds both sites, reads the pages and writes the report.
- `scripts/parity/logseq-export/`: `source.json` with the pinned Logseq source and frontend, and `package.json` with `package-lock.json` for `@logseq/nbb-logseq`, `mldoc` and `fs-extra`.
- `scripts/parity/test_logseq_export.py` and fixture pages under `scripts/parity/testdata/logseq-export/`.
- Docs: `lsg/Ref/Parity Tasks`, `lsg/How To/Compare a Build with Logseq` and `lsg/Ref/Logseq Compatibility/Comparison Harness`.
- Disk and network: the 30 Logseq source files, about 200 KB, are cached under `~/.cache/logseq-gardener/`. Without Logseq.app 0.10.15, the first run downloads a 202 MB release archive and caches the 130 MB frontend from it.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
