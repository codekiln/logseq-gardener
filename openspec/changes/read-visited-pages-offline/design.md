# Design

## Context

[Issue #603](https://github.com/codekiln/logseq-gardener/issues/603) plans installable gardens, and [Issue #607](https://github.com/codekiln/logseq-gardener/issues/607) asks whether offline reading is worth offering. See proposal.md for the motivation. The installation proposal ([make-gardens-installable](../make-gardens-installable/design.md)) fixed the site identity as the garden's own folder, the relative-path rule, the pattern of one config key with a hidden menu item, and a promise that installation adds no service worker, cache or storage. This change adds those things only for gardens that choose them, and for readers who choose them again.

What the repository already has:

- **Static pages with fixed shared files.** Every page links stylesheets and scripts with names and contents that depend only on the `lsg` version ([published-site-size](../../specs/published-site-size/spec.md)). `lsg-sw.js` is not linked from page markup, so its contents may also depend on the set of published pages.
- **Performance rules.** A page's first paint cannot depend on garden size, phone memory stays small, and benchmark rows run in CI ([#106](https://github.com/codekiln/logseq-gardener/issues/106), [publication-speed](../../specs/publication-speed/spec.md)). The encode garden publishes about 13,500 pages and 600 MB, so no design may fetch the whole garden.
- **Several gardens on one origin.** The documentation site, logseq-encode-garden and gitpa all publish under paths of `codekiln.github.io`.

### What the sources say and what was tested

Sources read on 2026-11-11. Rows marked tested were run in Chromium on a local server with two sites at `/a/` and `/ab/` and one at the origin root.

| Claim | Source | Result |
| --- | --- | --- |
| A worker's default scope is the folder that holds its script, and registering a wider scope fails unless the server sends `Service-Worker-Allowed` | [MDN: Using Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers) | Supported and tested: `/a/sw.js` got scope `/a/`, and `scope: '/'` raised `SecurityError`. GitHub Pages sets no such header, so a garden's worker cannot reach beyond its folder. |
| A worker at the origin root controls every path of that origin | same page | Tested: after `/sw.js` registered with scope `/`, a page at `/b/` reported that worker as its controller. A garden served from an origin root would therefore control its neighbours. |
| Cache Storage is shared by the whole origin, so names need a site prefix and cleanup must touch only that prefix | [web.dev: The service worker lifecycle](https://web.dev/articles/service-worker-lifecycle) | Supported and tested: a script on `/a/` listed and read the cache named for `/ab/`. A prefix test `startsWith("lsg-offline:/a")` matched both caches, so the design uses exact names. Any script in the origin can read any cache, which no naming scheme prevents. |
| A browser compares the worker script byte for byte, ignores HTTP caching when it checks the script, and a new worker waits unless it calls `skipWaiting()` | web.dev lifecycle article | Supported. A changed `lsg-sw.js` is the update signal, and a garden edit that leaves the file unchanged triggers nothing. |
| Quotas are per origin, best effort, and evict least recently used origins whole. Chromium allows up to 60% of disk, Firefox the smaller of 10% of disk or 10 GiB per group, Safari up to 60% per origin in a browser | [MDN: Storage quotas and eviction criteria](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria), [WebKit: Updates to storage policy](https://webkit.org/blog/14403/updates-to-storage-policy/) | Supported. Quotas are far above 32 MB, so the project's own cap is the binding one. Eviction removes everything of an origin, so every gardens' caches on `codekiln.github.io` can disappear together. |
| Safari deletes script-writable storage, including service worker registrations and caches, after seven days of Safari use without user interaction; a Home Screen web app keeps its own count | [WebKit: Full Third-Party Cookie Blocking and More](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/) | Supported. In a Safari tab, offline copies are a convenience for regular readers, not a guarantee. The how-to says so. |
| `navigator.storage.persist()` is granted or denied by browser heuristics with no prompt in Chromium and Safari, and with a prompt in Firefox | MDN quotas page | Supported. The design never calls it, because a prompt about a reading site is not worth its cost. |
| Chrome installs from the menu without a service worker | [Chrome install criteria](https://developer.chrome.com/blog/update-install-criteria) | Supported; this is why offline reading is a separate switch. |

Not tested, and left to the real-device checklist: Safari and Firefox behaviour on a phone, and what a browser does when the registered `lsg-sw.js` later returns 404.

## Goals / Non-Goals

**Goals:**

- A reader who chooses it can reopen the pages they have already read with no network, on a phone, with a visible note that the page is a saved copy.
- A reader who does not choose it sees the same site, the same first paint and no worker, cache or stored data.
- A reader is never shown an old page while online, and an unpublished page leaves their device soon.
- Gardens that share an origin cannot reach each other's copies through the worker.

**Non-Goals:**

- Saving the whole garden or chosen pages ahead of time. This is a competing goal for long trips, but it needs a download size choice and a progress interface, and the encode garden is 600 MB.
- Offline search. The search index loads on demand and has its own size and update rules.
- Offline media. Audio and video are large, and the sticky player has its own work.
- Background sync, push and badges.
- Offline reading for a garden at an origin root. It needs an opt-in that names the neighbours it will control.

## Decisions

### Is it worth offering?

Yes, with limits. The benefit is concrete for the garden's phone readers, and the cost to everyone else is zero: no file loads before a reader opts in, a garden without the key is unchanged, and a reader who does not use the menu item has no worker. The risks the issue lists each have a rule:

| Risk | Rule |
| --- | --- |
| Pages the author unpublishes stay on devices | 30-day expiry, delete on 404 or 410, a check of every saved page on each new worker version, an immediate Remove item |
| Stale content | Network first, so an online reader always sees the current page; a visible saved-on notice for copies |
| Updates | `lsg-sw.js` changes with the `lsg` version and the page list; the worker needs no install list; stored copies survive an update |
| Storage use and quotas | 250 pages, 32 MB, 256 KB per response, least recently used dropped first, quota errors handled, never `persist()` |
| Neighbouring gardens | Scope is the garden's folder; exact cache names; no worker at an origin root |
| Opting out and removing a worker | Reader: Remove offline copies. Author: `:lsg/offline false` publishes a removing worker |
| Local disk use | At most 32 MB per garden, shown to the reader in the how-to |

The performance rules decide the shape. First paint is untouched because registration waits for the reader. Memory does not grow with the garden, because the worker holds no page list and reads Cache Storage on demand. The worker is stopped by the browser when idle.

### The key

```clojure
:lsg/offline true
```

`true` turns the feature on, `false` publishes the removing worker, and no key changes nothing. Other values stop the build before the output directory exists, with the same error path as `:lsg/plugins`. The SDK reads the key into a three-case value, so no caller can hold a malformed setting:

```rust
pub enum OfflineSetting { Absent, On, Off }
```

There are no options for limits, because every option is a budget row to test and a surface to document. The limits are constants in the SDK.

### Files and pages

| File | Contents | Limit |
| --- | --- | --- |
| `lsg-sw.js` | The worker: the build identifier, the limits, the fetch rules, the removal check | 5 KB, 2 KB gzip |
| `lsg-offline.js` | The menu items, registration, the saved-on notice | 2 KB, 1 KB gzip |

The names are fixed and reserved under `--readable-urls`. Every full page of an `On` site gains one `<script defer src="lsg-offline.js">` after the `<base>` that a readable build writes, and two hidden menu items in the More menu, written the same way as the installation proposal's Install app item. Redirect pages and fetched fragments get nothing. The script is separate from `lsg-shell.js`, whose contents depend only on the `lsg` version, and from `lsg-install.js`.

### Registration and scope

`lsg-offline.js` resolves `lsg-sw.js` against the page's base, then registers it with `scope` set to that file's folder. When the folder path is `/`, the script shows neither item. When the page is not served over HTTPS or from localhost, it shows neither item. A garden at `/gitpa/` gets scope `/gitpa/`; a request for `/gitpa-notes/` is outside it, because scopes match whole path segments. The worker passes through any request that is not a same-origin GET inside its scope, so queries to other hosts, such as the media player's sources, never touch it.

### What is kept

The worker caches every same-origin GET with status 200 that a page of the garden requests: the page, `lsg-site.css`, `lsg-theme.js` and the other shared files, linked-reference fragments under `_refs/`, and images. Media, the search index and any response above 256 KB go to the network only. The cap keeps pages and shared files, which are within the project's 100 KB data budget, and excludes the 15 MB-class files that would make eviction frequent. A page shown offline without a large image is acceptable; the how-to says so.

One more cache entry holds a JSON record of each copy: its address, size, save time and last use. The record is how the worker applies the 250 page, 32 MB and 30 day rules without reading every body. If the record is missing or corrupt, the worker deletes the cache and starts over, which loses copies and never shows a wrong page.

### Network first, with a time limit

The worker answers a navigation like this:

1. Start the network request, with navigation preload turned on so the worker's start does not delay it.
2. If it returns 200, store the copy, update the record and show the response.
3. If it returns 404 or 410, delete the stored copy and show the response.
4. If it fails or takes more than 4 seconds, show the stored copy, or let the browser's offline page appear when there is none.

Cache first would load faster offline but would show old content to an online reader. The stored copy is the fallback, so the answer to "is this the current page?" is always yes online. Four seconds follows the same trade as other network-first guides: long enough for a slow mobile response, short enough to be useful on a failed connection. The reader sees the page's own load, not a spinner, because the page shows from the first byte. A served copy carries a notice with the day it was saved.

### The saved-on notice

The worker marks the copy it returns with a response header that holds the saved time. A navigation response's headers are not readable by the page, so `lsg-offline.js` reads the stored copy of its own address through the Cache API when `navigator.onLine` is false or when the page was loaded from a copy, and shows a one-line notice above the page: "Saved copy from 3 November". The page reads from the origin's own cache by exact name, so it needs no message from the worker and has no startup race. A reader who is online with a failing server sees a copy with no notice; the how-to says so, and the cost is accepted because a second channel would grow the script past its budget.

### Removing pages that are no longer published

Copies of pages the author has unpublished are the main privacy risk, because an author who unpublishes a page may be correcting a mistake. The device already received the text, so the risk is lasting local copies, not new disclosure. Four rules bound it:

1. A copy expires after 30 days, checked whenever the worker starts.
2. A 404 or 410 on an online navigation deletes the copy.
3. A new `lsg-sw.js` triggers a check of every saved page: when it activates, the worker sends a HEAD request for each stored page, two at a time, after the reader's own requests, and deletes each copy answered 404 or 410. That is at most 250 small requests per publish per reader. A 405 or a failed request keeps the copy.
4. The Remove item deletes everything at once.

The build writes a build identifier into `lsg-sw.js`, a hash of the published page addresses and the `lsg` version. A publish that adds or removes a page changes it, and a publish that edits a block does not, so most edits cost readers nothing. The hash covers addresses already known at build time; the added build cost is bounded by the row below.

A page that is unpublished but never revisited and never checked (a host that rejects HEAD) stays up to 30 days. That is the stated bound.

### Updating

The worker has no install list and no versioned caches. A new version calls `skipWaiting()` and claims open pages. This is safe because copies are not tied to the worker version: each is an ordinary page, replaced when the network returns a newer one. The browser's own update check compares the file and runs on navigation; no code of ours schedules it.

### Turning it off

A reader chooses Remove offline copies: the page deletes the garden's cache and unregisters the garden's worker. An author who wants every reader's worker gone publishes `:lsg/offline false`. Then `lsg-sw.js` is a worker that deletes the garden's cache and unregisters itself on activation, and no page references it. A registered worker keeps running after its script address disappears, so removing the key without publishing `false` would leave workers on readers' devices. The reference page says so. The browser test fetches a registered worker's script as 404 and records what Chromium does, and the checklist records it for Safari and Firefox.

### What lives in the SDK and what lives in the publisher

The SDK holds every rule:

- reading `:lsg/offline` into `OfflineSetting` (`garden_settings.rs`);
- the limits, the cache name rule and the address rules, as constants and pure functions (`offline.rs`);
- the worker's source for each setting, as a pure function of the setting and the build identifier;
- the build identifier from the published page addresses.

The publisher writes the two files, adds the script tag and the menu items to full pages through the same page writer, reserves the names and adds the `[offline]` rows to the budget checks. The two scripts react to events and draw one notice and two menu states. The CLI reports a rejected key through the existing error path and adds a paragraph to publishing help.

### Accessibility

- Both menu items are native buttons with text labels, in the menu's tab order, with the neighbours' focus style. Activating one moves focus to the More button and writes the outcome to the page's existing polite status region.
- The saved-copy notice is plain text above the page content, not a dialog.

### Budgets

Measured by `mise run parity:budgets` and `parity:build-bench` on the fixture garden and the pinned encode garden, in addition to the existing first-paint, shell and search rows. Row names are the keys of the `[offline]` table.

| Budget | Limit | Row |
| --- | --- | --- |
| Output without the key | Identical to the previous release | |
| `lsg-sw.js` | 5 KB, 2 KB gzip | `sw_kb`, `sw_gzip_kb` |
| `lsg-offline.js` | 2 KB, 1 KB gzip | `script_kb`, `script_gzip_kb` |
| Added bytes per page | 0.6 KB, 0.3 KB gzip | `page_growth_kb`, `page_growth_gzip_kb` |
| Page growth when the garden gains 2,000 pages | 0 | |
| First paint before Save | Existing 100 KB of data and 16 KB of script before the title, desktop and phone | `[desktop]`, `[phone]` |
| Online navigation through an active worker | At most 50 ms slower at the median | `nav_overhead_ms` |
| Page memory after Save | At most 2 MB more | `memory_growth_mb` |
| Stored copies | At most 250 pages and 32 MB, 256 KB per response | `cache_pages`, `cache_mb` |
| Search files | Identical to the build without the key | |
| Added build time on the encode garden | 50 ms | `build_ms` |

The files are fetched after the page has painted, `defer` does not delay paint, and a worker's start adds no delay to the page's first bytes because navigation preload is on. Worst-case storage is 32 MB per garden, so three gardens on one origin may use 96 MB, far below any browser's quota.

### Real-device checks

Each row is recorded in `verification.md` with the device, operating system, browser version, date and result. A row that cannot run is marked not run with its reason.

| Device and browser | Check |
| --- | --- |
| iPhone, Safari | Choose Save, open three pages, enable airplane mode, reopen them with the saved-on notice, open an unvisited page and see Safari's offline page; VoiceOver reads both items and the notice |
| iPhone, Home Screen app | The same, then close and reopen the app offline |
| Android phone, Chrome | The same, and clear site data to confirm Remove |
| Mac, Safari and Chrome | The same, and confirm that two gardens on `codekiln.github.io` keep separate caches in the Application panel |
| Firefox desktop | The same |
| Author turns it off | Publish `:lsg/offline false` after a reader saved pages; the reader's cache and worker are gone after one online visit |
| Registered worker with a missing script | Record what each browser does when `lsg-sw.js` returns 404 |

Chromium checks that need no device run in CI: nothing registered or stored before Save, the registered scope, the cache name and neighbour isolation, network-first behaviour with the network stopped, the 4-second fallback, the limits, the unpublished-page deletions, the removing worker, and no worker at an origin root.

### Documentation

`lsg/How To/Read a Published Garden Offline` is for the reader: the steps, what is kept and for how long, the Safari seven-day rule, the notice, and how to clear data. `lsg/Ref/Offline Options` is for the author: the key, the limits, the files, the update and removal rules, the origin-root rule, the shared-origin caveat and the steps to turn the feature off. `lsg/How To/Publish a Garden to GitHub Pages` gains one pointer. Publishing help gains a paragraph that names the reference page. The how-to for installation states that offline reading is a separate choice.

### Existing publishing sites

| Site | Turns it on | Notes |
| --- | --- | --- |
| Documentation site (this repository) | Yes, in the implementation pull request | Exercises the key on the site that documents it. |
| logseq-encode-garden, gitpa | After a release, in each repository's own issue | Each author decides; an older `lsg` ignores the key without an error. |

## Risks / Trade-offs

- A reader in a Safari tab who does not visit for a while loses their copies → the how-to says so, and a Home Screen app has its own count.
- An unpublished page may stay on a device up to 30 days when the host rejects HEAD and the reader never returns → the bound is stated, and Remove is immediate.
- Any script on the same origin can read every garden's cache → no naming scheme prevents it; the reference page says so, and readers of gardens that share an origin already trust that origin's scripts.
- A reader online with a failing server sees a copy with no notice → accepted, to keep the script small.
- Eviction by the browser removes every cache of an origin together → the worker treats a missing cache as empty and the reader can save again.

## Migration Plan

1. Merge this proposal, then implement behind the key; no garden changes until it adds the key.
2. After release, the documentation site turns the key on in the implementation pull request, and other gardens do so in their own issues.
3. To undo, publish `:lsg/offline false` for as long as readers' workers need to find it, then remove the key.

## Resolved Questions

### 1 - Is offline reading worth offering at all?

Decided by the agent without codekiln: yes, for visited pages only, behind a key and a reader choice. Phone readers lose the network often, and a static page costs little to keep. The performance rules rule out offering the whole garden. Every risk in the issue has a rule that a CI test checks, and the cost to a garden that does not opt in is zero bytes.

### 2 - Does the author or the reader decide?

Decided by the agent without codekiln: both. The author's key decides whether the site offers it, because a garden's author knows whether pages are sensitive. The reader's Save choice decides whether their device keeps anything, so a first visit stores nothing and a reader never receives a worker by accident.

### 3 - Should offline reading require installation?

Decided by the agent without codekiln: no. Installation and offline reading answer different questions, and Chrome installs without a worker. The two keys are independent so a garden can have either. A Home Screen app does keep its own count against Safari's seven-day deletion, and the how-to says that installing helps the copies last.

### 4 - Cache first or network first?

Decided by the agent without codekiln: network first, with a 4-second limit. Cache first would show stale pages to online readers, which the issue names as a risk. Network first keeps the site correct online and uses the copy only when the network fails.

### 5 - Precache the garden or cache visited pages?

Decided by the agent without codekiln: visited pages only. The encode garden is about 600 MB, and a precache list would grow with garden size, break the rule that cost does not grow with the garden, and cost every reader a large download. A visited-page cache has a fixed upper bound.

### 6 - What are the limits?

Decided by the agent without codekiln: 250 pages, 32 MB and 256 KB per response, least recently used first. Pages and their shared files are inside the 100 KB data budget, so 250 pages are about 25 MB at most, which fits 32 MB. A phone keeps working with 32 MB per garden, and every browser's quota is far higher. No setting changes them, because each option would need tests and documentation.

### 7 - How do unpublished pages leave the device?

Decided by the agent without codekiln: expiry after 30 days, deletion on 404 or 410, a HEAD check of every saved page when a new worker activates, and an immediate Remove item. A manifest of published pages would be 500 KB for the encode garden and would be fetched by every worker, so a per-page check bounded at 250 requests costs less. The residual 30-day window is documented.

### 8 - What isolates gardens that share an origin?

Decided by the agent without codekiln: the scope is the garden's folder, the cache name is `lsg-offline:` plus the scope path, and every cache operation uses the exact name. Testing showed that a prefix test matches `/ab/` from `/a/`. Testing also showed that any script on the origin can read every cache, so the documentation states that limit and the design does not claim stronger isolation.

### 9 - What about a garden at an origin root?

Decided by the agent without codekiln: no offline reading there. A worker at the root controls every neighbouring project site, as the test showed. An opt-in that names the neighbours is a later change.

### 10 - How does an author remove workers already on readers' devices?

Decided by the agent without codekiln: publish `:lsg/offline false`, which writes a worker that deletes its cache and unregisters itself. Removing the key leaves registered workers in place, so the reference page tells authors to publish `false` first.

### 11 - Which build identifier changes the worker?

Decided by the agent without codekiln: a hash of the `lsg` version and the published page addresses. A publish that edits a block leaves the file unchanged, so readers' workers do not run a check for edits. A publish that adds or unpublishes a page changes it, which is the case the removal check exists for. Page content changes need no worker update, because network first shows them.

### 12 - How does a reader know a page is a saved copy?

Decided by the agent without codekiln: the page script reads the stored copy of its own address from the garden's cache when the browser reports it is offline, and shows the save date above the page. A message from the worker would race with a deferred script, and response headers of navigations are not readable by a page.

### 13 - Should the garden call `navigator.storage.persist()`?

Decided by the agent without codekiln: no. Firefox shows a prompt, and the other browsers grant it by heuristics a reading site cannot influence. The limits are small and a lost cache costs a reader only the saved copies.

### 14 - Should the worker be part of `lsg-shell.js`?

Decided by the agent without codekiln: no. `lsg-shell.js` depends only on the `lsg` version, and a site without the key must not load worker code. Separate files keep the budget rows separate and match the installation proposal.
