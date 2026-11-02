## 1. Reproduce

- [x] 1.1 Run `mise run parity:logseq-export` on a test garden with each case from the issue and record the rows only Logseq shows
- [x] 1.2 Add an SDK test of the `alias` row and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `page_names::AliasGroups`, which lists the names of a page's alias group and the page each one opens, with tests
- [x] 2.2 Build the alias groups of the published pages once and render each `alias` row from them; keep the `aliases` row as it was

## 3. Verification

- [x] 3.1 Rerun the Logseq export comparison on the test garden, and publish the encode garden with page properties on with both builds; record the results in `verification.md`
- [x] 3.2 Run `mise run ci`
