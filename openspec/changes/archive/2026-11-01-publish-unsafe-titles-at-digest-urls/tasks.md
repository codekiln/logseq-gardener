## 1. Reproduce

- [x] 1.1 Add `unsafe_titles_keep_their_digest_file` to `crates/logseq-gardener-sdk/tests/site_readable.rs`, with a page stored as `Docs___Q___Rename with %2Frename, then resume%3F.md` and a page titled `Docs/A\B`, and see it fail with `PageUrl(Unsafe { .. })`
- [x] 1.2 Publish the encode garden with `--readable-urls` using an `lsg` built from `origin/main`, and record the failure in `verification.md`

## 2. Publication

- [x] 2.1 Make `route_for` in `crates/logseq-gardener-sdk/src/publication.rs` return the digest route and the reason for a title that cannot form a safe path
- [x] 2.2 Record a `Notice` for each such page in `select_with_urls`, and pass only folder routes to `check_collisions`
- [x] 2.3 Add `cli_readable_urls_publish_an_unsafe_title_at_its_digest_file` to `tests/publish.rs` for the exit code and the diagnostic on standard error

## 3. Browser scripts

- [x] 3.1 Make the readable graph script in `crates/logseq-gardener-sdk/src/site/readable-graph.js` return `p-<digest>.html` for a title that cannot name folders, and let the page graph find a page from its digest file
- [x] 3.2 Make the readable search engine pattern in `crates/logseq-gardener-sdk/src/site/readable.rs` accept digest files and folder routes
- [x] 3.3 Extend `scripts/readable/browser.py` with a fallback page: link, search result, page graph, graph route and plugin host navigation

## 4. Docs and help

- [x] 4.1 Describe titles that keep their digest file in the Readable URLs section of `docs/pages/lsg___Ref___Publish Options.md`
- [x] 4.2 Update the `--readable-urls` behavior text in `src/help.rs`, and record the review in `docs/pages/lsg___How To___Review Help Examples.md`

## 5. Verification

- [x] 5.1 Publish the encode garden with `--readable-urls` using this branch, and record the result in `verification.md`
- [x] 5.2 Run `mise run readable:browser-test`
- [x] 5.3 Run `mise run ci`
