tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Static Search Data]], [[lsg/Ref/Host Integration API]], [[lsg/Ref/Logseq Compatibility/Search]]

- # Search Dialog Reference
	- ## Overview
		- Every page of a site published with `--search` shares one search dialog, modelled on Logseq's. The header's search button and Cmd/Ctrl+K open it. It reads the files that [[lsg/Ref/Static Search Data]] describes.
		- Serve a search-enabled site over HTTP or HTTPS, also for a local preview. Pages opened from `file:` keep their ordinary navigation and hide the search button.
	- ## Loading
		- Search-enabled pages load a content-fingerprinted `lsg-search-start-*.mjs` startup module. It imports the full `lsg-search-ui-*.mjs` controller when a reader points at or focuses the search button, or on the first open.
		- The search button stays hidden until JavaScript finds a search manifest link and native dialog support. The controller hides it again when the link is invalid.
	- ## Layout
		- The dialog has no visible title, `aria-label="Search"`, a visually hidden input label and the placeholder `What are you looking for?`.
		- Results appear in the groups Filters, Pages, Commands, Current page and Blocks. Each group shows a count, `99+` from 100, and five rows until `Show more`, `mod+Down`, `mod+Up` or a click on the group name expands or collapses it.
		- A block row shows a breadcrumb of its page and up to three parents. Matched words are wrapped in `span.ui__list-item-highlighted-span`.
		- The page being viewed is the page whose title is the text of `main > h1`. Its matching blocks appear under Current page.
		- Typing `/` lists the filters `Search only pages`, `Search only current page`, `Search only blocks` and `Search only commands`.
		- Commands run the actions of `lsg-shell.js`: theme, left sidebar, wide mode, home, and All pages and journals when the page links to them. A host page without that script shows no commands.
		- Result titles, breadcrumbs and text are written as text nodes. The controller uses the shared appearance variables on desktop and mobile, in light, dark and system themes.
	- ## Keyboard and focus
		- The input keeps focus. Up and Down, and Ctrl+P and Ctrl+N, move the highlight across groups and wrap around.
		- Enter opens the highlighted row, runs a command or applies a filter.
		- Escape clears the input, then the filter, then closes the dialog. A click outside the dialog also closes it.
		- Closing returns focus to the control that opened the dialog.
		- Native modal behaviour keeps Tab focus inside the dialog.
		- Modified clicks and new-tab clicks on result links keep the browser's behaviour.
	- ## States and failures
		- The dialog shows loading, an empty query, no results and failed publication loading as separate states. A failure offers Retry search and Reload page buttons. Loading never reloads the page by itself.
		- Reopening a loaded publication reuses its prepared index. Closing aborts pending requests, and a changed revision discards cached results.
		- The loader checks that file paths are same-origin HTTP or HTTPS, the response status and redirects, the expected revision and format version, and the engine's digest and shape validation.
		- Each query fetches its missing shards in parallel, then the chunks of the rows it draws. Concurrent queries share requests.
	- ## State attributes
		- | Attribute | Meaning |
		  | --- | --- |
		  | `data-lsg-search-state` | The dialog's state: `loading`, `ready`, `error` or `closed`. |
		  | `data-lsg-search-complete` | Advances after each completed result render. |
		- The parity harness measures each keystroke by waiting for `data-lsg-search-complete` to advance, so queries with no results are measured too.
	- ## With a persistent host
		- A host that keeps the page across navigations calls `window.logseqGardenerInitSearch` after each page swap, as [[lsg/Ref/Host Integration API]] describes. The dialog stays one body-level element outside `<main>`.
		- An ordinary result activation closes the dialog before its click bubbles to the host's delegated navigator. The controller does not prevent the link's navigation, and the host keeps its own player, theme and filter initialization and fragment reveal steps.
		- The controller uses no audio API, and changes the theme only when a reader runs that command.
	- ## Checks
		- `mise run search:browser-test` publishes a static site and runs it in a pinned Chromium. It checks keyboard and focus, groups, Show more, highlighting, aliases, filters, commands, the dialog's computed styles, safe snippets, mobile and themes, lazy requests and caching, including one shard for `tm` and then `tmux`, modified links, repeated initialization in a single-page host, metadata invalidation, explicit retry and the fallback without JavaScript.
		- Unit fixtures for the loader also cover digest mismatch, remote and version failures, one fetch per shard, and stale fetches and module loads that finish after a close or a revision change.
		- These are functional tests. [[lsg/Ref/Logseq Compatibility/Search Performance]] records measured performance from the parity harness.
