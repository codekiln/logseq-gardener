tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Plugin Host API]], [[lsg/Ref/Published Pages]], [[lsg/Ref/Site Navigation]], [[lsg/Ref/Search Dialog]]

- # Host Integration API Reference
	- ## Overview
		- A host is a page or application that shows a published site's pages inside its own navigation: it replaces `<main>` without a full page load, keeps a media player playing across pages, or serves pages at addresses of its own. Gitpa's podcast site is one. When a build has a plugin with a browser module, `lsg`'s own plugin host does this, as [[lsg/Ref/Plugin Host API]] describes. It calls the outline, references, All pages and theme functions and `window.logseqGardenerRevealFragment` itself, and calls neither `window.logseqGardenerInitProperties` nor `window.logseqGardenerInitSearch`.
		- The site's scripts expose browser functions on `window`, a `lsg:media-seek` event, `link` metadata and data attributes. A host calls the functions after it inserts new content, so the new content gets the behaviour a full page load gives it.
	- ## Order of calls
		- A host that replaces page content calls the functions in this order:
			- Adopt the destination URL and insert the new `<main>`, keeping the body-level search dialog. Update the `link[rel="lsg-search"]` metadata from the incoming page.
			- `window.logseqGardenerInitProperties?.(main)`
			- `window.logseqGardenerInitOutline(main)`
			- `window.logseqGardenerInitReferences(main)`, `window.logseqGardenerAllPages.init(main)` on an All pages screen, `window.logseqGardenerInitTheme(navigation)` after inserting an Appearance selector, and `window.logseqGardenerInitSearch(document, options)`
			- `window.logseqGardenerRevealFragment(location.hash, {scroll: false})`
			- The host's own focus and scrolling.
	- ## Functions
		- ### window.logseqGardenerInitOutline
			- `window.logseqGardenerInitOutline(main)` sets up the collapse arrows and block zoom of the outline in `main`, an imported `<main>` element.
			- Call it before the host's route and focus hooks.
			- Calling it again keeps the arrows and the collapse state. Journal outlines that the journals feed appends set themselves up.
			- `lsg-outline.js` defines it. `lsg-shell.js` loads that script on every page after the page has loaded.
		- ### window.logseqGardenerRevealFragment
			- `window.logseqGardenerRevealFragment(hash, {scroll: false})` opens the closed disclosures that contain the fragment's target, such as collapsed outline blocks and the Properties disclosure, and returns the target element, or `null` when there is none.
			- `hash` is an encoded fragment that begins with `#`, and defaults to `location.hash`. The function opens only the target's containing disclosures. Without `{scroll: false}`, it also scrolls the target into view; with it, scrolling and focus stay with the host.
			- Call it after the new page is in place and before the host's own fragment focus and scrolling. For a search result, load the resolved page through the host's normal navigation hook, then call it with the result's fragment.
			- Ordinary static navigation uses the same helper on page load and on hash changes. It opens Properties for a fragment without changing the reader's remembered choice.
			- An inline script on every page defines it.
		- ### window.logseqGardenerInitProperties
			- `window.logseqGardenerInitProperties?.(main)` sets up the Properties disclosure in `main`, an imported `<main>` element.
			- Call it after adopting the destination URL and before fragment reveal or focus handling.
			- It is available early on every page of a site with page properties turned on, including pages with no visible rows. Calling it again keeps an initialized disclosure's current state.
			- Keep the initial `link[rel=lsg-properties]` element. Its asset folder sets the scope of the saved choice across readable nested routes.
		- ### window.logseqGardenerInitReferences
			- `window.logseqGardenerInitReferences(root)`, also available as `window.logseqGardenerInitBacklinks`, sets up the reference sections and the Linked References filter in `root`.
			- Call it after inserting the new content. It loads sections as readers open them and installs the filter dialog, and changes neither navigation nor media playback.
			- Calling it again is safe. It is separate from `window.logseqGardenerRevealFragment`.
			- `lsg-references.js` defines it, and loads after the page's `load` event.
		- ### window.logseqGardenerAllPages.init
			- `window.logseqGardenerAllPages.init(root)` sets up the All pages screen in `root`.
			- Call it after swapping an All pages screen into the page.
			- `lsg-all-pages.js` defines it, and loads only on All pages screens.
		- ### window.logseqGardenerInitTheme
			- `window.logseqGardenerInitTheme(navigation)` makes a native Appearance selector that the host inserted reflect the active choice. The selector is a `<select data-lsg-theme-select>` with `system`, `light` and `dark` options inside an element with `data-lsg-theme-control`, and `navigation` is the imported element that contains it.
			- Call it after inserting the selector. Calling it again is safe, and it replaces neither page content nor media players.
			- `lsg-theme.js` defines it. Every page loads that script with `defer`, so it exists before `DOMContentLoaded`.
		- ### window.logseqGardenerInitSearch
			- `window.logseqGardenerInitSearch(document, options)` points the search button and dialog at the current document's search publication.
			- Before calling it, update the current document's `link[rel="lsg-search"]` from the incoming page, and remove the link when the incoming page has no search publication.
			- ```javascript
			  window.logseqGardenerInitSearch(document, {
			    resolvePage: (canonicalTitle, publishedHref) => readableRouteFor(canonicalTitle)
			  });
			  ```
			- `resolvePage` is optional. It receives a result page's stored title and its published address, and returns the host's route for it, without a fragment. It stays configured on later calls, and `resolvePage: null` restores the default links, which resolve beside the manifest, also from nested current pages.
			- The function is idempotent and reads the current document's metadata. It keeps one body-level dialog outside `<main>`, so hosts can replace `<main>` normally and should keep that dialog when they replace page content.
			- When the publication has changed or is missing, it closes the dialog, discards pending and cached data and shows or hides the search button.
			- [[lsg/Ref/Search Dialog]] describes how the dialog behaves inside a persistent host.
			- `lsg-search-boot.js` defines it. Every page of a site published with `--search` loads that script with `defer`, so it exists before `DOMContentLoaded`.
		- ### window.logseqGardenerPage
			- `window.logseqGardenerPage(init)` registers a page initializer with `lsg`'s plugin host. `init(main, signal)` runs now and after every navigation the plugin host performs, with the new `<main>` and the page's abort signal.
			- The plugin host module defines it, so it exists on pages of a site built with a plugin module. [[lsg/Ref/Plugin Host API]] describes the navigation that runs it.
	- ## Events
		- ### lsg:media-seek
			- A host with a persistent player handles dialogue timestamps through `lsg:media-seek`, a bubbling, cancelable custom event that the timestamp helper dispatches when a reader activates a timestamp.
			- | `detail` field | Value |
			  | --- | --- |
			  | `source` | The absolute URL of the recording |
			  | `seconds` | The position in the recording, an integer |
			  | `anchor` | The timestamp link the reader activated |
			- To play through the host's player, check `source` and `seconds` against the host's current recording, call `preventDefault()` on the event and seek the host player. The helper then prevents the original click and plays nothing inline.
			- The helper uses one delegated capture listener per document, so replaced transcript controls work without another setup call.
			- A host that does not load the helper can handle the timestamp links directly: each is an ordinary link to the recording with a `#t=` fragment.
			- [[lsg/Ref/Published Pages]] describes timestamps and their inline playback.
	- ## Link metadata
		- | Element | Purpose |
		  | --- | --- |
		  | `link[rel="lsg-search"]` | The page's search manifest, in `href`, and its revision, in `data-revision`. [[lsg/Ref/Static Search Data]] describes it. |
		  | `link[rel=lsg-properties]` | The Properties script. Its folder sets the scope of the saved Properties choice. |
		  | `link[rel="lsg-dialogue"]` | The dialogue timestamp helper. |
		- A host that repackages routes rewrites the `href` of the `link[rel="lsg-dialogue"]` element along with the other local resources. The loader reads that resolved URL before navigation.
	- ## Data attributes
		- | Attribute | On | Meaning |
		  | --- | --- | --- |
		  | `data-lsg-theme` | `:root` | The active appearance, `light` or `dark`, which the theme script sets. Absent without JavaScript. |
		  | `data-lsg-theme-select` | A host's `<select>` | Marks a native Appearance selector. |
		  | `data-lsg-theme-control` | The selector's container | Marks the element that holds the selector. |
		  | `data-lsg-search-state` | The search dialog | The dialog's state, as [[lsg/Ref/Search Dialog]] describes. |
		  | `data-lsg-search-complete` | The search results list, `#lsg-search-results` | Advances after each completed result render, as [[lsg/Ref/Search Dialog]] describes. |
		  | `data-lsg-page` | A `<script type="application/json">` in the head | The page metadata that plugins read, as [[lsg/Ref/Plugin Host API]] describes. |
		  | `data-lsg-plugin`, `data-lsg-region` | A plugin's persistent region | The plugin's id and the region's local id, as [[lsg/Ref/Plugin Host API]] describes. |
	- ## Rewritten page addresses
		- A host that rewrites page addresses after the build, such as one that serves `p-….html` at a readable route, keeps Shift-click to the right sidebar working on links inside `.page-reference`, `a.page-ref`, `a.tag` and `a.block-ref-link`, and on links whose address holds `#zoom/` or `#source-block-`. The sidebar recognizes those links by their markup or their fragment.
		- `lsg` renders every page link in one of those forms, as Logseq renders each page link as `a.page-ref`: inline references, page-valued properties, Pages tagged with, reference group titles, page embed headers, query results, All pages and plugin rows. A link the host writes itself needs the same markup or a `p-*.html` address.
		- Search results follow the host's routes through the `resolvePage` option of `window.logseqGardenerInitSearch`, or through the resolver of the search engine's `resolveResult`, which [[lsg/Ref/Static Search Data]] describes.
	- ## Fragment offset
		- Every page sets `scroll-padding-top: 56px` on `html`, so a host's own fragment scrolling lands below the 48px sticky header without a rule of its own.
