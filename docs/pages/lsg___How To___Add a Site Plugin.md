tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Plugin Host API]], [[lsg/Ref/Build Plugin Protocol]], [[lsg/Static Publishing]]

- # How To Add a Site Plugin
	- ## Overview
		- A site plugin adds browser behaviour to a published site, such as a recording player that keeps playing while the reader moves between pages. A build plugin adds content to pages while `lsg` builds the site, such as a list of episodes under a heading of the homepage, or the chapter data an episode player reads; it replaces editing the generated HTML after the build. One plugin can do both.
		- Each plugin is a folder with a JSON manifest, which you pass to `lsg publish` with `--plugin`.
	- ## Prerequisites
		- A garden you can publish, as [[lsg/How To/Publish Selected Namespaces]] shows.
		- Trust in the plugin's code. A browser module runs in the reader's browser with the page's full access, and a build command runs on your machine with your file access. [[lsg/Static Publishing]] explains how far `lsg` limits what plugins receive.
		- For a build plugin, the program its command runs, such as Node.js for `node build.mjs`.
	- ## Steps
		- ### 1. Write the browser module
			- If the plugin changes pages in the browser, write an ES module whose default export has the plugin's `id`, `apiVersion: 1` and a `setup(host)` function. This module keeps one audio player across pages and loads into it the recording a page names in its `recording::` property:
			- ```javascript
			  export default {
			    id: 'episode-player',
			    apiVersion: 1,
			    setup(host) {
			      const region = host.persistentRegion('player');
			      const audio = host.document.createElement('audio');
			      audio.controls = true;
			      region.element.append(audio);
			      return host.onPageAdopted(({ metadata }) => {
			        if (metadata.recording && audio.dataset.recording !== metadata.recording) {
			          audio.dataset.recording = metadata.recording;
			          audio.src = metadata.recording;
			        }
			      });
			    },
			  };
			  ```
			- `host.persistentRegion` gives the plugin an element outside `<main>` that stays across navigations, and `host.onPageAdopted` calls its listener for the current page and after every navigation. [[lsg/Ref/Plugin Host API]] lists every member of `host` and what the listener receives.
		- ### 2. Write the manifest
			- Save the manifest beside the module, for example as `lsg-plugin.json`:
			- ```json
			  {
			    "id": "episode-player",
			    "apiVersion": 1,
			    "version": "1.0.0",
			    "module": "player.mjs",
			    "styles": ["player.css"],
			    "activation": "after-load",
			    "pageProperties": ["recording"]
			  }
			  ```
			- `styles` lists stylesheets in the manifest's folder, such as a `player.css` for the player. Leave it out when the plugin has none.
			- `pageProperties` names the page properties the module reads. A page passes the values it shows readers to the module as `metadata`.
			- `activation` is `after-load` to import the module on every page, or `on-demand` to import it on the first page whose metadata has an entry for the plugin.
			- [[lsg/Ref/Plugin Host API]] describes every field and the checks the build makes.
		- ### 3. Add a build command
			- If the plugin adds content at build time, add a `build` section to the manifest. A plugin that only builds content may leave out `module` and `activation`:
			- ```json
			  {
			    "id": "episode-list",
			    "apiVersion": 1,
			    "version": "1.0.0",
			    "pageProperties": ["recording"],
			    "build": {"command": ["node", "build.mjs"], "blocks": false, "timeoutSeconds": 60}
			  }
			  ```
			- The program reads a JSON view of the published pages on standard input and writes a JSON document of contributions on standard output: rows of links and text to add under a block, and metadata values for the browser module. Set `"blocks": true` if it needs every block of every page. [[lsg/Ref/Build Plugin Protocol]] describes the view, the contributions and the rules they follow.
		- ### 4. Publish with the plugin
			- Pass the manifest's path with `--plugin`. Repeat the option to add more plugins:
			- ```sh
			  lsg publish --graph /path/to/garden --output /private/tmp/plugin-site --filename-format triple-lowbar --include GitP --plugin plugins/episode-player/lsg-plugin.json --plugin plugins/episode-list/lsg-plugin.json
			  ```
			- The build copies each module and stylesheet to `plugins/<id>/` in the site and lists each plugin with a module in `lsg-site.json`.
			- Rust callers set `SiteOptions::plugins` to the manifest paths. To run build-time code inside a Rust program instead of a separate process, add an in-process plugin to `SiteOptions::build_plugins`, as [[lsg/Ref/Build Plugin Protocol]] describes.
		- ### 5. Check the site
			- Serve the site and move between pages. The plugin's persistent region stays in place while `<main>` changes.
			- Open the browser console to see errors from the module. A module that fails to import or throws in `setup` is logged and skipped, and the page and the other plugins keep working.
	- ## Examples
		- [`tests/fixtures/plugins/recorder`](https://github.com/codekiln/logseq-gardener/tree/main/tests/fixtures/plugins/recorder) is a complete browser plugin that uses every host function.
		- [`tests/fixtures/plugins/episode-list`](https://github.com/codekiln/logseq-gardener/tree/main/tests/fixtures/plugins/episode-list) is a complete build plugin, with a browser module that reads its metadata.
		- [`examples/build_plugin.rs`](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/examples/build_plugin.rs) registers an in-process build plugin.
	- ## Troubleshooting
		- If the build stops before it creates the output directory and names a manifest, fix the field it names. The build refuses a missing, unknown or malformed field, an unsupported `apiVersion`, a repeated `id`, a manifest with neither `module` nor `build`, and a module, stylesheet or program path that is absolute, contains `..`, passes through a symlink or is not a regular file.
		- If the build stops and names a plugin, its program failed to start, exited non-zero, ran past `timeoutSeconds`, wrote something other than one contributions document, or returned a contribution the build rejected. The program's standard error appears in `lsg`'s.
		- If the module gets no `metadata` for a property, check that the page shows that property to readers: page properties must be turned on with `:publishing/page-properties?`, and the key must not be in `:block-hidden-properties`.
		- If clicks between pages reload the whole page, check that the page has page metadata; a fetch failure, or a page without metadata such as the graph view, loads normally.
