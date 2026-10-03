# Parser comparison baseline verification

- `mise run parser:baseline-check` matches the saved `mldoc` 1.5.9 parse and reference output for all six fixtures.
- Adding a newline to one fixture makes `mise run parser:baseline-check` fail and name that file; restoring the fixture makes it pass again.
- `mise run ci` passes with the parser baseline check included.
- Strict OpenSpec change validation and main spec validation pass.

This verifies a syntax reference baseline only. It does not compare `lsdoc`, run Logseq's graph-parser or graph-validator, or establish graph behavior and text-preserving writes. Issue #3 remains open for those results and a parser decision.
