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
		  logseq/config.edn: :lsg/plugins names :podcast-epsiode, which lsg 0.0.39 does not ship. It ships :ableton-set, :podcast-episode and :sequence-header.
		  ~~~
		- The same key inside a nested map has no effect.
	- ## What a turned-on plugin adds
		- Its module and stylesheet in `plugins/<id>/`, and an entry in the `plugins` list of `lsg-site.json` with the `lsg` version as its version. The build reads no file outside the garden for a first-party plugin.
		- A metadata element on every page except the graph view, and the plugin host, which loads after first paint, as [[lsg/Ref/Plugin Host API]] describes.
		- An entry in the page metadata of each page the plugin applies to. Each first-party plugin uses `on-demand` activation, so a page without its entry loads none of its files, unless the reader arrived from a page that had one.
		- Search files are the same as without the plugin.
	- ## ableton-set
		- A track view at the top of an Ableton Live Set page: one column per track, side by side as in Live's Session View, with a group's tracks inside the group's column. Each column shows the track's type, input and output routing, MIDI channel, monitor and arm state, and devices in chain order, and links to the track's page. The page's outline stays below the view.
		- ### Pages it applies to
			- A published page whose `logseq-entity::` value names `[[Logseq/Entity/Ableton/Set]]`, matched as `podcast-episode` matches its type, and which embeds at least one published group or track page.
			- The Set page embeds each top-level group and track page once, in Live's order. A group page, typed `[[Logseq/Entity/Ableton/Track/Group]]`, embeds its tracks the same way, and a track page is typed `[[Logseq/Entity/Ableton/Track/Track]]`. Each of these pages has one heading that names it, and writes its settings as block properties of that heading:
			- ~~~markdown
			  logseq-entity:: [[Logseq/Entity/Ableton/Set]]
			  
			  - # Evening Set
			    live-version:: 11.3.43
			    tempo:: 96
			    time-signature:: 4/4
			  	- {{embed [[Evening/Set/Track/Synths]]}}
			  	- {{embed [[Evening/Set/Track/Commentary]]}}
			  ~~~
			- ~~~markdown
			  logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]
			  
			  - # Commentary
			    type:: audio
			    audio-from:: AudioIn/External/S0
			    audio-to:: AudioOut/Master
			    monitor:: off
			    arm:: false
			    devices:: Compressor, EQ Eight, Limiter
			  	- Records the spoken afterword.
			  ~~~
			- The build step reads these block properties from the garden's source. `lsg` publishes no block property as a row, so the track view is where a reader sees them.
			- An embed adds no column when it names a withheld, excluded, missing or ambiguous page, a page that is neither a group nor a track, the Set itself, or a group already being read. Such a page's title and URL reach no generated file through the plugin. A page the Set only links to adds no column.
			- A property the garden hides with `:block-hidden-properties`, such as `#{:midi-to}`, stays out of the entry.
			- A Set page that embeds no published group or track page gets no entry and one build diagnostic naming the page. So does a Set whose entry would exceed 64,000 bytes.
		- ### Entry
			- | Key | Value |
			  | --- | --- |
			  | `title` | The text of the Set page's first heading, or the page title without one. |
			  | `live-version`, `tempo`, `time-signature` | The heading's block properties of those names, as written. Each is left out when the heading does not state it. |
			  | `tracks` | The Set's top-level groups and tracks, in page order. |
			- Each item of `tracks`:
			- | Key | Value |
			  | --- | --- |
			  | `title` | The text of the track page's first heading, or the last segment of the page title without one. |
			  | `url` | The track page's published path relative to the site root. |
			  | `type`, `midi-channel`, `monitor` | The heading's block properties of those names, as written. Each is left out when the heading does not state it. |
			  | `audio-from`, `audio-to`, `midi-from`, `midi-to` | The heading's block property of that name, left out when the heading does not state it. A value that starts with a reference to one published page, such as `[[Evening/Set/Track/Synths]] Track In`, is `{title, url, detail}`: the page's title as a column shows it, its published path relative to the site root, and the text after the reference, left out when there is none. A value that starts with a reference to any other page is the last segment of the name it writes, then the text after it. Any other value is as written. |
			  | `arm` | `true` or `false` for `arm:: true` or `arm:: false`. Left out for any other value. |
			  | `devices` | The names in `devices::`, split at commas, in chain order. Left out without the property. |
			  | `tracks` | On a group page only: the group's tracks, in the same form. A track page has no `tracks` key. |
		- ### In the browser
			- The module puts a `<section class="lsg-ableton-set">` with the accessible name `Track view of <title>` above the page's outline. A caption gives the Live version, the tempo, the time signature and the number of tracks, counting groups.
			- Tracks are a row of columns that scrolls sideways inside the section, so the page itself never scrolls sideways, at any width. A group is a column that holds its own settings and then its tracks' columns.
			- A column's heading is the track's name, linked to its page, with its type beside it: `Audio`, `MIDI`, `Return` or `Group` for those `type::` values, and any other value as written. A coloured top edge repeats the type.
			- Settings follow the order of Live's In/Out section: Audio from, MIDI from, MIDI channel, Monitor, Audio to, MIDI to, Arm. A routing that names another page of the site shows that page's title as a link, then the rest of the value, such as **Synths** Track In; any other routing appears as the page writes it. Monitor shows Live's three states, In, Auto and Off, with the track's state marked; a screen reader hears that state alone. Arm reads `Armed` or `Off`.
			- Devices are a list in chain order.
			- Keyboard focus reaches each track link and shows an outline. The view uses the site's theme colours in light and dark.
			- After each navigation the module draws the view of the new page, or none when the page has no entry.
			- When the module is absent or fails, the page shows its outline, with the embedded group and track pages, as it does without the plugin.
		- ### Budget
			- The `[plugins.ableton-set]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml): at most 4 KB gzip of module and stylesheet, 2 KB gzip of added page size, 0.5 MB of added phone heap, and 0.05 ms of added build time per published page.
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
			- The player also has a **Full recording** link to the recording, which opens in a new tab so the player keeps playing, and a status that reads Playing, Paused or Ended and reports a recording that fails to load.
			- For screen readers, the current chapter's link in the trail carries `aria-current="true"`, and the player's `<audio>` element is named `Full episode recording`. The status is a polite live region, so a screen reader reads each change after its current speech.
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
