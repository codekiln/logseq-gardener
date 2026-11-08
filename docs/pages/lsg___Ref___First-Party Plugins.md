tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Plugin Host API]], [[lsg/Ref/Build Plugin Protocol]], [[lsg/How To/Add a Site Plugin]]

- # First-Party Plugins Reference
	- ## Overview
		- `lsg` ships plugins inside its binary. A garden turns them on with one top-level key in its own `logseq/config.edn`, so every site that turns a plugin on runs the same code at the same `lsg` version. Logseq ignores the key.
		- ~~~clojure
		  :lsg/plugins [:podcast-episode :sequence-header]
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
		  logseq/config.edn: :lsg/plugins names :podcast-epsiode, which lsg 0.0.39 does not ship. It ships :podcast-episode and :sequence-header.
		  ~~~
		- The same key inside a nested map has no effect.
	- ## What a turned-on plugin adds
		- Its module and stylesheet in `plugins/<id>/`, and an entry in the `plugins` list of `lsg-site.json` with the `lsg` version as its version. The build reads no file outside the garden for a first-party plugin.
		- A metadata element on every page except the graph view, and the plugin host, which loads after first paint, as [[lsg/Ref/Plugin Host API]] describes.
		- An entry in the page metadata of each page the plugin applies to. Each first-party plugin uses `on-demand` activation, so a page without its entry loads none of its files, unless the reader arrived from a page that had one.
		- Search files are the same as without the plugin.
	- ## podcast-episode
		- A player that keeps an episode's recording playing while the reader moves between pages, and chapter timestamps and dialogue timestamps that seek it.
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
			  | `chapters` | The page's chapters in page order, left out when it has none. Each is `{anchor, seconds, title, parent}`: the `source-block-<offset>` id that the page carries on the chapter's block, the start in seconds, the first line's text after the timestamp (cut at a space to at most 48 characters, ending in `…`, since the player's trail shows it and the page keeps the full text), and the index in the list of the nearest enclosing chapter, or `null`. |
			- A chapter is a block whose text starts with a timestamp: one or two digits, then one or two groups of a colon and two digits, such as `5:15`, `18:17`, `1:02:03` or `01:00:23`, followed by a space, a dash or the end of the line. The block is either an H3 to H6 heading anywhere on the page, or any bullet at any depth under an H2 heading whose text is `Chapters`. A chapter's enclosing chapter is the nearest chapter among the block's ancestors. A timestamp later in the block, inside code or emphasis, or after a task marker or priority leaves the block as text:
			- ~~~markdown
			  - ## Chapters
			  	- ### 18:17 Enter [[Microfreak/Preset/260 8bitfi]]
			  		- #### 19:30 - Old warbling
			  		- [[My Notes]]
			  			- 20:05 - the bass comes in
			  				- 20:40 - and stays
			  ~~~
			- Here `20:05 - the bass comes in` is a chapter enclosed by `18:17 Enter Microfreak/Preset/260 8bitfi`, and `20:40 - and stays` is enclosed by `20:05`. A timestamp-led bullet outside `## Chapters` stays text.
			- The entry stays within the 64,000-byte page entry limit. When an episode's chapters do not all fit, the entry keeps them from the first, and the build lists one diagnostic naming the page and how many chapters it kept.
		- ### In the browser
			- On the first episode page, the module moves the page's `<audio>` element into a player fixed to the bottom of the window, with the native controls and the episode's title linked to its page. The player stays while the reader follows links, goes Back or goes Forward.
			- On another episode page, the page's recording becomes a Play button. The playing recording stays until the reader presses it; the player then plays the new recording from its start.
			- Each chapter's timestamp, in a heading or at the start of a bullet, becomes a link that plays the episode's recording in the player from the chapter's start. A link opened in a new tab or with a modifier key opens the recording itself at that time.
			- While a recording plays, the player shows its current chapter after the episode's title, with the chapters that enclose it, each linked to its block. The current chapter is the one with the latest start at or before the playing time.
			- A dialogue timestamp on the episode page that the reader is on plays its recording in the player from that time. The module cancels the `lsg:media-seek` event, so the page's own `<audio>` stays paused.
			- When the module is absent or fails, the page shows its own `<audio>` element, chapter timestamps stay text and dialogue timestamps seek the page's recording, as without the plugin.
		- ### Budget
			- The `[plugins.podcast-episode]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml): at most 8 KB gzip of module and stylesheet, 4 KB gzip of added page size on the largest fixture episode, 1 MB of added phone heap after playing across 20 navigations, and 0.05 ms of added build time per published page. `mise run parity:budgets` and `mise run parity:build-bench` check the size and build rows in CI.
	- ## sequence-header
		- Links to the previous and next pages in the header, beside the spacer, for a page whose `prev::` or `next::` names a published page.
		- ### Pages it applies to
			- A published page with at least one published neighbour. A neighbour comes from the page's last leading `prev::` or `next::` property, in any case, whose whole value is one page reference:
			- ~~~markdown
			  prev:: [[GitP/A/Session/26/10/22-Thu]]
			  next:: [[GitP/A/Session/26/10/26-Mon]]
			  ~~~
			- The reference must resolve, through titles and aliases, to exactly one published page other than the page itself. A private, excluded, missing or ambiguous target, or the page itself, gives no link and its title and URL reach no generated file.
			- These are the same rules the Page sequence region at the bottom of a page follows, so the header and the region always name the same pages. A direction the garden hides with `:block-hidden-properties`, such as `#{:next}`, gives no header link, as it gives no link in the region. The plugin leaves the region and the property rows as they are, and it works with `:publishing/page-sequence? false`, which turns only the region off.
		- ### Entry
			- | Key | Value |
			  | --- | --- |
			  | `prev` | `{url, title}` of the previous page: its published path relative to the site root and its page title. Left out without a previous page. |
			  | `next` | `{url, title}` of the next page, in the same form. Left out without a next page. |
		- ### In the browser
			- The module puts a Previous link with a left arrow before the header's spacer and a Next link with a right arrow after it. Each link has `rel="prev"` or `rel="next"`, shows the page title, and has the accessible name `Previous: <title>` or `Next: <title>`. A long title ends in an ellipsis, and keyboard focus shows an outline.
			- Below 640 pixels wide, each link shows its arrow alone; the title stays in the accessible name.
			- After each navigation the module replaces the links with those of the new page, or removes them when the page has no entry. Back shows the earlier page's links again.
			- When the module is absent or fails, the header has no sequence links and the Page sequence region at the bottom of the page still links both neighbours.
		- ### Budget
			- The `[plugins.sequence-header]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml): at most 1.5 KB gzip of module and stylesheet, 0.5 KB gzip of added page size, 0.2 MB of added phone heap, and 0.05 ms of added build time per published page.
	- ## Budget rows for every first-party plugin
		- Each first-party plugin has a `[plugins.<id>]` table in `scripts/parity/budgets.toml` with `script_styles_gzip_kb`, `page_growth_gzip_kb`, `phone_heap_growth_mb` and `build_ms_per_page`; `mise run test` fails when a shipped plugin lacks one. With any first-party plugin on, a page without a plugin entry grows by at most `page_overhead_gzip_kb` of the `[plugins]` table, and the size plugins add to a page stays the same when the garden gains 2,000 pages (`garden_growth_kb`).
