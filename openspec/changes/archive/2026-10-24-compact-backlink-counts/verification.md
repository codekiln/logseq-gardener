## Verification

- Twelve browser-controller tests pass, including equal totals, one page with several contexts, zero results, filtering/reset, preserved collapse state and repeated initialization.
- Eight publication fixtures pass, including initial hidden detail, plural labels, compact instruction, aliases, privacy and one-page/two-context output.
- Full `mise run ci` passes, including Rust/Node tests, release build, strict specifications, RuleSync, docs and release checks.
- The product diff changes count labels/visibility and instruction text only. Index construction and include/exclude predicates are unchanged.
- Root independently reviewed the product diff and cleared its count visibility, filtered totals, zero state and unchanged matching/index behavior. [PR98](https://github.com/codekiln/logseq-gardener/pull/98) merged after core/custom checks and the corrected title passed. The separate specification cleanup is tracked in issue97.
