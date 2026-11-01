## 1. Baseline

- [x] 1.1 Build `lsg` from `main`, keep the binary as the baseline, and publish the encode garden at the CI-pinned commit with `--journals --search` and every top-level namespace; record each part's file count and size
- [x] 1.2 Take `parity:screenshots` of the fixture pages in both appearances with the baseline build, for the look comparison in 6.2

## 2. Shared stylesheets

- [x] 2.1 Write the built-in page styles once as `lsg-site.css` and link it from each page where the inline `<style>` block was, before the optional `custom.css` link
- [x] 2.2 Write the search styles as `lsg-search.css`, the flashcard widget styles as `lsg-flashcards-widget.css` and the derived-row styles as `lsg-derived.css`, each linked from the position its inline block held, on the pages that carried it
- [x] 2.3 Add the new file names to the reserved names that `--readable-urls` checks titles against
- [x] 2.4 Update the tests in `crates/logseq-gardener-sdk/tests/` that look for inline styles to look for the links and the shared files, and add a test that a garden edit leaves the shared stylesheets unchanged

## 3. Inline scripts

- [x] 3.1 Split `theme.js` into a minified inline part that applies the saved layout and theme before paint, and `lsg-theme.js`, loaded with `defer`, that wires the menu items and exposes `window.logseqGardenerInitTheme`
- [x] 3.2 Minify the inline fragment-reveal script and keep it at the end of the body
- [x] 3.3 Move the search start-up into `lsg-search-boot.js`, loaded with `defer` from the place the inline script held, and keep its `DOMContentLoaded` and animation-frame timing
- [x] 3.4 Update the theme, search and site tests for the new script layout, and add a test that the inline scripts on an ordinary page total at most 1.5 KB

## 4. Compressed reference fragments

- [x] 4.1 Write Linked and Unlinked References fragments without the readable-page wrapper, gzip-compressed at level 6 with an empty gzip header, as `_refs/<page>-linked-<n>.html.gz` and `_refs/<page>-unlinked-<n>.html.gz`, compressing on several threads during the write phase
- [x] 4.2 Remove the `<noscript>` fragment links from the Linked and Unlinked References headers; keep the Hierarchy section's fragments and fallback as they are
- [x] 4.3 In `references.js`, fetch fragments as binary data, decompress with `DecompressionStream` when the data starts with the gzip header, read it as text otherwise, and say that the references could not be loaded when either step fails
- [x] 4.4 Check that the right sidebar's reference sections load the compressed fragments
- [x] 4.5 Add tests: a fragment is gzip with an empty header, two builds of an unchanged garden write identical fragments, and a browser test opens Linked and Unlinked References on a fixture page from compressed and from already-decoded files

## 5. Size checks

- [x] 5.1 Add the `[site]` table to `scripts/parity/budgets.toml`: `site_mb = 550`, `page_html_mb = 200`, `refs_mb = 100`, `inline_script_kb = 1.5` and `build_slowdown_pct = 10`
- [x] 5.2 Make `parity:publish-bench` record each part's file count and size in its rows, and make `--check` fail when the encode garden's site, page HTML or `_refs/` exceeds its budget
- [x] 5.3 Make `parity:budgets` fail on any inline `<style>` element written by the publisher, on inline scripts over `inline_script_kb`, and on any `_refs/` file over 100 KB as stored
- [x] 5.4 Change the `[shell]` frame check to measure the header and sidebar markup only, and the references fragment check to measure files as stored
- [x] 5.5 Add tests for the new checks in `scripts/parity/test_parity.py`

## 6. Measure

- [x] 6.1 Publish the encode garden at the CI-pinned commit with the changed binary and confirm the site is under 550 MB with no post-processing; record each part's size beside the baseline in the pull request
- [x] 6.2 Run `parity:screenshots` again and confirm the fixture pages look the same as in 1.2
- [ ] 6.3 Run `parity:build`, `parity:compare` and `parity:perf --record --baseline-lsg <baseline>` on the whole encode garden against Logseq's `gh-pages` build, and record both sites' sizes and file counts on the Performance page
- [x] 6.4 Run `parity:perf --search-site` on the whole encode garden and confirm the search budgets still hold
- [x] 6.5 Run `mise run parity:publish-bench --corpus all --record` and add the rows, with their part sizes, to `docs/parity/publication-speed.json`
- [x] 6.6 Update `docs/pages/lsg___Static Publishing.md` to describe the shared files and the compressed fragments
- [x] 6.7 Run `mise run ci`

## 7. Follow-up

- [ ] 7.1 After a release with this change, open an issue in logseq-encode-garden to remove the `share_repeated_styles` step from `mise-tasks/site/build`
