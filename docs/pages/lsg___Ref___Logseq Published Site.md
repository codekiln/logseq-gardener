tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]]

- # Logseq reference boards
	- The pages under this one are the reference for how Logseq's published static site looks and behaves. The parity issue [codekiln/logseq-gardener#106](https://github.com/codekiln/logseq-gardener/issues/106) and its child issues compare the `lsg` output against them.
	- The target is Logseq 0.10 "publish-spa", as built for `codekiln/logseq-encode-garden`.
	- ## How to use it
		- Read the pages in the Boards table. Each one is a text copy of one HTML board, with every measurement, class name, string and example from the board.
		- The HTML boards are in [`docs/logseq-reference/html/`](https://github.com/codekiln/logseq-gardener/tree/main/docs/logseq-reference/html). They come from a Claude Design canvas and need that canvas's runtime (`support.js`, not in that folder) to render or run. Without it the files are plain markup and script. Read the source or these pages instead.
		- [`html/canvas.json`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/canvas.json) is the canvas layout (board positions and titles). It carries no behaviour.
		- For behaviour (search, sidebars, flashcards, queries, slides), start with [[lsg/Ref/Logseq Published Site/Clickable Prototype]]. It documents the script of the clickable prototype and includes the exact text of the 24 page sources the prototype renders.
		- For how a garden file turns into DOM, start with the six spec boards. Each section gives the garden source, what Logseq renders, and the DOM and rules.
			- [[lsg/Ref/Logseq Published Site/Frontmatter and Properties]]
			- [[lsg/Ref/Logseq Published Site/Transclusion]]
			- [[lsg/Ref/Logseq Published Site/Backlinks]]
			- [[lsg/Ref/Logseq Published Site/Namespace Hierarchy]]
			- [[lsg/Ref/Logseq Published Site/Outline and Blocks]]
			- [[lsg/Ref/Logseq Published Site/Inline Syntax]]
	- ## Boards
		- | Page | HTML board | Covers |
		  | --- | --- | --- |
		  | [[lsg/Ref/Logseq Published Site/Clickable Prototype]] | [`html/Prototype.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Prototype.dc.html) | Clickable prototype: routes, header, sidebars, More menu, export, theme tokens, search, flashcards, filter dialog, slides, Markdown renderer rules, queries, known gaps, page sources |
		  | [[lsg/Ref/Logseq Published Site/Page View]] | [`html/Main.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Main.dc.html) | Page view: header, left sidebar, title, properties, blocks, tagged pages, references, hierarchy |
		  | [[lsg/Ref/Logseq Published Site/Journals Home]] | [`html/Journals.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Journals.dc.html) | Home route: stacked journal day pages |
		  | [[lsg/Ref/Logseq Published Site/All Pages]] | [`html/AllPages.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/AllPages.dc.html) | All pages table and paging |
		  | [[lsg/Ref/Logseq Published Site/Search and More Menu]] | [`html/Overlays.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Overlays.dc.html) | Search dialog and the More menu in publishing mode |
		  | [[lsg/Ref/Logseq Published Site/Flashcards Review]] | [`html/Flashcards.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Flashcards.dc.html) | Flashcards review modal |
		  | [[lsg/Ref/Logseq Published Site/Graph View]] | [`html/Graph.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Graph.dc.html) | Graph view and its settings panel |
		  | [[lsg/Ref/Logseq Published Site/Frontmatter and Properties]] | [`html/Frontmatter.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Frontmatter.dc.html) | Spec: page and block properties, hidden keys, keys that change the page |
		  | [[lsg/Ref/Logseq Published Site/Transclusion]] | [`html/Transclusion.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Transclusion.dc.html) | Spec: block refs, block and page embeds, broken targets |
		  | [[lsg/Ref/Logseq Published Site/Backlinks]] | [`html/Backlinks.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Backlinks.dc.html) | Spec: Linked References, filter dialog, Unlinked References |
		  | [[lsg/Ref/Logseq Published Site/Namespace Hierarchy]] | [`html/Hierarchy.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Hierarchy.dc.html) | Spec: namespace pages and the Hierarchy section |
		  | [[lsg/Ref/Logseq Published Site/Outline and Blocks]] | [`html/Outline.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Outline.dc.html) | Spec: bullets, collapse, headings, zoom, tasks |
		  | [[lsg/Ref/Logseq Published Site/Inline Syntax]] | [`html/Inline.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Inline.dc.html) | Spec: page refs, tags, links, code, emphasis, images, macros |
		  | [[lsg/Ref/Logseq Published Site/Reference Sheet]] | [`html/Reference.dc.html`](https://github.com/codekiln/logseq-gardener/blob/main/docs/logseq-reference/html/Reference.dc.html) | Build pipeline, kept and dropped features, tokens, DOM anatomy, feature map |
	- ## Where the values come from
		- The boards say which values they measured and which they took from Logseq's source. The pages keep those statements.
		- Measured on the live published build:
			- Design tokens on [[lsg/Ref/Logseq Published Site/Reference Sheet]]: colours, type sizes and weights, layout sizes. The board says "measured from the live build, light theme".
			- The section order on a page (blocks, Pages tagged with, Linked References, Hierarchy, Unlinked References), "measured on the published `tmux` page" ([[lsg/Ref/Logseq Published Site/Namespace Hierarchy]]).
			- Page title size 36px ([[lsg/Ref/Logseq Published Site/Outline and Blocks]]), and heading sizes h1 to h4 (the `Prototype/Fixture/Headings` page on [[lsg/Ref/Logseq Published Site/Clickable Prototype]]).
			- Counts: 16,693 pages, 430 flashcards, 795 Linked References on `tmux`.
		- Taken from Logseq's source (file names are on each board):
			- Spec boards: `block.cljs`, `block.css`, `reference.cljs`, `reference.css`, `hierarchy.cljs`, `common.css`, `dicts/en.edn`.
			- h5 and h6 sizes (`block.css`, per the headings fixture).
			- Filter dialog width (`.ls-filters`, max-width 704px) and the strings from `dicts/en.edn`.
		- Illustrative or placeholder values, marked as such on their boards:
			- Filter chip counts on the Backlinks board.
			- Node positions on the Graph board (the real layout is a force simulation).
			- The `[overdue]` count on the Flashcards board.
			- Image and video placeholders on [[lsg/Ref/Logseq Published Site/Inline Syntax]] and [[lsg/Ref/Logseq Published Site/Clickable Prototype]].
	- ## Conflicts between boards
		- [[lsg/Ref/Logseq Published Site/Clickable Prototype]] ends its "Known gaps" section with a list of differences between the prototype and the other boards (favorites count, dialog widths, search group names). Read it before treating any one board as the single answer.
