## 1. The key, the icon and the manifest

- [x] 1.1 Read `:lsg/install` in `garden_settings.rs` into `InstallSettings`, with errors for a missing key, an unknown key, a wrong type, a length out of range, a name over 12 characters without a short name and a nested use, each naming `logseq/config.edn`, the key and the problem; unit-test each, an absent key and a name of exactly 12 characters, and run `cargo test -p logseq-gardener-sdk garden_settings`
- [x] 1.2 Add `install.rs` with the `Icon` constructor, which reads the PNG header and checks inside-the-garden path, square size from 512 to 1024 pixels and at most 256 KB; unit-test a valid icon, a missing file, an SVG, a 300 by 300 PNG, a 600 by 512 PNG, a 300 KB PNG and `../icon.png`
- [x] 1.3 Build the manifest from `InstallSettings` as a pure function and unit-test its exact bytes, the absence of `id`, a 45-character name staying under 1 KB and the escaping of quotes and non-ASCII characters in the name
- [x] 1.4 Make `lsg publish` exit 1 with empty stdout and no output directory on a key or icon error, and describe the key, one example and the `lsg/Ref/Install Options` page in publishing help; test both in the CLI tests and run `mise run clilint:check`

## 2. Files, tags and the menu item

- [x] 2.1 Write `lsg.webmanifest`, `lsg-icon.png` (byte-identical to the source) and `lsg-install.js` at the site root of an installable site, reserve the three names under `--readable-urls`, and test the reserved-name failure in `tests/`
- [x] 2.2 Add the manifest link, the `apple-touch-icon` link, the two `theme-color` elements and the deferred script to the head of every full page of an installable site, after the `<base>` of a readable build; test the home page, journals feed, All pages, graph view, namespace pages and nested readable pages, and that redirect pages and fragments carry no tag
- [x] 2.3 Add the hidden Install app menu item to the More menu of an installable site and write `install.js`, which keeps `beforeinstallprompt`, shows the item, calls `prompt()` once and returns focus to the More button; add Node tests in `tests/` for the script's functions and run `node --test tests/*.test.cjs`
- [x] 2.4 Test that a garden without the key publishes files identical to the previous release, that the files are identical when a page changes, and that every URL in the manifest and head resolves to a file in the output with and without `--readable-urls`

## 3. Browser checks and budgets

- [x] 3.1 Add `plugins:browser-test` scenarios that serve the fixture garden under a path prefix with the key and check: the manifest and icon return 200 from a nested page; the manifest validates and resolves `start_url` and `scope` to the site folder; no service worker, cache entry or storage entry exists after three navigations; the item stays hidden when the event never fires and appears when a test page dispatches it; the page opened from disk raises no error; the manifest link survives plugin navigation; run `mise run plugins:browser-test`
- [x] 3.2 Add the `[install]` table to `scripts/parity/budgets.toml` and make `parity:budgets` build the fixture garden with the key, checking page growth, script and manifest sizes, first paint, growth with 2,000 more pages and identical search files; run `mise run parity:budgets`
- [x] 3.3 Add an `install` variant to `parity:build-bench` that adds the key and an icon to a copy of the pinned encode garden and checks `build_ms`; run `mise run parity:build-bench --garden target/encode-garden --runs 5`
- [x] 3.4 Record the install rows in `lsg/Ref/Logseq Compatibility/Performance`

## 4. Documentation

- [x] 4.1 Add `lsg/How To/Install a Published Garden as an App` with the steps and versions per browser, what installation does and does not do, and where the Share menu is for Safari and Firefox readers
- [x] 4.2 Add `lsg/Ref/Install Options` with the key, each check, the generated files, the budgets, the scope and identity rules, the origin-root caveat and the limits
- [x] 4.3 Link both pages from `lsg/How To/Publish a Garden to GitHub Pages` and from the publishing help, and run `mise run docs:check`
- [x] 4.4 Turn the key on for the documentation site with an icon file under `logseq-gardener-docs/`, and check that `mise run docs:site` builds it with no diagnostic

## 5. Verify

- [x] 5.1 Run the real-device checklist in the design and record each row in `verification.md` with the device, operating system, browser version, date and result, marking a row that cannot run as not run with its reason
- [x] 5.2 Check that Chrome on desktop and Android promotes installation with one 512 pixel icon; if it does not, record the result and stop for a design change before merging
- [x] 5.3 Run `mise run ci` and `mise run openspec:validate`, then open the implementation pull request with a cleanup sub-issue under its implementation issue
- [x] 5.4 File the consumer issues in logseq-encode-garden and gitpa that turn the key on after the release
