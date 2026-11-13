# Verification

Recorded on 2026-11-12 for the implementation pull request. A row that could not run says so and gives the reason.

## Real-device checklist

Everything the agent ran is Chromium 148.0.7778.96 (Chrome for Testing, headless shell) driven by Playwright 1.60.0 on macOS 26 (Apple silicon). No physical phone, tablet or second computer was available, and the agent cannot drive Safari or Firefox. The Playwright builds of Firefox and WebKit were not installed, and installing them needs a download that nobody was available to approve.

| Device and browser | Check | Result |
| --- | --- | --- |
| iPhone, Safari | Save, three pages, airplane mode, the notice, an unvisited page, VoiceOver | Not run: no iPhone. |
| iPhone, Home Screen app | The same, then close and reopen the app offline | Not run: no iPhone. |
| Android phone, Chrome | The same, and clear site data to confirm Remove | Not run: no Android device. Chromium 148 on the desktop ran the same steps; see the checks below. |
| Mac, Safari and Chrome | The same, and two gardens on one origin keep separate caches | Safari not run: the agent cannot drive Safari. Chrome: headless Chromium 148 ran the steps, and gardens at `/a/` and `/ab/` of one origin kept separate caches and registrations; the Application panel was not opened. |
| Firefox desktop | The same | Not run: Firefox not available to the agent. |
| Author turns it off | Publish `:lsg/offline false` after a reader saved pages | Run in Chromium 148: after one online page view, the reader had no registration and no cache. Not run in other browsers. |
| Registered worker with a missing script | What each browser does when `lsg-sw.js` returns 404 | Run in Chromium 148: `registration.update()` rejected with "A bad HTTP response code (404) was received when fetching the script"; the registration stayed, the page stayed controlled and the cache stayed. Not run in Safari or Firefox. |

## Checks run on this change

| Check | Result |
| --- | --- |
| `cargo test` for the key, the worker and the publisher | Pass. `garden_settings` and `offline` unit tests, `site_offline` integration tests, `cli_offline_key_writes_the_worker_and_key_errors_fail_without_output` and the help test. |
| `node --test tests/*.test.cjs` | Pass, including `tests/offline.test.cjs` for `lsg-offline.js`. |
| `mise run offline:browser-test` | Pass in Chromium 148; the next table lists each check. |
| `mise run parity:budgets` | Pass. On the fixture garden, 53 pages: the largest growth is 249 bytes and 53 gzip; `lsg-sw.js` is 4,606 bytes (1,776 gzip); `lsg-offline.js` is 1,957 bytes (959 gzip); first paint and growth with 2,000 more pages are unchanged; search files are identical. |
| `mise run parity:build-bench --offline-only --runs 5` | Pass: +162 ms by the paired median, budget 1,000 ms. See the build time section. |
| Garden without the key against the previous release | Pass. `lsg` built from `origin/main` (`6ee0319`) and this branch published the whole encode garden (66,241 files, with `--include-all --journals --search --readable-urls`) and the documentation graph without its new key (946 files with `--readable-urls --search`, 955 files with `--journals --search`), all with `SOURCE_DATE_EPOCH` set: every file is byte-identical. |
| `mise run docs:site` with `:lsg/offline true` | Pass, with no diagnostic. |
| `mise run ci` | Pass. |
| `mise run openspec:validate` | Pass. |

## Chromium checks

`mise run offline:browser-test` publishes a garden with `:lsg/offline true`, `:lsg/install` and `--readable-urls --search` at `/a/` and `/ab/` of one origin and at the root of another, and serves it from a test server that can stop, slow down and answer HEAD with 405.

| Check | Result |
| --- | --- |
| Three pages without opening the menu | No registration, cache, local or session storage entry or database; no request for `lsg-sw.js`. |
| Save from the keyboard | One registration with scope `/a/`; the status line says saving is on; focus returns to More; the menu shows Remove. |
| What Save keeps | The pages opened, `_refs/` fragments, `lsg-site.css`, `lsg-shell.js`, `lsg-theme.js`, `lsg-offline.js` and the other shared scripts and styles. The search index and a 411 KB image are not kept; the image still shows online. Chromium does not request `lsg.webmanifest` through the worker, so it is not kept. |
| Online after the page changed | The new page shows and replaces the copy. |
| Server stopped, visited page | The copy shows with "Saved copy from November 12". |
| Server stopped, unvisited page | `net::ERR_FAILED`, the browser's offline page; nothing stored for that address. |
| Browser offline, visited page | The copy shows with the notice. |
| Server answers after 6 seconds | The copy shows after 4.05 seconds, with the notice. |
| Page now answers 404 | The site's 404 shows and the copy is deleted. |
| Remove from the keyboard | The cache and the registration are gone and Save shows again. |
| Page opened from disk | Neither item, no script error. |
| Site at an origin root | Neither item, no registration. |
| Gardens at `/a/` and `/ab/` | Scopes `/a/` and `/ab/`; caches `lsg-offline:/a/` and `lsg-offline:/ab/`; a request from a page of `/a/` for a page of `/ab/` is not stored by `/a/`; Remove in `/a/` leaves the `/ab/` cache and registration unchanged. |
| Block edit, then publish | `lsg-sw.js` unchanged. |
| Unpublished page, then publish and browse | The new worker took over, sent 16 HEAD requests and deleted the copy of the unpublished page. |
| Host answers HEAD with 405 | The copy stays. |
| Copy saved 31 days ago | Deleted the next time the worker started. |
| 300 pages of 100 KB | 250 copies, 25 MB; the earliest are gone. |
| 200 pages of 200 KB | 160 copies, 32 MB. |
| Quota of 3 MB | All 60 pages of 200 KB loaded; the worker kept 11 copies, 2.2 MB. |
| `:lsg/offline false`, then one online page view | No registration and no cache. |
| `lsg-sw.js` returns 404 | Recorded in the checklist above. |
| `nav_overhead_ms` | 0.4 ms (median of 15 navigations each, 1.7 ms without a worker, 2.1 ms with one); budget 50. |
| `memory_growth_mb` | 0.15 MB; budget 2. |
| `cache_pages`, `cache_mb` | 250 and 32.0; budgets 250 and 32. |

## Build time on the encode garden

Measured with `mise run parity:build-bench --garden <encode garden> --offline-only --runs 5` on the local encode garden at commit `17a6d17` (14,474 published pages). One copy of the garden gains `:lsg/offline true`; a second copy has no key.

| Build | Seconds, per round |
| --- | --- |
| Copy without the key | 20.67, 20.73, 20.36, 20.42, 20.49 |
| Copy with the key | 20.43, 22.82, 21.21, 20.58, 20.09 |

The median of the paired differences is +162 ms, which `parity:build-bench` checks against `build_ms`. The fastest build with the key was 270 ms faster than the fastest without it. The second round's 2.1 s difference is machine noise of the kind that made the design's 50 ms row unusable.
