# Verification: fold-case-folded-namespace-folders

A readable build of the encode garden completes. Pages under a folder that titles spell two ways are published under one spelling, and links between them resolve.

Checked on 2026-11-01 against the local checkout of codekiln/logseq-encode-garden, with the `mise-tasks/site/build` script from its branch `claude/295-readable-urls`, which passes `--readable-urls --journals --search --timings` and includes every namespace root.

## Before: `lsg` v0.0.11

The build stops and writes no output:

```
error: publication failed: pages "CLAUDE" and "Claude/Artifact" cannot both have readable URLs: their folders "CLAUDE" and "Claude" differ only in letter case or Unicode normalization
```

## After: `lsg` built from this branch

```
Published 8926 pages, 513 journals and 111 assets from 8929 page files; withheld 2; 36358 diagnostics.
```

- `lsg-site.json` lists 13,905 pages, journals and namespace pages. 240 of them use a respelled folder in their URL, such as `Claude/Artifact` at `CLAUDE/Artifact/` and `brew/Q` at `Brew/Q/`.
- The graph script holds 258 folder spellings.
- `My/AI/Rule/index.html`, `Person/Andrew Huberman/index.html` and `CLAUDE/index.html` exist with their titles.
- Every relative link on 300 pages resolves to a file in the output: 200 pages chosen at random and 100 respelled pages, 12,905 links in all.
- The three unsafe-title diagnostics from issue #363 are still the only `cannot have a readable URL` diagnostics.
