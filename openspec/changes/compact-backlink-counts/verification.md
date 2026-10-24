## Verification

- Twelve browser-controller tests pass, including equal totals, one page with several contexts, zero results, filtering/reset, preserved collapse state and repeated initialization.
- Eight publication fixtures pass, including initial hidden detail, plural labels, compact instruction, aliases, privacy and one-page/two-context output.
- Full `mise run ci` passes, including Rust/Node tests, release build, strict specifications, RuleSync, docs and release checks.
- The product diff changes count labels/visibility and instruction text only. Index construction and include/exclude predicates are unchanged.
- Independent review is pending. The separate specification cleanup is tracked in issue97 after implementation merge.
