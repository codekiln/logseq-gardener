# Verification

Recorded on 2026-11-11 for the implementation pull request. A row that could not run says so and gives the reason.

## Real-device checklist

Everything the agent ran is Chromium 148.0.7778.96 (Chrome for Testing, headless shell) driven by Playwright 1.60.0 on macOS 26.7.3 (Apple silicon). No physical phone, tablet or second computer was available, and the iOS Simulator listed no device. Chromium cannot show Chrome's menu or install dialog in headless mode, so the rows that need them are not run.

| Device and browser | Check | Result |
| --- | --- | --- |
| iPhone, Safari, iOS 17 or later | Add to Home Screen name, icon, own window, Back, external link, VoiceOver | Not run: no iPhone, and the iOS Simulator had no device. |
| iPhone, Chrome, iOS 16.4 or later | Add to Home Screen | Not run: no iPhone. |
| iPhone, iOS 16.3 or earlier | Bookmark, page still works | Not run: no such device. |
| iPad, Safari | Same as iPhone, in portrait, landscape and split view | Not run: no iPad. |
| Mac, Safari 17 or later | File, Add to Dock | Not run: the agent cannot drive Safari's menus. |
| Mac, Chrome and Edge | Menu, Install; address-bar button after a click and 30 seconds; the Install app item opens the dialog | Not run in a headed browser. In headless Chromium 148, `Page.getInstallabilityErrors` returned no error for the generated site with one 512 pixel icon, and the Install app item opened its prompt from the keyboard when the page dispatched `beforeinstallprompt`. The real event, the 30 second rule and the dialog were not observed. |
| Android phone, Chrome, with Google Mobile Services | Menu, Install app; WebAPK; the item appears | Not run: no Android device. |
| Android phone, Firefox | Add to Home Screen; no item | Not run: no Android device. |
| Two gardens on one origin | Install the documentation site and gitpa on `codekiln.github.io`; each its own app | Not run: needs a release and a browser that installs. Chromium 148 resolved `start_url` and `scope` to the site folder under `/gitpa/` and at `/`, and the manifest has no `id`, so the two gardens have two identities by the manifest specification. |
| Garden at the origin root | Install; a link to another project site | Not run: needs a browser that installs. |
| Icon | Check the icon in each launcher; one 512 pixel icon passes Chrome's install criteria | Partly run: Chromium 148 reports no installability error for the 512 pixel icon (Task 5.2: no design change needed on that evidence). Launcher cropping and contrast were not checked. |
| Chrome with a throttled network | Installing from the menu adds no service worker and no cache entry | Not run as written: no menu install. Chromium 148 showed no service worker registration, cache entry, storage entry or cookie after three navigations, the same as the build without the key. |

## Checks run on this change

| Check | Result |
| --- | --- |
| `cargo test` for the key, the icon, the manifest, the tags and the CLI | Pass. `garden_settings` and `install` unit tests, `site_install` integration tests, `cli_install_key_writes_the_manifest_and_key_errors_fail_without_output` and the help test. |
| `node --test tests/*.test.cjs` | Pass, including `tests/install.test.cjs` for `install.js`. |
| `mise run plugins:browser-test` | Pass in Chromium 148. The site under `/gitpa/` with readable URLs and the probe plugin: manifest and icon 200 from a nested page; Chromium parsed the manifest with no error and resolved `start_url` and `scope` to `/gitpa/`; the same site served at the origin root resolved to `/`; no service worker, cache, storage entry or cookie after three navigations, the same as the site without the key; the manifest link survived plugin navigation; the item stayed hidden until a test page dispatched `beforeinstallprompt`, then showed, opened the prompt once from the keyboard and returned focus to More; a page opened from disk raised no script error. |
| `mise run parity:budgets` | Pass. On the fixture garden, 53 pages: the largest growth is 380 bytes and 90 gzip; `lsg-install.js` is 694 bytes (380 gzip); the manifest is 240 bytes; first paint and growth with 2,000 more pages are unchanged; search files are identical. |
| `mise run parity:build-bench --install-only --runs 5` | Not within the budget by the fastest build, inside it by the paired median; see below. On the encode garden at commit `d9722ef`, 14,349 pages: the fastest build with the key was 252 ms slower than the fastest copy without it, and the median of the five paired differences was 82 ms faster. The run-to-run spread, about 1 second, is larger than the 50 ms budget. |
| Garden without the key against the previous release | Pass. `lsg` built from `origin/main` (`29b1e1e`) and this branch published the whole encode garden (`d9722ef`, 65,774 files, with `--journals --search`), the fixture garden (322 files, with and without `--readable-urls`) and the documentation graph without its new key (1,030 files, with `--readable-urls --search --redirect-old-urls`), all with `SOURCE_DATE_EPOCH` set: every file is byte-identical. |
| `mise run ci` | Pass. |
| `mise run openspec:validate` | Pass. |

## Build time on the encode garden

The 50 ms budget is 0.26 percent of a 19 second build, and the machine varies by about 1 second between identical builds, so the budget is below what this machine can measure. The five interleaved rounds took 19.2 to 20.4 s:

| Build | Seconds, per round |
| --- | --- |
| Copy without the key | 20.39, 20.09, 19.78, 19.55, 19.21 |
| Copy with the key and an icon | 20.18, 19.78, 20.13, 19.47, 19.62 |

The paired differences were -216, -308, +350, -82 and +406 ms. The median is -82 ms, which `parity:build-bench` checks against `build_ms`. The fastest build with the key was 252 ms slower than the fastest build without it. The first version of the check compared the key's build with the unmodified garden folder and showed 664 ms, because a hard-linked copy of the garden builds about half a second slower than the original; the check now compares two copies.
