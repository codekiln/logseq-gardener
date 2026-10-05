# lsdoc graph check report

Generated: 2026-10-05T16:57:23.261Z
Graph: `github.com/logseq/docs` at `08f855f24d66e4509b7ea808554c13b4649e6ee1`
Mode: `both`, format: `auto`, journals: `on`, jobs: `4`, timeout: `10000ms`
lsdoc source revision: `32e63ef095c711d6d9947257bf5fd07d540fa59d`
Local runner repair: Pass each note format to mldoc reference extraction: extractRefs(ast, format).
mldoc npm version: `1.5.9`

Privacy: nothing was uploaded. Temporary parser inputs were kept in a fresh mode-0700 temp directory and removed on exit. This report is the only persistent output; snippets below are anonymized and re-verified, or omitted.

## Graph stats

- Matched files: 333
- Total bytes: 582635
- Largest file: `pages/Changelog.md` (195020 bytes)
- Skipped files over 8 MB: 0

## Bench

Fairness notes: mldoc here is the npm js_of_ocaml build used by Logseq/Electron, so it is the real-world shipped comparison, but it is not native OCaml. Each side ran 3 times; totals below are best-of-3 parse time sums, excluding crashed/timed-out files. Per-file values are from parser-reported in-process parse timings.

### lsdoc

- Parsed files in aggregate: 333
- Best total: 9.818 ms
- p50 / p95 / max: 0.004 / 0.070 / 3.768 ms
- 5 slowest files:
  - `pages/Changelog.md` (3.768 ms)
  - `pages/changelog_06.md` (0.687 ms)
  - `pages/Changelog_07_09.md` (0.458 ms)
  - `pages/Changelog_2020.org` (0.410 ms)
  - `pages/Advanced Queries.md` (0.234 ms)

### mldoc

- Parsed files in aggregate: 333
- Best total: 778.372 ms
- p50 / p95 / max: 0.299 / 5.132 / 246.907 ms
- 5 slowest files:
  - `pages/Changelog.md` (246.907 ms)
  - `pages/changelog_06.md` (73.238 ms)
  - `pages/Changelog_2020.org` (55.982 ms)
  - `pages/Changelog_07_09.md` (51.404 ms)
  - `pages/Advanced Queries.md` (23.018 ms)

## Diff findings

No divergences, crashes, or timeouts found.
