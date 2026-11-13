# Spec Delta

## Purpose

A reader can choose to keep the pages of a published garden that they open, and open those pages again with no connection. The author turns the feature on with one entry in the garden's `logseq/config.edn`. A garden without the entry publishes the same site as before. Nothing loads or is stored before the reader chooses, stored copies stay within fixed limits, and pages that the site stops serving are removed from readers' devices.

## ADDED Requirements

### Requirement: Opt in with `:lsg/offline`

`lsg publish` SHALL add offline reading to a site only when the garden's `logseq/config.edn` holds a top-level `:lsg/offline` key whose value is `true` or `false`. Any other value, or a nested use of the key, SHALL stop the build before it creates the output directory, exit 1 with empty stdout, and write a message to stderr that names `logseq/config.edn`, the key and the problem. The key SHALL be the only switch: `lsg publish` SHALL have no flag for it, and Rust callers of the SDK SHALL get the same result as the command line. A garden without the key SHALL publish files identical to the previous release with the same options and `SOURCE_DATE_EPOCH`. The key SHALL work with or without `:lsg/install`, and neither SHALL read the other.

#### Scenario: Turned on
- **WHEN** a garden whose `logseq/config.edn` holds `:lsg/offline true` is published
- **THEN** the output holds `lsg-offline.js` and `lsg-sw.js` at the site root and every full page links `lsg-offline.js`

#### Scenario: Wrong value
- **WHEN** the key holds `"yes"`
- **THEN** the build exits 1 with empty stdout, names `logseq/config.edn` and `:lsg/offline`, says the value must be `true` or `false`, and creates no output directory

#### Scenario: No key
- **WHEN** a garden without the key is published by this release and by the previous release with the same options and `SOURCE_DATE_EPOCH`
- **THEN** the two outputs are byte-identical

### Requirement: Register and store nothing before the reader chooses

Every full page of a site with `:lsg/offline true` SHALL contain a Save pages for offline reading item and a Remove offline copies item in the More menu, both as native buttons, and one `<script defer src="lsg-offline.js">`. Until the reader activates Save, the page SHALL register no service worker, open no cache, write no browser storage and send no request that a site without the key does not send. Activating Save SHALL register `lsg-sw.js` once, then show a status message that says saving is on and that pages opened from now on are kept. Activating Remove SHALL delete the garden's cache, unregister the garden's worker and show a status message. After Save, the page SHALL show Remove; before it, the page SHALL show Save. A page opened from disk, or from any address that is not HTTPS or localhost, SHALL show neither item.

#### Scenario: First visit
- **WHEN** a reader opens three pages of an offline-enabled garden and never opens the More menu
- **THEN** the browser has no service worker, no cache entry and no storage entry for the garden

#### Scenario: Choosing Save
- **WHEN** the reader activates Save pages for offline reading
- **THEN** one worker is registered at the garden's folder, the menu shows Remove offline copies in place of Save, and a status message says saving is on

#### Scenario: Choosing Remove
- **WHEN** the reader activates Remove offline copies
- **THEN** the garden's cache is gone, the garden's worker is unregistered and the menu shows Save again

#### Scenario: Opened from disk
- **WHEN** the site is opened with a `file:` address
- **THEN** neither item is shown and no script error is raised

### Requirement: Keep the worker inside the garden's folder

The worker SHALL be registered with the folder that holds `lsg-sw.js` as its scope, so it controls only that garden's pages, at `/`, at `/gitpa/` and anywhere else, with and without `--readable-urls`. When that folder is the origin root, the page SHALL show neither item and SHALL register nothing. The worker SHALL pass every request that is not a same-origin GET inside its scope to the network unchanged.

#### Scenario: Garden under a path
- **WHEN** two gardens are served at `/a/` and `/ab/` of one origin and the reader chooses Save in both
- **THEN** the registrations have the scopes `/a/` and `/ab/`, and the worker of `/a/` does not handle a request for a page of `/ab/`

#### Scenario: Garden at the origin root
- **WHEN** the garden is served at the root of its origin
- **THEN** the menu shows neither item and no worker is registered

### Requirement: Name and touch only the garden's own cache

The worker SHALL keep its copies in one cache named `lsg-offline:` followed by its scope's path, and SHALL open, list, match or delete a cache only when the name equals that exact string. It SHALL NOT use a name prefix test. It SHALL never call `navigator.storage.persist()`.

#### Scenario: Neighbouring prefix
- **WHEN** caches `lsg-offline:/a/` and `lsg-offline:/ab/` both exist and the worker of `/a/` clears its cache
- **THEN** `lsg-offline:/ab/` is unchanged

### Requirement: Keep visited pages, network first

After the reader chooses Save, the worker SHALL keep a copy of each same-origin GET response with status 200 inside its scope that a page of the garden requests, except responses that the storage limits exclude. A navigation SHALL go to the network first. When the network answers 200, the reader SHALL see that response and the worker SHALL replace the stored copy. When the network fails, the worker SHALL answer from the stored copy of that address, and from none otherwise, which leaves the browser's own offline page. When the network does not answer within 4 seconds, the worker SHALL answer from the stored copy, and SHALL keep waiting for the network when there is none. The worker SHALL turn on navigation preload so that waiting for the worker's start adds no delay to an online navigation. A page served from a stored copy SHALL show a notice with the date the copy was saved, written from the stored time.

#### Scenario: Online
- **WHEN** an online reader opens a page that was changed and published after they saved it
- **THEN** the reader sees the new page and the stored copy is replaced

#### Scenario: Offline, visited
- **WHEN** an offline reader opens a page they visited while saving was on
- **THEN** the page appears from the stored copy with a notice that gives the day it was saved

#### Scenario: Offline, never visited
- **WHEN** an offline reader opens a page they never opened while saving was on
- **THEN** the browser's offline page appears and the worker has stored nothing for that address

#### Scenario: Slow network
- **WHEN** the network does not answer a navigation for a stored page within 4 seconds
- **THEN** the page appears from the stored copy

### Requirement: Bound what is stored

The worker SHALL store at most 250 pages' worth of copies, at most 32 MB in total and at most 256 KB per response, keep a record of each copy's size, save time and last use, and drop the copy used least recently first until both totals are met. It SHALL NOT store audio or video responses, any file of the search index, or a response above the per-response limit; those requests go to the network only. When writing a copy raises a quota error, the worker SHALL drop the older half of its copies, try once more, and then skip that copy without failing the page.

#### Scenario: Size cap
- **WHEN** a saved page links an image of 400 KB
- **THEN** the page is stored and the image is not, and the image appears again when the reader is online

#### Scenario: Count and size caps
- **WHEN** the reader opens 300 pages of 100 KB each with saving on
- **THEN** the cache holds the most recently used pages, at most 250 copies and at most 32 MB, and the earliest pages are gone

#### Scenario: Quota error
- **WHEN** the browser refuses a write for lack of space
- **THEN** the page still loads, the older half of the copies is dropped, and the worker tries the write once more

### Requirement: Remove copies of pages that are no longer published

A copy SHALL expire 30 days after it was saved, and the worker SHALL delete expired copies whenever it starts. A network answer of 404 or 410 to a navigation SHALL delete that page's copy and show the network answer. A new worker version SHALL, when it activates, ask the network about each stored page with a HEAD request, two at a time and after the reader's own requests, and delete each copy whose answer is 404 or 410. Any other answer or a failed request SHALL keep the copy. Every publish that changes the set of published pages SHALL change `lsg-sw.js`, so that browsers install the new version.

#### Scenario: Page unpublished
- **WHEN** an author unpublishes a page that a reader saved, publishes again, and the reader opens any page of the garden online
- **THEN** the worker updates, asks about each stored page, and deletes the copy of the unpublished page

#### Scenario: Reader opens the removed page online
- **WHEN** the reader opens the removed page while online
- **THEN** the reader sees the site's 404 and the stored copy is deleted

#### Scenario: Never revisited
- **WHEN** a copy is 31 days old and the reader opens any page of the garden
- **THEN** the copy is gone

#### Scenario: Host rejects HEAD
- **WHEN** the host answers a HEAD request with 405
- **THEN** the copy stays

### Requirement: Update without losing the reader's copies

`lsg-sw.js` SHALL change only when the `lsg` version or the set of published page addresses changes. A new version SHALL call `skipWaiting()` and claim open pages once it activates, and SHALL keep the stored copies of the old version. The worker SHALL need no list of files to install.

#### Scenario: Garden edit that keeps the page list
- **WHEN** a block is edited, the garden is published again with the same `lsg`, and the reader opens a page
- **THEN** `lsg-sw.js` is unchanged and the browser keeps its running worker

#### Scenario: New version
- **WHEN** a new version activates
- **THEN** the stored copies from the old version are still available offline

### Requirement: Turn offline reading off after readers have saved pages

`:lsg/offline false` SHALL write `lsg-sw.js` as a worker that, on activation, deletes the garden's cache and unregisters itself, and SHALL write no `lsg-offline.js`, no menu item and no page tag. A garden with no key SHALL write no `lsg-sw.js`. `lsg-sw.js` SHALL be a reserved file name under `--readable-urls` whether the key is `true` or `false`.

#### Scenario: Author turns it off
- **WHEN** a garden changes `:lsg/offline true` to `false`, is published, and a reader who saved pages opens any page online
- **THEN** the reader's worker updates to the removing version, the cache is deleted and the worker is unregistered

#### Scenario: Key removed
- **WHEN** a garden removes the key
- **THEN** the output holds no `lsg-sw.js` and no `lsg-offline.js`, and the documentation says readers who saved pages keep their worker until the author publishes `false` or the reader chooses Remove

### Requirement: Stay inside size and speed budgets

The `[offline]` table in `scripts/parity/budgets.toml` SHALL set: `lsg-sw.js` at most 5 KB and 2 KB gzip, `lsg-offline.js` at most 2 KB and 1 KB gzip, page growth at most 0.6 KB and 0.3 KB gzip, page growth when the garden gains 2,000 pages at 0, page-load memory growth after Save at most 2 MB, `nav_overhead_ms` at most 50 at the median for an online navigation through an active worker, and `build_ms` at most 1000 on the encode garden, a limit above the run-to-run noise of a whole build. First paint of a page SHALL stay inside the existing 100 KB of data and 16 KB of script before its title when a reader has not chosen Save. `parity:budgets` SHALL fail and name the row that exceeds its limit.

#### Scenario: Oversized worker
- **WHEN** a change makes `lsg-sw.js` 7 KB
- **THEN** `parity:budgets` fails and reports `lsg-sw.js` at 7 KB against its 5 KB budget

#### Scenario: Garden growth
- **WHEN** the fixture garden gains 2,000 pages
- **THEN** every page's added bytes are unchanged

### Requirement: Document offline reading for readers and authors

The documentation SHALL include `lsg/How To/Read a Published Garden Offline`, which tells a reader how to choose Save and Remove, what is kept, how long, that Safari can delete a website's stored data, including saved pages, after seven days of Safari use in which the reader has not opened the site, and that a garden added to the Home Screen has its own count that resets when the reader uses the app, and how to clear it in the browser. It SHALL also include `lsg/Ref/Offline Options`, which lists the key, each limit, the generated files, the update and removal rules, the origin-root rule, the shared-origin caveat that scripts of any garden on the same origin can read the others' caches, and the steps to turn the feature off.

#### Scenario: Reader's page
- **WHEN** a reader opens the how-to page
- **THEN** it states the limits and the Safari deletion rule before the steps for clearing data
