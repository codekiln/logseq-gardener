# Tasks

## 1. Scripts after the first paint

- [x] 1.1 In `lsg-shell.js`, start the post-load scripts after the next paint and load `lsg-all-pages.js` on an All pages screen; `cargo test -p logseq-gardener-sdk --test site_shell` passes and `lsg-shell.js` stays within 6,900 bytes
- [x] 1.2 Drop the All pages `<script defer>` tag and let the plugin host load `lsg-all-pages.js` when it adopts All pages; `site_all_pages` asserts the page names no `lsg-all-pages.js` and the shell loads it
- [x] 1.3 Measure `All pages` and `Contents` with `parity:perf --builds lsg` before and after; record the script before the title in `verification.md`

## 2. Reference filter

- [x] 2.1 Add `content-visibility: auto` to reference groups and set `[references] styles_gzip_kb` to 1.05 with its reason in `budgets.toml`; `parity:budgets` passes
- [x] 2.2 Measure the filter change on `mise` before and after on desktop and phone; record it in `verification.md`

## 3. Harness

- [x] 3.1 Count a keystroke typed while a shard loads as cold in `metrics.keystroke_fetches`, and update the search section texts in `perf.py`; `test_parity` covers it
- [x] 3.2 Count every script on every search-build page, All pages included, with the page-property start-up, in `budgets.check_search`; `test_parity` covers it
- [x] 3.3 Measure the phone search before and after the new classification; record it in `verification.md`

## 4. Integration

- [x] 4.1 Record the phone title and build time explanations in `verification.md`
- [x] 4.2 `mise run ci` passes

## Workflow follow-up

- After merge, sync the delta specs and archive the change, as `lsg/How To/Finish an OpenSpec Change After Merge` describes.
- The next full `parity:perf --record` run rewrites the Performance page with these changes.
