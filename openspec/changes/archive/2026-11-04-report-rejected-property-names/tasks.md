## 1. Reproduce

- [x] 1.1 Add `site_properties` tests that expect the diagnostic for a block, for page properties with two names and for an embedded block, and see them fail on `origin/main`

## 2. Report the diagnostic

- [x] 2.1 Record the diagnostic in `Renderer::invalid_properties`; verify with the tests from 1.1 and the existing warning tests in `site_properties`
- [x] 2.2 List the diagnostic under "Differences from Logseq" in `docs/pages/lsg___Ref___Logseq Compatibility___Page Outline.md`

## 3. Verification

- [x] 3.1 Publish the encode garden at `cbab08523e` and at `9860577214` with this branch, count the diagnostics, and record the result in `verification.md`
- [x] 3.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `published-page-outline` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #474.
