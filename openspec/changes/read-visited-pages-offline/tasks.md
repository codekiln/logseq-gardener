## 1. The key and the worker source

- [x] 1.1 Read `:lsg/offline` in `garden_settings.rs` into `OfflineSetting` (`Absent`, `On`, `Off`), with errors for a value that is not a boolean and a nested use, each naming `logseq/config.edn`, the key and the problem; unit-test each and the absent key, and run `cargo test -p logseq-gardener-sdk garden_settings`
- [x] 1.2 Add `offline.rs` with the limits, the cache name from a scope path and the build identifier from the `lsg` version and the published page addresses; unit-test that a block edit leaves the identifier unchanged, that adding or removing a page changes it, and that `/a/` and `/ab/` give names that no prefix test can confuse
- [x] 1.3 Write the worker source for `On` and `Off` as pure functions; unit-test its exact bytes for a fixed identifier and check it stays under 5 KB
- [x] 1.4 Make `lsg publish` exit 1 with empty stdout and no output directory on a bad key, and describe the key and `lsg/Ref/Offline Options` in publishing help; test both in the CLI tests and run `mise run clilint:check`

## 2. Files, tags and menu items

- [x] 2.1 Write `lsg-sw.js` for `On` and `Off`, and `lsg-offline.js` for `On`, at the site root; reserve both names under `--readable-urls` for either value and test the reserved-name failure in `tests/`
- [x] 2.2 Add the deferred script tag and the two hidden menu items to every full page of an `On` site, after the `<base>` of a readable build; test the home page, journals feed, All pages, graph view, namespace pages and nested readable pages, and that redirect pages and fragments carry nothing
- [x] 2.3 Write `lsg-offline.js`: show Save or Remove by registration state, register with the folder scope, refuse at an origin root, a `file:` address or an insecure origin, delete the cache and unregister on Remove, write outcomes to the polite status region, and show the saved-on notice from the stored copy when offline; add Node tests in `tests/` and run `node --test tests/*.test.cjs`
- [x] 2.4 Test that a garden without the key publishes files identical to the previous release, that `lsg-sw.js` is identical after a block edit, and that every URL in the output resolves with and without `--readable-urls`

## 3. The worker's behaviour

- [x] 3.1 Implement network first with preload, the 4-second fallback, 200 stored, 404 and 410 deleted, and pass-through for requests outside the scope; add browser scenarios with the network stopped and slowed (in `offline:browser-test`; see design.md, question 21)
- [x] 3.2 Implement the limits, the record entry, least recently used dropping, quota handling and the exclusions for media, search files and large responses; test 300 pages of 100 KB, a 400 KB image and a refused write
- [x] 3.3 Implement expiry at 30 days, deletion on 404 and 410, and the HEAD check at activation, two at a time, keeping a copy on 405 or failure; test an unpublished page, a 31-day-old copy and a host that rejects HEAD
- [x] 3.4 Implement the removing worker for `Off`; test that a reader who saved pages loses the cache and registration after one online visit, and record what Chromium does when a registered worker's script later returns 404
- [x] 3.5 Add browser scenarios for the rest: nothing registered or stored before Save, scope under a path prefix, neighbour isolation for `/a/` and `/ab/`, no worker at the origin root, and the manifest and shared files cached after Save; run `mise run offline:browser-test`

## 4. Budgets

- [x] 4.1 Add the `[offline]` table to `scripts/parity/budgets.toml` and make `parity:budgets` build the fixture garden with the key, checking file sizes, page growth, first paint before Save, growth with 2,000 more pages, identical search files, `nav_overhead_ms`, `memory_growth_mb` and the stored copy limits; run `mise run parity:budgets`
- [x] 4.2 Add an `offline` variant to `parity:build-bench` that adds the key to a copy of the pinned encode garden and checks `build_ms`; run `mise run parity:build-bench --garden target/encode-garden --runs 5`
- [x] 4.3 Record the offline rows in `lsg/Ref/Logseq Compatibility/Performance`

## 5. Documentation

- [x] 5.1 Add `lsg/How To/Read a Published Garden Offline` with the steps, limits, expiry, notice, the Safari seven-day rule and how to clear data
- [x] 5.2 Add `lsg/Ref/Offline Options` with the key, limits, files, update and removal rules, the origin-root rule, the shared-origin caveat and the steps to turn the feature off
- [x] 5.3 Link both pages from `lsg/How To/Publish a Garden to GitHub Pages` and the publishing help, and run `mise run docs:check`
- [x] 5.4 Turn the key on for the documentation site and check that `mise run docs:site` builds it with no diagnostic

## 6. Verify

- [x] 6.1 Run the real-device checklist in the design and record each row in `verification.md` with the device, operating system, browser version, date and result, marking a row that cannot run as not run with its reason
- [ ] 6.2 Run `mise run ci` and `mise run openspec:validate`, then open the implementation pull request with a cleanup sub-issue under its implementation issue
- [ ] 6.3 File the consumer issues in logseq-encode-garden and gitpa that turn the key on after the release
