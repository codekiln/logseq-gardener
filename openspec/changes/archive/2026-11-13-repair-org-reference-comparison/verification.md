# Org comparison verification

The public-garden worker now passes the note format to reference extraction. The repair checks the expected upstream call before changing extracted source, and generated reports name the repair.

- `mise run parser:comparison-check` passes the saved syntax, graph, source-preservation, and direct regression comparisons, plus the new Markdown/Org runner check.
- Removing `format` from the extracted worker makes `runner-check.mjs` fail on the Org fixture. The repaired worker passes after restoration.
- `mise run parser:garden-comparison docs` completes with the existing pinned corpus and unchanged corpus hash. Its report contains no normalized syntax/reference disagreements, crashes, or timeouts.
- `openspec validate repair-org-reference-comparison --strict` and `mise run ci` pass.
- The corpus task's usage metadata now accepts the documented `docs` argument; the earlier invalid choices syntax left `usage_garden` unset.

All planned tasks and requirement scenarios are covered. Current Logseq OG relationships and Pengx's publishing corpus remain pending in [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3), so this repair does not complete the parser adoption decision.
