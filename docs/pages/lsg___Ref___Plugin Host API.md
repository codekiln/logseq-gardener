tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Add a Site Plugin]], [[lsg/Ref/Build Plugin Protocol]], [[lsg/Ref/Host Integration API]]

- # Plugin Host API Reference
	- ## Overview
		- A site plugin adds browser behaviour to a published site, such as a recording player that keeps playing while the reader moves between pages. Each plugin has a JSON manifest, which `lsg publish --plugin PATH` reads, and a browser module, which the plugin host imports into published pages. The host API version is `1`.
		- This page describes the manifest, the data each page carries for plugins, the `host` object a module receives, and the navigation the host performs. [[lsg/Ref/Build Plugin Protocol]] describes the `build` section, which adds content to pages at build time. [[lsg/How To/Add a Site Plugin]] shows a complete manifest and module.
		- Rust callers set `SiteOptions::plugins` to the manifest paths.
	- ## Manifest
		- | Field | Meaning |
		  | --- | --- |
		  | `id` | Lower-case letters, digits and single hyphens; unique in the build. The plugin's files go to `plugins/<id>/`. |
		  | `apiVersion` | The host API version. This build supports `1`. |
		  | `version` | The plugin's own version, copied to `lsg-site.json`. |
		  | `module` | An ES module (`.js` or `.mjs`), relative to the manifest's folder. Optional with `build`. |
		  | `styles` | Optional stylesheets (`.css`), relative to the manifest's folder. The host adds each once. Needs `module`. |
		  | `activation` | `after-load` imports the module on every page; `on-demand` imports it on the first page whose metadata has an entry for the plugin. Present exactly when `module` is. |
		  | `pageProperties` | Optional page property names the plugin reads. |
		  | `build` | Optional command that runs once per build, as [[lsg/Ref/Build Plugin Protocol]] describes. |
		- ### Manifest checks
			- The build checks every manifest before it creates the output directory, and stops with the manifest's path and the problem when:
				- a field is missing, unknown or malformed;
				- `apiVersion` is unsupported;
				- two manifests share an `id`;
				- a manifest has neither `module` nor `build`;
				- a module, stylesheet or build program path is absolute, contains `..`, passes through a symlink or is not a regular file;
				- a stylesheet name collides with the module or another stylesheet of the plugin, ignoring letter case.
			- Plugins are ordered by `id`, so the order of `--plugin` arguments does not change the output.
			- The SDK's `plugins::read_manifests` runs the same checks for other callers.
			- [`tests/fixtures/plugins/recorder`](https://github.com/codekiln/logseq-gardener/tree/main/tests/fixtures/plugins/recorder) is a complete example.
		- Each plugin with a browser module is listed under `plugins` in [[lsg/Ref/Site Manifest]].
	- ## What a page carries
		- With at least one plugin that has a `module`, every page except the graph view gets two elements before the end of its head:
			- A `<script type="application/json" data-lsg-page>` element with the page's `id`, `title` and `url`, the site `revision`, and `plugins`. For each plugin, `plugins` holds the values of its `pageProperties` that the page shows readers, as authored text, and the values its build command contributed for the page.
			- A module script that imports `lsg-host-<digest>.mjs` after first paint.
		- A property hidden by `:block-hidden-properties`, a page without page properties turned on (`:publishing/page-properties?`) and a withheld page give the plugin nothing. A page with none of a plugin's properties or contributions has no entry for it.
		- Without plugins, or with only build-time plugins, pages carry neither element and request no host or plugin code.
	- ## Module
		- The module's default export is `{ id, apiVersion: 1, setup(host) }`, with `id` matching the manifest.
		- The host checks the export, then calls `setup` once per document. `setup` may return a cleanup function.
		- A mismatched export or a failing import is logged to the console and skipped, and the page keeps working.
		- If `setup` throws, the host removes that plugin's regions and listeners and logs the error, and the other plugins keep working.
		- When the page unloads, the host aborts the page signal, calls every cleanup function once and removes the plugins' regions.
	- ## The host object
		- | Member | Description |
		  | --- | --- |
		  | `apiVersion` | `1`. |
		  | `document` | The document. |
		  | `baseURL` | The site root. |
		  | `onPageAdopted(listener)` | Calls `listener` now for the current page and after every later navigation. Returns a function that removes the listener. |
		  | `persistentRegion(localId)` | Returns `{ element, dispose }`. `element` is a `<div data-lsg-plugin="<id>" data-lsg-region="<localId>">` at the end of the body, outside `<main>`, kept across navigations. |
		- ### onPageAdopted
			- The listener receives `{ page: { id, title, url }, main, revision, metadata, signal }`:
				- `page` names the adopted page, and `main` is its `<main>` element.
				- `revision` is the site revision.
				- `metadata` is this plugin's entry from the page metadata, or `{}`.
				- `signal` is aborted before the page is replaced. Pass it to `addEventListener` for listeners that belong to one page.
	- ## Navigation
		- On a site with plugins, the host owns navigation. A plain left click on a link to another page of the site, and Back and Forward between such pages, adopt the next page in the same document. The host takes these steps in order:
			- Fetch and parse the page. A response overtaken by a newer navigation is dropped.
			- Compare its `revision` with the current page's. When they differ, the site was rebuilt: the current page stays, and a notice offers a link that loads the new version normally, so a playing recording is not cut off.
			- Abort the current page's signal.
			- Replace `<main>`, the title and the page metadata, update the left sidebar and the header's page-specific buttons, add any styles and scripts the new page needs, and push a history entry.
			- Run `lsg`'s page initializers on the new `<main>`: outline arrows and zoom, references, All pages, journals, queries and theme controls. Other scripts register more with `window.logseqGardenerPage`, which [[lsg/Ref/Host Integration API]] describes.
			- Notify plugins.
			- Reveal the fragment, as a full page load would, and move focus to it.
		- A fetch failure, or a page without metadata such as the graph view, loads normally.
		- Modified clicks, links with a `target` or `download` attribute, and links within the same page keep their usual behaviour.
	- ## Trust
		- Plugins run in the reader's browser with the page's full access; the publisher does not sandbox them. The publisher limits the data that reaches a plugin to the page properties readers can already see. [[lsg/Static Publishing]] describes what that means for choosing plugins.
