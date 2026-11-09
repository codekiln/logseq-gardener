tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Published Pages]], [[lsg/Ref/Host Integration API]], [[lsg/How To/Customize a Published Site]]

- # Site Navigation Reference
	- ## Overview
		- A published site surrounds each page with Logseq's header and left sidebar, and offers a right sidebar of cards, the All pages table, the journals feed, slides and the graph view. Each feature's script loads only on the pages that use it.
		- [[lsg/Ref/Published Pages]] describes the page itself. [[lsg/How To/Customize a Published Site]] describes the `logseq/config.edn` settings for favorites, shortcuts and colours. [[lsg/Ref/Host Integration API]] describes the browser functions a host page calls.
	- ## Header and left sidebar
		- Every page has Logseq's header: Toggle left sidebar on the left; Home on every page except `index.html`; and Graph, More and, from 640px wide, Toggle right sidebar on the right.
		- The left sidebar lists Journals when a journals feed is published, Flashcards unless `logseq/config.edn` sets `:feature/enable-flashcards? false`, Graph view, All pages, then the garden's favorites from `:favorites` in `logseq/config.edn`, in config order. Each favorite shows the title or alias as the author spelled it, and the current page is highlighted.
		- The sidebar starts closed. The header button or `t l` toggles it, and browser localStorage saves the state under `logseq-gardener-left-sidebar`.
		- From 640px wide, the open sidebar is 246px and pushes the page right. On narrower screens it opens over the page and closes when the reader taps the shade, follows a link or presses Escape.
		- ### Keyboard shortcuts
			- | Keys | Action |
			  | --- | --- |
			  | `t l` | Toggle the left sidebar |
			  | `t r` | Toggle the right sidebar |
			  | `t w` | Switch between the 810px and 1280px page widths, saved under `logseq-gardener-wide` |
			  | `t t` | Toggle between light and dark |
			  | `g h` | Open Home |
			  | `g a` | Open All pages |
			  | `g j` | Open the journals feed |
			  | `g g` | Open the graph view, unless `logseq/config.edn` sets `:shortcuts {:go/graph-view false}` |
			  | `g f`, `t c` | Open or close the flashcard review, unless flashcards are turned off |
			  | `alt+shift+c` | Add or remove the Contents card in the right sidebar |
			- Shortcuts are ignored while the reader types in a field.
		- ### Scripts
			- `lsg-shell.js` at the site root runs the header and the left sidebar, and every page loads it with `defer`. After the page has loaded, it also loads `lsg-outline.js` on every page and `lsg-journals.js` on journals feed pages.
			- Without JavaScript, the left sidebar stays closed, and its button and More stay hidden.
	- ## Right sidebar
		- From 640px wide, the Toggle right sidebar button at the right end of the header, or `t r`, opens a column of cards beside the page. The current page and its address stay the same, and the header and the page narrow to the remaining width. Below 640px there is no right sidebar, and Shift-click follows the link.
		- ### Adding cards
			- Shift-click a page link or a tag to add a card for that page, a bullet to add a card for that block, or a block reference to add a Block references card. In a zoomed block's breadcrumb, the page crumb adds a card for the page and each block crumb adds a card for its block.
			- In Linked References and query results, a bullet adds a card for its block on the source page, and a crumb adds a Block references card for its block, as in Logseq.
			- The top bar's Page graph button adds a card that draws the current page's neighbourhood, as Graph view below describes.
			- The top bar also adds Contents and Help cards. An empty sidebar shows the garden's `contents` page, and `alt+shift+c` adds or removes the Contents card.
		- ### Cards
			- A page card shows the page without its title or Unlinked References. Its Linked References load when opened.
			- Each card has a header that collapses it, a Card menu (close; collapse or expand this card or the others; Open as page) and a Close button. A middle click on the header also closes the card.
			- Drag the column's left edge, or focus it and press Left or Right, to resize the column.
			- Browser localStorage saves the cards, their order, their collapsed flags, the width and whether the sidebar is open under `logseq-gardener-right-sidebar`, and every page restores them.
			- On a first visit, `:default-home {:sidebar [..]}` in `logseq/config.edn` chooses the starting cards: `"Contents"` or page names. A name that matches no published page gets a diagnostic.
			- Cards fetch the published page files. Each card stores its page's address relative to the folder that serves `lsg-sidebar.js`, so a saved card opens the same page from every route.
		- ### Scripts
			- `lsg-sidebar.js` and `lsg-sidebar.css` run the right sidebar. They load the first time a reader uses the sidebar, or after the page loads when the sidebar was left open.
			- [[lsg/Ref/Host Integration API]] lists the link markup the sidebar recognizes, which a host that rewrites page addresses keeps.
	- ## All pages
		- The page index is Logseq's All pages table, titled `All pages`: a total, a `Search page names` box, a journals toggle, paging, and the columns `Page name`, `Backlinks`, `Created At` and `Updated At`.
		- It opens at `index.html`, or at `pages.html` when a homepage or a journals feed takes `index.html`.
		- ### Columns
			- Backlinks counts the blocks that refer to the page, including references to its namespace children and blocks on the page itself, as Logseq's column does. The page's Linked References count leaves out its own blocks, so the two can differ.
			- Created At and Updated At are the times of the first and last git commits that touched the page's file, shown in the reader's time zone, or in UTC without JavaScript. Generated namespace pages have no file and no dates.
			- A garden that is not a git repository gets no date columns, sorts by page name and gets one diagnostic. A shallow clone gets dates from the history it holds and also gets a diagnostic. The publisher reads the history while it renders.
			- Pages with an `icon::` property show the icon before the name.
			- Below 640px wide, only the icon and `Page name` columns show.
		- ### Sorting, search and paging
			- The default view hides journals and lists pages newest first, 40 rows at a time. `pages-2.html`, `pages-3.html` and so on hold the rest of that view, so every page is reachable without JavaScript.
			- Clicking a column title sorts by that column and flips the direction that all columns share.
			- Search runs on Enter with Logseq's fuzzy match on page names and shows at most 20 rows. Escape clears it.
			- The journals toggle adds journals.
			- Sorting, search and the journals toggle load `_pages/list.json`, which holds every entry, the first time a reader uses one of them. Paging through the default view never loads it.
		- Logseq's editing controls, the row checkboxes, `Delete`, the whiteboard toggle and the `⋯` menu, are not drawn, because they change nothing that lasts in a published site.
		- `lsg-all-pages.js` loads only on All pages screens. `window.logseqGardenerAllPages.init` is described on [[lsg/Ref/Host Integration API]].
	- ## Journals feed
		- With `--journals`, `index.html` becomes Logseq's journals feed and the page list moves to `pages.html`. With `--homepage`, the feed starts at `journals.html` instead.
		- The feed lists journals dated on or before the build date, newest first: three in its first file, then seven in each of `journals-2.html`, `journals-3.html` and so on.
		- The build date is the UTC date of `SOURCE_DATE_EPOCH` when it is set, and of the system clock otherwise. Journals dated later keep their pages and stay out of the feed.
		- As the reader nears the bottom, `lsg-journals.js` appends the next file's journals. Without JavaScript, Earlier opens it.
		- Each journal's Linked References show their page count and load from the journal's page when opened.
		- A garden with `:feature/enable-journals? false` gets journal pages and no feed.
	- ## View as slides
		- On a page or journal page, from 640px wide, More starts with View as slides. It adds a Slides card for the page to the top of the right sidebar.
		- Each top-level block is a column, and the block and then each of its descendants is a slide below it. Blocks that show nothing make no slide.
		- A block's `background-image::`, `background-color::`, `transition::` and `data-*` properties apply to its slide, as in Logseq.
		- The deck takes keyboard focus:
			- | Keys | Action |
			  | --- | --- |
			  | Arrow keys, or `h j k l` | Move between slides |
			  | `n`, Space | Step through the whole deck |
			  | `g` | Jump to a slide number or `column/slide` |
			  | `o`, Escape | Show every slide |
			  | `b` | Black out the deck |
			  | `f` | Toggle fullscreen |
		- `lsg-slides.js` loads with the first Slides card and holds the deck's styles. It builds the deck from the page's own outline and keeps only the current slide and its neighbours in the document, so images on distant slides load when the reader gets near them.
	- ## Graph view
		- `graph.html` draws every page as a node and every page reference, tag and namespace parent as an edge, as Logseq's graph view does. Pages that exist only because a published page names them are nodes that open nothing. Pages that are not published never appear.
		- The header's Graph link and the sidebar's Graph view item open it. They carry Logseq's classes, `cp__header` on the header and `graph-view-nav` on the item, so a stylesheet written to hide Logseq's entry points hides these too.
		- ### Controls
			- Drag to pan, scroll or pinch to zoom, click a node to open its page, and Shift-click or drag a node to select it.
			- The panel at the top right has Logseq's sections:
				- Nodes: the Journals, Orphan pages, Built-in pages and Excluded pages switches, and N hops from the selected nodes.
				- Search: title filters and a list of matching nodes, which is also the keyboard path to a node.
				- Forces: the layout sliders.
				- Export: as PNG.
			- The panel's starting values come from `:graph/settings` and `:graph/forcesettings` in `logseq/config.edn`. A reader's changes last for the browser tab's session.
		- ### Loading
			- A download that takes longer than half a second shows a spinner, `Loading the graph…` and a Cancel button over the graph area. Cancel stops the download and offers `Load the graph`, and a failed download offers Retry.
			- The panel appears with the first drawn graph.
			- The Page graph card shows the same states for its own files.
		- ### Files
			- The publisher lays the graph out at build time with Logseq's forces and writes it to `_graph/default.bin`, and the journals graph to `_graph/journals.bin`, both stored gzip-compressed.
			- `graph.html` loads them through `lsg-graph.js`, and so does a Page graph card whose page has no neighbourhood file. On `graph.html`, `lsg-graph-worker.js` runs a force simulation only when a reader changes a force or drags a node. A Page graph card uses it to lay out each neighbourhood before its first frame.
			- The Page graph card reads `_graph/p/<route hash>.json`, the page's neighbourhood. A page whose neighbourhood file would exceed 100 KB gzipped has none, and its card draws the neighbourhood from the global file instead.
			- Through the SDK, `SiteOptions::layout_cache` keeps layouts between builds in a directory of your choice.
	- ## Appearance
		- A published site follows the reader's system appearance by default.
		- Toggle theme in the More menu, or `t t`, switches between light and dark and saves the choice under `logseq-gardener-theme` in browser localStorage for the origin. While a choice is saved, More also offers Use system theme, which removes it.
		- If storage is unavailable, the choice lasts for the current document and its internal navigation.
		- A script in each page's head applies the initial appearance and updates it when the system preference changes. Without JavaScript, the shared CSS still follows the system preference.
		- The default colours are Logseq's indigo theme. [[lsg/How To/Customize a Published Site]] shows how to change them, and [[lsg/Ref/Host Integration API]] describes `window.logseqGardenerInitTheme` for a host that adds its own Appearance selector.
