# Current Logseq OG relationships

Current Logseq OG and the historical parser produce equal saved page and block relationships on the shared fixtures, under both default and triple-lowbar filename configuration. The historical run with mldoc 1.5.7 also matches the earlier mldoc 1.5.9 snapshot for its recorded fields. [relationships.json](relationships.json) contains the source revisions, runtime versions, input hashes, and full results; [the experiment guide](README.md) explains how to reproduce them.

## Page lookup

The results preserve the earlier SDK requirements:

| Fixture behavior | SDK responsibility |
| --- | --- |
| `Names___Nested.md` keeps underscores by default and becomes `Names/Nested` with triple-lowbar configuration. | Derive page identity using graph configuration. |
| `title:: Chosen Name` overrides the filename. | Keep page identity separate from file location. |
| `Other Name` points to both `Alias Source` and `Chosen Name`. | Retain alias candidates and report ambiguity when lookup needs a unique file. |
| `Missing Target` exists as a linked name without a source file. | Distinguish known names from stored pages. |
| The journal file resolves to `Oct 3rd, 2026`, with a journal flag and date. | Apply Logseq journal naming rules. |
| Nested notes retain parent and previous-sibling relationships; UUID links point to the explicit target or retain the missing UUID. | Preserve hierarchy and unresolved reference diagnostics. |
| Publication properties remain properties, and fenced reference-like text contributes no references. | Respect literal syntax and keep publication policy separate from name resolution. |

The current Logseq source requires no change to these fixture-based page lookup rules. A future SDK implementation should compare its results with these recorded relationships before treating the rules as implemented.

## Coverage limits

Agreement covers the selected page and block fields for the shared Markdown fixtures. Generated UUIDs are normalized, built-in pages remain visible, and explicit UUID properties are retained. The experiment compares graph relationships directly; current graph-validator assertions, public-garden graph construction, query evaluation, publication filtering, and formatting-preserving edits remain outside this evidence.

The syntax recommendation in [the parser findings](../../../parser-comparison-evidence/experiments/parser-comparison/findings.md) continues to favor lsdoc for read-only parsing with Logseq relationship rules in a separate SDK layer. Pengx's publishing corpus and the final adoption/license decision remain in [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3). [PR #8 — Correct Org references](https://github.com/codekiln/logseq-gardener/pull/8) separately repairs the public-garden comparison runner.
