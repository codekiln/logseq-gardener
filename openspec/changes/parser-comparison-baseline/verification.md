# Parser comparison baseline verification

- `npm test` matches the saved `mldoc` 1.5.9 parse and reference output for all six fixtures.
- Adding a newline to one fixture makes `npm test` fail and name that file; restoring the fixture makes it pass again.
- `mise run parser:baseline-check` installs the locked experiment dependencies and passes.
- `mise run ci` passes with the new baseline check included.
- Strict OpenSpec change validation and main spec validation pass.

This verifies a syntax reference baseline only. It does not compare `lsdoc`, run Logseq's graph-parser or graph-validator, or establish graph behavior and text-preserving writes. Issue #3 remains open for those results and a parser decision.
