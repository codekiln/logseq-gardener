tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Add a Site Plugin]], [[lsg/Ref/Plugin Host API]]

- # Build Plugin Protocol Reference
	- ## Overview
		- A build plugin adds content to pages while `lsg` builds the site, such as a list of episodes under a heading of the homepage, or the chapter data an episode player reads. It replaces editing the generated HTML after the build.
		- A build plugin is a program that a plugin manifest's `build` section names. `lsg` runs it once per build, writes a JSON view of the published pages to its standard input, and reads a JSON list of contributions from its standard output. [[lsg/Ref/Plugin Host API]] describes the rest of the manifest.
		- Without a `build` section and without in-process plugins, the build creates no view and runs no plugin code. [`tests/fixtures/plugins/episode-list`](https://github.com/codekiln/logseq-gardener/tree/main/tests/fixtures/plugins/episode-list) is a complete build plugin.
	- ## The build section
		- | Field | Meaning |
		  | --- | --- |
		  | `command` | The program and its arguments. `lsg` runs it once per build, without a shell, in the manifest's folder. A program name containing `/` is a file in that folder; otherwise it is looked up on `PATH`. |
		  | `blocks` | `true` to receive every block of every page. Default `false`: pages only. |
		  | `timeoutSeconds` | How long the program may run, 1 to 600. Default 60. |
		- A manifest with `build` may leave out `module` and `activation`. It then runs only at build time and adds nothing to pages beyond its contributions.
	- ## The view
		- `lsg` writes one JSON document to the program's standard input and closes it:
		- ```json
		  {
		    "apiVersion": 1,
		    "plugin": {"id": "episode-list", "version": "1.0.0"},
		    "view": {
		      "apiVersion": 1,
		      "revision": "5b0c2a4f1e9d8c7b",
		      "pages": [
		        {"id": "show/episode 1", "title": "Show/Episode 1", "url": "p-0e2769dd320a1844.html", "kind": "page", "properties": {"recording": "https://example.com/1.mp3"}}
		      ],
		      "blocks": [
		        {"id": "1", "pageId": "show/episode 1", "text": "Intro", "children": ["1.1"]},
		        {"id": "1.1", "pageId": "show/episode 1", "text": "Detail", "children": []}
		      ]
		    }
		  }
		  ```
		- `pages` holds every published page, journal and namespace page, sorted by `id`, after namespace selection, `public::` rules and alias resolution. Withheld and excluded pages are absent, and no field holds a source path.
		- `properties` holds the values of the plugin's `pageProperties` that the page shows readers, the same values the browser module receives.
		- `blocks` lists each page's blocks in document order, page by page, and is empty unless the plugin asked for blocks.
			- A block's `id` is its `id::` UUID when no other published block has the same UUID, and its position path, such as `3.1.2`, otherwise. The pair of `pageId` and `id` names a block.
			- `text` is the block's Markdown without its property lines and without its children.
		- `revision` is a digest of the view and the `lsg` version. It differs from the `lsg-site.json` revision, which covers the generated files.
	- ## Contributions
		- The program writes one JSON document to standard output. Its standard error passes through to `lsg`'s.
		- ```json
		  {
		    "apiVersion": 1,
		    "contributions": [
		      {"kind": "rows", "pageId": "home", "blockId": "4",
		       "rows": [{"content": [{"page": "show/episode 1"}, {"text": " — 42 min"}], "children": []}]},
		      {"kind": "metadata", "pageId": "show/episode 1", "values": {"chapters": [{"start": 0, "title": "Intro"}]}}
		    ]
		  }
		  ```
		- ### rows
			- A `rows` contribution adds bullets after the children of block `blockId`, or at the end of the page's outline when `blockId` is absent.
			- A row's `content` is a list of these parts:
				- `{"text"}`: text, shown as written. It is never read as Markdown or HTML.
				- `{"page", "label"?}`: a link to a page.
				- `{"page", "block", "label"?}`: a link to a block.
				- `{"url", "label"}`: a link with an `http`, `https` or `mailto` URL.
			- A page or block link shows the page's title unless it has a label. A row's `children` holds its child rows.
		- ### metadata
			- A `metadata` contribution adds `values` to the plugin's entry in a page's metadata, beside its page property values, for its browser module's `onPageAdopted` listener.
			- It needs a `module`. A key may not repeat one of the plugin's `pageProperties`, and a page's entry is at most 64 KB of JSON.
		- ### How rows appear
			- Rows render inside a `<ul>` with the class `lsg-derived` and `data-lsg-plugin="<id>"`.
			- They are added after linked references, search records, the graph, All pages counts and embeds are built, so they create no backlink, search result or graph edge, and an embed of the target block shows only its authored children.
			- Rows carry no block anchor of their own.
			- Contributions to the same place appear in plugin id order, then in the order each plugin returned them.
	- ## Failures
		- The build stops before it creates the output directory, naming the plugin and the problem, when the program:
			- cannot start, exits non-zero, or runs past its timeout, in which case it is stopped;
			- writes more than 64 MB, or anything other than one contributions document;
			- answers with another `apiVersion`.
		- The build also stops, naming the plugin and the contribution's position, when a contribution names a page or block outside the plugin's view, links to one, uses another URL scheme, has no rows, or breaks a metadata rule.
	- ## In-process plugins
		- Rust callers can add in-process plugins to `SiteOptions::build_plugins`. Any type that implements `contributions::BuildPlugin` receives the same view, as a `publication::PluginView`, and returns the same contributions.
		- Their ids share one namespace with manifest ids. In-process plugins have no browser module, so they contribute rows only.
		- [`examples/build_plugin.rs`](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/examples/build_plugin.rs) registers one.
		- `publication::select` and `contributions::resolve` give other tools the same page selection and contribution checks that the publisher uses.
	- ## Trust
		- A build command runs on the publishing machine with your file access. The view limits what `lsg` hands the program; it does not isolate the program, which can read anything you can. [[lsg/Static Publishing]] describes what that means for choosing build plugins.
