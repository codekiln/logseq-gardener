## 1. Render copies with gaps

- [x] 1.1 Make `render::render_reference_context` return a `ReferenceContext` that leaves a gap for each copied bullet's row attribute and bullet, and fills them with `ReferenceContext::fill`
- [x] 1.2 Assert in debug builds that a render with gaps binds no recordings, since linking dialogue timestamps would move the gaps

## 2. Reuse copies across listings

- [x] 2.1 Keep each block's rendered copy, by block and open depth, and each block's breadcrumb in `References`, and render them only on first use
- [x] 2.2 Fill each listing's row numbers and bullet links in `group`, and report the copy's diagnostics for each listing as before
- [x] 2.3 Use the same copy path for flashcard fronts and backs with empty fills

## 3. Verify

- [x] 3.1 Add a test that lists one block on two pages and checks each page's row numbers and bullet links
- [x] 3.2 Publish the encode garden and the generated Gitpa-sized garden with the `main` binary and the changed binary, and confirm identical files and diagnostics
- [x] 3.3 Run `mise run parity:publish-bench --corpus all --record` and add the rows to `docs/parity/publication-speed.json`
- [x] 3.4 Update `docs/parity/publication-speed.md` with the new measurements
- [x] 3.5 Run `mise run ci`
