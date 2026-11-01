alias:: [[lsg-site.json]]
tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Publish Options]], [[lsg/Ref/Plugin Host API]]

- # Site Manifest Reference
	- ## Overview
		- Every build writes `lsg-site.json` at the output root, so build tools can read the site's routes without parsing HTML. Generated pages never request it.
		- ```json
		  {
		    "apiVersion": 1,
		    "revision": "102ef744a7e1b199",
		    "pages": [
		      {"id": null, "title": "Journals", "url": "index.html", "kind": "home"},
		      {"id": "show/episode 1", "title": "Show/Episode 1", "url": "p-0e2769dd320a1844.html", "kind": "page"}
		    ],
		    "assets": ["assets/a-ffc572edff75a892.mp4"]
		  }
		  ```
	- ## Fields
		- ### apiVersion
			- The manifest format version, `1`.
		- ### revision
			- A digest that changes when any generated file changes, except the graph view's layout data.
		- ### pages
			- One entry for each published page, sorted by `url`. Withheld and excluded pages are absent. The graph view is not listed.
			- | Field | Value |
			  | --- | --- |
			  | `id` | The page's lower-cased name, as Logseq identifies pages. Generated lists, All pages and the journals feed, have `null`. A homepage at `index.html` has its page's id. |
			  | `title` | The page's title. |
			  | `url` | The page's address relative to the output root: its file, or with `--readable-urls` its folder, such as `GitP/A/Session/`, whose file is that folder's `index.html`. |
			  | `kind` | `page`, `journal`, `namespace` (a generated namespace page, which has no source file), `home` (`index.html`) or `list`. |
		- ### assets
			- The paths of the copied assets, sorted. Assets that only withheld or excluded pages use are absent.
		- ### plugins
			- Present only in a build with plugins that have a browser module. Each such plugin's entry holds its `id`, `version`, `module`, `styles` and `activation`, with paths relative to the output root. A plugin that runs only at build time is not listed. [[lsg/Ref/Plugin Host API]] describes these fields.
	- ## Technical details
		- Because `pages` is sorted by `url` and `assets` by path, identical input produces an identical file.
		- The `revision` of `lsg-site.json` covers the generated files. The `revision` that build plugins receive in their view is a different digest, as [[lsg/Ref/Build Plugin Protocol]] describes.
		- [[lsg/Ref/Publish Options]] describes the folder layout of `--readable-urls`.
