tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Read a Published Garden Offline]], [[lsg/Ref/Publish Options]], [[lsg/Ref/Install Options]]

- # Offline Options Reference
	- ## Overview
		- A garden lets readers keep copies of the pages they open, and read them again without a network, with one top-level key in its own `logseq/config.edn`. Logseq ignores the key.
		- ~~~clojure
		  :lsg/offline true
		  ~~~
		- A garden without the key builds the same site as before: no worker, script, tag or menu item. The key is the only switch. `lsg publish` has no flag for it, and Rust callers of `publish_site_with_options` get the same result as the command line. The key works with or without `:lsg/install`, which [[lsg/Ref/Install Options]] describes.
		- [[lsg/How To/Read a Published Garden Offline]] tells readers how to use it.
	- ## The key
		- | Value | Result |
		  | --- | --- |
		  | `true` | The site offers offline reading. |
		  | `false` | The site publishes a worker that deletes readers' copies and unregisters itself. Pages carry no tag or menu item. |
		  | No key | The site is the same as before. |
		- Any other value, a repeated key or a key inside another entry, such as `:default-home {:lsg/offline true}`, stops the build before it creates the output directory. `lsg publish` exits 1 with empty stdout and writes a message to stderr:
		- ~~~text
		  logseq/config.edn: :lsg/offline must be true or false, not "yes"
		  ~~~
	- ## What the build writes
		- | File | Written for | Contents | Limit |
		  | --- | --- | --- | --- |
		  | `lsg-sw.js` | `true` and `false` | The service worker | 5 KB, 2 KB gzip |
		  | `lsg-offline.js` | `true` | The two menu items and the saved-copy notice | 2 KB, 1 KB gzip |
		- Both files sit at the site root with fixed names, and are not listed in [[lsg/Ref/Site Manifest]]. With `--readable-urls`, the names are reserved like the site's other generated files, for `true` and for `false`: a page titled `lsg-sw.js` stops the build before it creates the output directory.
		- `lsg-offline.js` depends only on the `lsg` version. `lsg-sw.js` holds a build identifier made from the `lsg` version and the addresses of the published pages. A publish that edits a block leaves `lsg-sw.js` unchanged; a publish that adds or removes a page changes it.
		- With `true`, every full page, which means the home page, the journals feed, All pages, the graph view, namespace pages and page folders, gains `<script defer src="lsg-offline.js">` in its head and two hidden items in the More menu: Save pages for offline reading and Remove offline copies. Redirect pages and the fragments that scripts fetch gain nothing. A page grows by 0.25 KB.
	- ## What a reader's browser does
		- Before the reader chooses Save, `lsg-offline.js` reads whether the garden's worker is registered and shows Save or Remove. It registers no worker, opens no cache and writes no storage.
		- Save registers `lsg-sw.js` with the folder that holds it as its scope, such as `/gitpa/`. The worker then handles only requests to that folder of the same site, and passes every other request to the network.
		- The worker keeps each response with status 200 that a page of the garden requests, in one cache named `lsg-offline:` followed by the scope, such as `lsg-offline:/gitpa/`. It skips audio, video, search index files and any response above 256 KB.
		- A page request goes to the network first. A response of 200 is shown and replaces the copy. A response of 404 or 410 is shown and deletes the copy. When the network fails, or gives no answer within 4 seconds, the copy is shown, with a line above the page that names the day it was saved. With no copy, the browser shows its own offline page.
		- Remove offline copies deletes the cache and unregisters the worker.
	- ## Limits
		- | Limit | Value |
		  | --- | --- |
		  | Stored responses per garden | 250 |
		  | Stored size per garden | 32 MB |
		  | Largest stored response | 256 KB |
		  | Age of a copy | 30 days |
		  | Wait before a copy answers | 4 seconds |
		- The worker keeps a record of each copy's size, save time and last use, and deletes the copy used longest ago until both totals are met. When the browser refuses a write for lack of space, the worker deletes the older half of its copies, tries once more and otherwise skips that copy; the page still loads.
		- The limits cannot be changed.
	- ## Updates and removed pages
		- Browsers check `lsg-sw.js` for changes when a reader opens a page. A new version takes over at once and keeps the copies of the old one.
		- When a new version takes over, it asks the site about each stored page with a HEAD request, two at a time, after the reader's own request, and deletes each copy answered 404 or 410. Any other answer, including 405 from a host that rejects HEAD, keeps the copy.
		- Each time the worker starts, it deletes copies older than 30 days. A page the author removed, that the reader never opens again, and whose host rejects HEAD, stays at most 30 days.
	- ## Gardens on one site, and gardens at a site root
		- Several gardens can share one site address, such as `codekiln.github.io/logseq-gardener/` and `codekiln.github.io/gitpa/`. Each worker's scope is its own folder and each cache has its own exact name, so one garden's worker never handles or deletes another's copies. Any script on that site can still read every garden's cache, so readers of these gardens trust every garden on the site.
		- A browser deletes a site's stored data as a whole when it needs space, so every garden on that site loses its copies together. Each garden can store up to 32 MB.
		- A garden at the root of a site, such as `owner.github.io`, offers neither menu item, because its worker would control every other garden on that site.
		- Pages opened from disk, and sites served over plain `http://` from any host other than `localhost`, offer neither item.
	- ## Turn offline reading off
		- 1. Change the key to `:lsg/offline false` and publish. Pages lose the menu items, and `lsg-sw.js` becomes a worker that deletes the reader's copies and unregisters itself the next time the reader opens a page online.
		- 2. Keep publishing with `false` while readers may still return. A registered worker keeps running when its file disappears, so removing the key first leaves readers' workers and copies in place until each reader chooses Remove offline copies or clears the site's data.
		- 3. Remove the key.
	- ## Size budgets
		- | Budget | Limit |
		  | --- | --- |
		  | `lsg-sw.js` | 5 KB, 2 KB gzip |
		  | `lsg-offline.js` | 2 KB, 1 KB gzip |
		  | Page growth with the key | 0.6 KB, 0.3 KB gzip |
		  | Page growth when the garden gains 2,000 pages | 0 |
		  | First paint of any page before Save | The existing 100 KB of data and 16 KB of script |
		  | Online page load through an active worker | 50 ms slower at the median |
		  | Page memory after Save | 2 MB more |
		  | Stored copies | 250 responses, 32 MB |
		  | Added build time on the encode garden | 1,000 ms, above the run-to-run noise of a 20 s build |
		- `mise run parity:budgets`, `mise run offline:browser-test` and `mise run parity:build-bench` check these; [[lsg/Ref/Logseq Compatibility/Performance]] records the measurements.
	- ## What has been checked
		- Chromium checks run with `mise run offline:browser-test`, in CI: nothing is registered or stored before Save; Save and Remove from the keyboard; the scope under a path; the separate caches of gardens at `/a/` and `/ab/`; no item at a site root or from disk; network first; a copy with its notice when the server stops, when the browser is offline and after 4 seconds; the browser's offline page for an unvisited page; what is and is not stored; deletion on 404, after a new publish and after 30 days; a host that rejects HEAD; the limits and a quota error; and the removing worker, which reaches a reader who only opens pages. Unit and integration tests cover the key, the worker's bytes, the build identifier and the tags on every kind of page, with and without `--readable-urls`.
		- Checks on phones, tablets and other browsers are recorded in the verification record of [Issue #608](https://github.com/codekiln/logseq-gardener/issues/608) and its pull request. A check that could not run says so.
	- ## Related
		- [[lsg/Ref/Publish Options]] lists the options of `lsg publish`.
		- [[lsg/Ref/Install Options]] describes the key that makes a garden installable as an app.
