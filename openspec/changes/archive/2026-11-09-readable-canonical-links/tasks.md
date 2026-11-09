## 1. Canonical links in readable pages

- [x] 1.1 Write `<link rel="canonical">` after `<base>` on every page, journal, namespace page and reference-only page of a readable build, `./` for the home page, and verify with unit tests in `site/readable.rs`
- [x] 1.2 Add fixture tests in `crates/logseq-gardener-sdk/tests/site_readable.rs` for a folder page, a digest-file page, the home page, the root lists and a default build, and verify they pass with `cargo test`

## 2. Plugin host navigation

- [x] 2.1 Patch the readable host to skip `link[rel=canonical]` when copying head elements and to replace it after adopting a page, and verify the patch lines exist once and appear in a readable build's host

## 3. Help and documentation

- [x] 3.1 Add the canonical link to the `--readable-urls` help text and record the review in `lsg/How To/Review Help Examples`; verify `mise run clilint:check` passes
- [x] 3.2 Describe the link in the Readable URLs section of `lsg/Ref/Publish Options` and the readable URL row of `docs/parity/gitp-a-shared-rendering.md`; verify `mise run docs:check` passes

## 4. Integration

- [x] 4.1 Publish a few logseq-encode-garden pages with and without `--readable-urls`, check the links, and verify `mise run ci` passes

## Workflow follow-up

- File the cleanup sub-issue under #589 after opening the implementation pull request.
- After merge, sync the delta into `openspec/specs/readable-page-urls/spec.md` and archive the change.
