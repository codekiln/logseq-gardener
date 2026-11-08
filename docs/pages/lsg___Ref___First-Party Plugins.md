tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Plugin Host API]], [[lsg/Ref/Build Plugin Protocol]], [[lsg/How To/Add a Site Plugin]]

- # First-Party Plugins Reference
	- ## Overview
		- `lsg` ships plugins inside its binary. A garden turns them on with one top-level key in its own `logseq/config.edn`, so every site that turns a plugin on runs the same code at the same `lsg` version. Logseq ignores the key.
		- ~~~clojure
		  :lsg/plugins [:podcast-episode]
		  ~~~
		- A garden without the key, or with `:lsg/plugins []`, builds the same site as an `lsg` without these plugins: no page carries plugin metadata and no page loads plugin code.
		- The key is the only switch. `lsg publish` has no flag for first-party plugins, and Rust callers of `publish_site_with_options` get the plugins the key names.
	- ## The key
		- The value is a vector of keywords. Each keyword is the id of a plugin this `lsg` ships. Order has no effect: plugins run in id order.
		- The build stops before it creates the output directory, exits 1 with empty stdout and names `logseq/config.edn`, the entry, the `lsg` version and the shipped plugins when:
			- the value is not a vector, such as `:lsg/plugins :podcast-episode`;
			- an entry is not a keyword, such as `"podcast-episode"`;
			- an entry names a plugin this `lsg` does not ship, such as a misspelling or a plugin from a newer `lsg`;
			- an entry repeats a name;
			- a `--plugin` manifest, or a Rust build plugin, has the id of a turned-on first-party plugin.
		- ~~~text
		  logseq/config.edn: :lsg/plugins names :podcast-epsiode, which lsg 0.0.38 does not ship. It ships :podcast-episode.
		  ~~~
		- The same key inside a nested map has no effect.
	- ## What a turned-on plugin adds
		- Its module and stylesheet in `plugins/<id>/`, and an entry in the `plugins` list of `lsg-site.json` with the `lsg` version as its version. The build reads no file outside the garden for a first-party plugin.
		- A metadata element on every page except the graph view, and the plugin host, which loads after first paint, as [[lsg/Ref/Plugin Host API]] describes.
		- An entry in the page metadata of each page the plugin applies to. Each first-party plugin uses `on-demand` activation, so a page without its entry loads none of its files, unless the reader arrived from a page that had one.
		- Search files are the same as without the plugin.
	- ## podcast-episode
		- A player that keeps an episode's recording playing while the reader moves between pages, and dialogue timestamps that seek it.
		- ### Pages it applies to
			- A published page whose `logseq-entity::` value names `[[Logseq/Entity/Podcast/Episode]]`. The name matches without case and through aliases, as Logseq matches page names. A subtype such as `[[Logseq/Entity/Podcast/Episode/GitP/A]]` alone does not match, so a page lists the base type beside it:
			- ~~~markdown
			  logseq-entity:: [[Logseq/Entity/Podcast/Episode/GitP/A]], [[Logseq/Entity/Podcast/Episode]]
			  ~~~
			- The plugin reads the authored value even when `:block-hidden-properties` hides `logseq-entity` from readers. The value reaches no generated file through the plugin.
			- The recording is the first audio embed, such as `![Episode recording](https://example.com/full.mp3)` or `![Recording](../assets/full.mp3)`, among the descendants of the page's first heading block. A page of the type without one gets no entry and one build diagnostic naming the page.
		- ### Entry
			- | Key | Value |
			  | --- | --- |
			  | `recording` | The recording's URL: an `https` URL as written, or the published asset's path relative to the site root. |
			  | `title` | The text of the page's first heading. |
		- ### In the browser
			- On the first episode page, the module moves the page's `<audio>` element into a player fixed to the bottom of the window, with the native controls and the episode's title linked to its page. The player stays while the reader follows links, goes Back or goes Forward.
			- On another episode page, the page's recording becomes a Play button. The playing recording stays until the reader presses it; the player then plays the new recording from its start.
			- A dialogue timestamp on the episode page that the reader is on plays its recording in the player from that time. The module cancels the `lsg:media-seek` event, so the page's own `<audio>` stays paused.
			- When the module is absent or fails, the page shows its own `<audio>` element and dialogue timestamps seek it, as without the plugin.
		- ### Budget
			- The `[plugins.podcast-episode]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml): at most 8 KB gzip of module and stylesheet, 4 KB gzip of added page size on the largest fixture episode, 1 MB of added phone heap after playing across 20 navigations, and 0.05 ms of added build time per published page. `mise run parity:budgets` and `mise run parity:build-bench` check the size and build rows in CI.
	- ## Budget rows for every first-party plugin
		- Each first-party plugin has a `[plugins.<id>]` table in `scripts/parity/budgets.toml` with `script_styles_gzip_kb`, `page_growth_gzip_kb`, `phone_heap_growth_mb` and `build_ms_per_page`; `mise run test` fails when a shipped plugin lacks one. With any first-party plugin on, a page without a plugin entry grows by at most `page_overhead_gzip_kb` of the `[plugins]` table, and the size plugins add to a page stays the same when the garden gains 2,000 pages (`garden_growth_kb`).
