tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Publish Selected Namespaces]], [[lsg/Ref/Command Line]], [[lsg/Ref/Site Manifest]]

- # Publish Options Reference
	- ## Overview
		- `lsg publish` reads a Logseq garden and writes its selected namespaces as a static website into a new directory, using the Rust SDK. This page lists the command's options, its output directory rules, the report it prints, its exit codes and the folder layout that `--readable-urls` writes. [[lsg/Ref/Command Line]] describes the rules that every `lsg` command shares.
		- ```sh
		  lsg publish --graph PATH --output PATH --filename-format legacy|triple-lowbar \
		    [--include NAMESPACE]... [--exclude NAMESPACE]... [OPTIONS]
		  ```
		- Run `lsg publish help` to read the command's documentation offline.
	- ## Options
		- | Option | Meaning |
		  | --- | --- |
		  | `--graph PATH` | Required. The root folder of the garden. |
		  | `--output PATH` | Required. A new directory to publish into, as Output directory below describes. |
		  | `--filename-format FORMAT` | Required. `triple-lowbar` when the garden's `logseq/config.edn` sets `:file/name-format :triple-lowbar`, and `legacy` for Logseq's default file name format. [[lsg/Ref/Page Titles]] describes how each format gives a page its title. |
		  | `--include NAMESPACE` | Selects a namespace and its descendants. Repeat it to select more namespaces. With no `--include`, no page is selected, and the index lists only the journals that `--journals` publishes. |
		  | `--exclude NAMESPACE` | Removes a namespace and its descendants from the selection. Repeat it to remove more namespaces. An exclusion always wins over an inclusion. |
		  | `--homepage TITLE` | Renders the selected page or alias `TITLE` at `index.html`. |
		  | `--journals` | Publishes every file under `journals/` as a page, with Logseq's journals feed. |
		  | `--search` | Writes the search files and shows the search button on every page. |
		  | `--stylesheet PATH` | Copies a CSS file to `custom.css` and loads it after the built-in styles. |
		  | `--plugin PATH` | Adds the site plugin whose manifest is `PATH`. Repeat it to add more plugins. |
		  | `--readable-urls` | Writes each page to a folder named by its title, as Readable URLs below describes. |
		  | `--timings` | Also reports how long each publication phase took. |
		  | `--format json` | Prints the report as one JSON document. |
		- [[lsg/Ref/Selection and Visibility]] describes how `--include` and `--exclude` match page titles and which selected pages are published.
		- [[lsg/How To/Customize a Published Site]] describes `--homepage`, `--journals`, `--search` and `--stylesheet`, and [[lsg/Ref/Site Navigation]] describes the journals feed.
		- [[lsg/How To/Add a Site Plugin]] describes `--plugin`, and [[lsg/Ref/Plugin Host API]] lists the manifest fields.
		- [[lsg/Ref/Static Search Data]] describes the files that `--search` writes, and [[lsg/Ref/Search Dialog]] describes the dialog.
	- ## Output directory
		- `--output` names a directory that does not exist yet, whose parent exists, outside the source garden. Use a fresh directory for each run. `lsg publish` refuses a directory that already exists, so files of pages excluded since an earlier run never stay in the new site.
		- `lsg publish` reads the garden and leaves every source file unchanged.
		- Invalid options, an invalid stylesheet, a rejected plugin manifest, a failing build plugin, a missing or ambiguous homepage, duplicate selected titles and the title problems that `--readable-urls` reports stop the build before it creates the output directory.
		- A file system write failure during the build may leave partial output in the newly created directory. Fix the error, then publish into a fresh directory.
	- ## Report
		- Without `--format json`, standard output holds one line with the counts and the path of the index:
		- ```text
		  Published 12 pages, 0 journals and 3 assets; withheld 1 pages; skipped 0 journals; 2 diagnostics. Open /tmp/site/index.html
		  ```
		- Standard error holds the diagnostics, one per line: the source file, a colon and the message. A diagnostic names an unsupported construct, an unresolved or ambiguous reference, a withheld page or another problem that leaves the site complete. Diagnostics stay out of the published site. To keep them, redirect standard error, such as `2> /private/tmp/garden-site-report.txt`.
		- With `--format json`, standard output holds one JSON document:
		- | Field | Value |
		  | --- | --- |
		  | `format_version` | `1` |
		  | `command_path` | `["publish"]` |
		  | `pages` | Selected page files, not counting journals, generated namespace pages and the index. |
		  | `journals` | Published journal files. |
		  | `assets` | Distinct asset files copied from the garden. A `--stylesheet` file is not counted. |
		  | `withheld_pages` | Selected pages that the visibility rules withhold. A diagnostic names each one. |
		  | `skipped_journals` | Journals left out: every journal without `--journals`, and journal files whose names are not dates. |
		  | `diagnostic_count` | The number of diagnostics on standard error. |
		  | `output_directory` | The `--output` path. |
		  | `index_path` | The path of `index.html`. |
		  | `timings` | With `--timings`, a list of `{"phase": NAME, "ms": MILLISECONDS}`. |
		  | `total_ms` | With `--timings`, the total time in milliseconds. |
		- Paths accept native operating system arguments. `output_directory` and `index_path` are display strings of those paths.
		- `--timings` reports the phases `read` (finding and reading source files and checking options), `parse`, `index` (selection, names, references, queries and search records), `plugins` (waiting for build plugins), `render` (page and list documents), `history` (waiting for the git history read), `search` (search files), `graph` (waiting for graph layouts and neighbourhoods) and `write`, each in milliseconds, rounded to one decimal place. A waiting phase measures only the time the publisher waited for work that ran alongside other phases. Without `--format json`, the timings are a line on standard error: `Timings in milliseconds: read 12.3, parse 45.6, ...; total 210.4`. `--timings` changes no published file. [[lsg/How To/Measure Performance]] shows how to use it, and [[lsg/Ref/Publication Speed]] records measured builds.
	- ## Exit codes
		- | Code | Meaning |
		  | --- | --- |
		  | 0 | Publication completed, with or without diagnostics. |
		  | 1 | Publication failed. Standard output is empty and standard error names the problem. |
		  | 2 | Invalid arguments or namespace selectors. Standard output is empty. |
		- A namespace selector is invalid when it is empty, has an empty segment, has whitespace around a segment or contains a control character.
	- ## Readable URLs
		- By default, `lsg publish` writes each page to `p-<digest of its title>.html` beside `index.html`. With `--readable-urls`, it writes each page, journal and namespace page to a folder named by its title, and links to that folder:
		- ```sh
		  lsg publish --graph /path/to/garden --output /tmp/new-site --filename-format triple-lowbar --include GitP --readable-urls
		  ```
		- | Page title | File | Link and `lsg-site.json` `url` |
		  | --- | --- | --- |
		  | `GitP/A/Session/26/10/20-Tue` | `GitP/A/Session/26/10/20-Tue/index.html` | `GitP/A/Session/26/10/20-Tue/` |
		  | `Notes/Q&A: 2026` | `Notes/Q&A: 2026/index.html` | `Notes/Q%26A%3A%202026/` |
		  | `2026-10-23 Fri`, a journal | `2026-10-23 Fri/index.html` | `2026-10-23%20Fri/` |
		- Each segment of a link keeps ASCII letters and digits, `-`, `.`, `_` and `~`, and percent-encodes every other character as UTF-8.
		- The home page, All pages, the journals feed and the graph view stay at the output root under the names they have without the option. A `--homepage` page is written both to `index.html` and to its own folder.
		- Rust callers set `SiteOptions::page_urls` to `PageUrls::Readable`. `page_urls::readable_url` gives the link of any title.
		- Every page in a folder carries a `<base>` element that points at the site root, so its links, assets, scripts and search results resolve as they do from `index.html`. The site works from a static server under any path, such as `https://example.org/logseq-gardener/`, and from disk.
		- A browser shows a folder listing for a `file://` link that ends in `/`, so pages add `index.html` to such a link as the reader follows it. With JavaScript turned off, open `index.html` from the listing.
		- ### Titles that stop the build
			- The build stops before it creates the output directory, and names the pages, when a selected title cannot be a path or two pages would share one:
				- A segment is empty, `.` or `..`, starts or ends with whitespace, or is longer than 255 bytes in UTF-8.
				- The title contains a backslash or a control character.
				- Two titles, or two of their folders, differ only in letter case or Unicode normalization, as `Tools/Vim` and `tools/vim` do. macOS and Windows file systems treat those as one path.
				- A title's last segment is `index.html` and its parent is also a page.
				- A title's first segment is a file or folder that the build writes at the output root, such as `assets`, `graph.html` or `lsg-site.json`.
			- Rename the page, or give it a `title::` property, then publish again.
			- Windows also refuses `<`, `>`, `:`, `"`, `|`, `?` and `*` in file names. A readable build on Windows stops at the first title with one of them, names the path it could not write, and can leave partial output.
