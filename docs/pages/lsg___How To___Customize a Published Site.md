tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Publish Options]], [[lsg/Ref/Published Pages]], [[lsg/Ref/Site Navigation]]

- # How To Customize a Published Site
	- ## Overview
		- `lsg publish` options and settings in the garden's `logseq/config.edn` change what a published site shows: its homepage, its journals, search, page properties, how reference sections start, the left sidebar and the site's styles. Each step below is independent; take the ones you need.
		- [[lsg/Ref/Site Navigation]] lists further settings, for the right sidebar's starting cards and the graph view's panel.
	- ## Prerequisites
		- A garden you can publish, as [[lsg/How To/Publish Selected Namespaces]] shows.
		- For the `logseq/config.edn` settings, write access to that file.
	- ## Steps
		- ### 1. Choose a homepage
			- If you want a page of the garden as the home page, add `--homepage` with its title or alias:
			- ```sh
			  lsg publish --graph /path/to/garden --output /private/tmp/site --filename-format triple-lowbar --include GitP --homepage "Ghost in the Patch"
			  ```
			- The page must be uniquely selected and visible. `lsg publish` renders it at `index.html`, keeps its own page route, and moves the complete page list to `pages.html`, which the left sidebar's All pages link opens.
			- A missing or ambiguous homepage fails the build before the output directory is created. Without `--homepage`, `index.html` is the generated page list.
		- ### 2. Publish journals
			- If you want journals in the site, add `--journals`. Every file under `journals/` becomes a page. Without `--journals`, journals are counted in the report and skipped.
			- Each journal is titled with `:journal/page-title-format` from `logseq/config.edn`, whose Logseq default is `MMM do, yyyy`. The encode garden's `journals/2026_10_23.md` becomes `2026-10-23 Fri`.
			- Journals follow the same publishing rules as pages. A file in `journals/` whose name is not a date is skipped with a diagnostic.
			- Links to a journal resolve in the configured format and in `MMM do, yyyy`, `yyyy-MM-dd` and `yyyy_MM_dd`, so `[[Feb 1st, 2025]]` opens `2025-02-01 Sat` and keeps its text.
			- With journals published, `index.html` becomes Logseq's journals feed, and `journals.html` with `--homepage`. [[lsg/Ref/Site Navigation]] describes the feed. A garden with `:feature/enable-journals? false` gets journal pages and no feed.
		- ### 3. Turn on search
			- If you want readers to search the site, add `--search`. Rust callers set `SiteOptions.search = Some(SearchOptions::default())`.
			- Serve the site over HTTP or HTTPS. Pages opened from disk hide the search button.
			- To keep a subtree of an outline out of search while publishing it, add `search-exclude:: true` to its top block.
			- [[lsg/Ref/Search Dialog]] describes the dialog, and [[lsg/Ref/Static Search Data]] describes what is searchable and the files `--search` writes.
		- ### 4. Show page properties
			- If you want each page's leading properties shown in a Properties disclosure, turn them on in `logseq/config.edn`:
			- ```clojure
			  {:publishing/page-properties? true
			   :publishing/page-properties-collapsed? true
			   :publishing/page-sequence? true
			   :block-hidden-properties #{:private-note :operational-field}}
			  ```
			- | Setting | Default | Effect |
			  | --- | --- | --- |
			  | `:publishing/page-properties?` | `false` | Shows leading page properties above each page's outline. |
			  | `:publishing/page-properties-collapsed?` | `true` | Starts the Properties disclosure closed. Set it to `false` to start it open. |
			  | `:publishing/page-sequence?` | `true` | Shows the Previous and Next region built from `prev::` and `next::`. Set it to `false` to hide the region. |
			  | `:block-hidden-properties` | none | A collection of keyword keys to leave out of Properties, property references and plugin data. |
			- Turning on page properties publishes every remaining scalar property value as public metadata. Add author-specific operational fields to `:block-hidden-properties`.
			- [[lsg/Ref/Published Pages]] describes the rows, the keys that never show, and the Previous and Next links.
		- ### 5. Set how reference sections start
			- If you want Linked References to start open or closed differently, set these top-level integers in `logseq/config.edn`:
			- | Setting | Default | Effect |
			  | --- | --- | --- |
			  | `:ref/linked-references-collapsed-threshold` | `100` | A Linked References section starts closed when its referring-block count reaches this number. `0` starts every section closed. |
			  | `:ref/default-open-blocks-level` | `2` | Copied parent blocks at this depth or deeper start closed. Authored collapsed parents also stay closed. |
			- A missing setting uses its default, and a setting `lsg` cannot use gets a diagnostic. Reading `logseq/config.edn` is bounded, never evaluates EDN and never follows symlinks on the path to the file.
		- ### 6. Fill the left sidebar
			- If you want pages listed in the left sidebar, add their titles or aliases to `:favorites` in `logseq/config.edn`. The sidebar lists them in that order, under Journals, Graph view and All pages.
			- Each favorite resolves through page titles and aliases. A favorite that names no published page is left out and reported as a diagnostic.
			- To turn off the `g g` shortcut to the graph view, set `:shortcuts {:go/graph-view false}`.
		- ### 7. Add your own stylesheet
			- If you want to restyle the index and every page, pass a local CSS file with `--stylesheet`:
			- ```sh
			  lsg publish --graph /path/to/garden --output /private/tmp/styled-site --filename-format triple-lowbar --include GitP/A/Session --stylesheet /path/to/theme.css
			  ```
			- `lsg publish` copies the file unchanged to `custom.css` beside the HTML files and loads it after the built-in styles, so standard CSS specificity and cascade rules apply. Without `--stylesheet`, the site keeps the built-in look. The stylesheet does not count toward the report's asset count.
			- The path must name a readable regular file with a `.css` extension, in any letter case, and have no symlink components. If a parent folder is a symlink, such as `/private/tmp` on macOS, use the physical path. An invalid stylesheet fails the build before the output directory is created.
			- Use CSS you trust: the browser can request resources that `url()` and `@import` name. Relative resource URLs resolve from the copied stylesheet in the output root. Only the stylesheet itself is copied, so arrange its other resources separately.
			- Rust callers use `site::publish_site_with_options` with `SiteOptions { stylesheet: Some(path) }`; `site::publish_site` keeps the built-in look. The `publish_site` SDK example accepts the same `--stylesheet PATH` flag.
		- ### 8. Change the colours
			- The default colours are Logseq's indigo theme. If you want other colours, set these custom properties in your stylesheet:
				- `--lsg-text`, `--lsg-background`, `--lsg-surface`, `--lsg-sidebar`, `--lsg-header`, `--lsg-link`, `--lsg-border`, `--lsg-quote-border`, `--lsg-warning`, `--lsg-focus` and `--lsg-muted`.
				- `--lsg-title` for page titles, `--lsg-guide` for outline guide lines and closed bullets, and `--lsg-rule` for the rule under h1 and h2 blocks.
			- Put light values on `:root` and dark values on `:root[data-lsg-theme="dark"]`. An equivalent `@media (prefers-color-scheme: dark)` rule for `:root:not([data-lsg-theme])` gives readers without JavaScript the dark palette:
			- ```css
			  :root { --lsg-link: #0b6bcb; }
			  :root[data-lsg-theme="dark"] { --lsg-link: #8ec5ff; }
			  @media (prefers-color-scheme: dark) {
			    :root:not([data-lsg-theme]) { --lsg-link: #8ec5ff; }
			  }
			  ```
			- A stylesheet with fixed colours needs matching dark values for its own components.
			- The header carries Logseq's `cp__header` class and the left sidebar's Graph view item carries `graph-view-nav`, so a stylesheet written to hide Logseq's entry points to the graph view hides these too.
	- ## Troubleshooting
		- If `--homepage` fails, the title or alias names no selected visible page, or names more than one. Check the letter case and the `--include` options.
		- If `--stylesheet` is refused, check the `.css` extension and replace any symlinked folder in the path with its physical path.
		- If a favorite or a setting has no effect, read the report: a favorite that names no published page, and a setting `lsg` cannot use, each get a diagnostic.
		- If the search button does not appear, serve the site over HTTP or HTTPS instead of opening it from disk.
