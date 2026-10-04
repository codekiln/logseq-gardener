# lsdoc graph check report

Generated: 2026-10-04T18:53:16.922Z
Graph: `github.com/logseq/docs` at `08f855f24d66e4509b7ea808554c13b4649e6ee1`
Mode: `both`, format: `auto`, journals: `on`, jobs: `4`, timeout: `10000ms`
lsdoc source revision: `32e63ef095c711d6d9947257bf5fd07d540fa59d`
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
- Best total: 10.357 ms
- p50 / p95 / max: 0.004 / 0.079 / 3.852 ms
- 5 slowest files:
  - `pages/Changelog.md` (3.852 ms)
  - `pages/changelog_06.md` (0.706 ms)
  - `pages/Changelog_07_09.md` (0.509 ms)
  - `pages/Changelog_2020.org` (0.444 ms)
  - `pages/Advanced Queries.md` (0.255 ms)

### mldoc

- Parsed files in aggregate: 333
- Best total: 780.488 ms
- p50 / p95 / max: 0.307 / 5.344 / 261.304 ms
- 5 slowest files:
  - `pages/Changelog.md` (261.304 ms)
  - `pages/changelog_06.md` (71.712 ms)
  - `pages/Changelog_07_09.md` (49.445 ms)
  - `pages/Changelog_2020.org` (48.055 ms)
  - `pages/Advanced Queries.md` (24.467 ms)

## Diff findings

2 finding(s). File paths are relative to the graph root.

### Finding 1: divergence

File: `journals/2020_05_20.org`
Local range: lines 3-3
Snippet status: fresh reproducible divergence derived from your page via tier 1. This is the anonymized input's own parser output, not the original page projection.

Anonymized snippet:
```
*** Aaa [[aaaa/aaaaa][Aaaa aaaaa]] (Aaaa aaaaaaaaaa aaa aaaa aaa aaa).

```

Visible JSON string: `"*** Aaa [[aaaa/aaaaa][Aaaa aaaaa]] (Aaaa aaaaaaaaaa aaa aaaa aaa aaa).\n"`

mldoc projection: `{"blocks":[{"inline":[{"k":"plain","text":"Aaa "},{"full":"[[aaaa/aaaaa][Aaaa aaaaa]]","k":"link","label":[{"k":"plain","text":"Aaaa aaaaa"}],"url":{"type":"search","v":"aaaa/aaaaa"}},{"k":"plain","text":" (Aaaa aaaaaaaaaa aaa aaaa aaa aaa)."}],"kind":"bullet","level":3}],"refs":{"block":[],"page":[]}}`
lsdoc projection: `{"blocks":[{"inline":[{"k":"plain","text":"Aaa "},{"full":"[[aaaa/aaaaa][Aaaa aaaaa]]","k":"link","label":[{"k":"plain","text":"Aaaa aaaaa"}],"url":{"type":"search","v":"aaaa/aaaaa"}},{"k":"plain","text":" (Aaaa aaaaaaaaaa aaa aaaa aaa aaa)."}],"kind":"bullet","level":3}],"refs":{"block":[],"page":["aaaa/aaaaa"]}}`

Post this anonymized, re-verified snippet to https://github.com/martinkoutecky/lsdoc/issues

### Finding 2: divergence

File: `pages/Changelog_2020.org`
Local range: lines 52-52
Snippet status: fresh reproducible divergence derived from your page via tier 1. This is the anonymized input's own parser output, not the original page projection.

Anonymized snippet:
```
**** [[aaaa/aaaaa][Aaaa aaaaa]] aaaaaaaaaaa aaaaaaa

```

Visible JSON string: `"**** [[aaaa/aaaaa][Aaaa aaaaa]] aaaaaaaaaaa aaaaaaa\n"`

mldoc projection: `{"blocks":[{"inline":[{"full":"[[aaaa/aaaaa][Aaaa aaaaa]]","k":"link","label":[{"k":"plain","text":"Aaaa aaaaa"}],"url":{"type":"search","v":"aaaa/aaaaa"}},{"k":"plain","text":" aaaaaaaaaaa aaaaaaa"}],"kind":"bullet","level":4}],"refs":{"block":[],"page":[]}}`
lsdoc projection: `{"blocks":[{"inline":[{"full":"[[aaaa/aaaaa][Aaaa aaaaa]]","k":"link","label":[{"k":"plain","text":"Aaaa aaaaa"}],"url":{"type":"search","v":"aaaa/aaaaa"}},{"k":"plain","text":" aaaaaaaaaaa aaaaaaa"}],"kind":"bullet","level":4}],"refs":{"block":[],"page":["aaaa/aaaaa"]}}`

Post this anonymized, re-verified snippet to https://github.com/martinkoutecky/lsdoc/issues
