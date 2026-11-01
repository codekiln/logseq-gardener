## 1. Reproduce

- [x] 1.1 Add an SDK test for a block listed in another page's Linked References with a missing page link and an asset outside the garden; see each problem reported twice
- [x] 1.2 Change the embed test to expect one diagnostic for a missing image, under the embedded file; see it reported three times

## 2. Fix

- [x] 2.1 Add `page_diagnostics`, which moves a borrowed diagnostic to its file and keeps it once unless that file reports it
- [x] 2.2 Record the publish loop's page diagnostics through it

## 3. Verification

- [x] 3.1 Publish a copy of `docs` with a broken page, with and without `--homepage`, before and after; record the reports in `verification.md`
- [x] 3.2 Publish `docs` and the encode garden before and after, and compare the published files and the diagnostic counts
- [x] 3.3 Run `mise run ci`
