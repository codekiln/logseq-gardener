tags:: [[Diataxis/Reference]]

- # Spec: frontmatter and properties
	- This board specifies how Logseq renders page properties and block properties: as a grey key-value box, with some keys hidden and some keys changing the page itself.
	- Title on the board: "Properties render as a grey key–value box".
	- Gardener specs: `markdown-page-titles`, `selected-page-aliases`, `local-static-publishing`.
	- Logseq source: `block.cljs › property-cp`, `block.css .page-properties`.
	- ## 1 · Page properties (the first block)
		- Garden source:
			- ```markdown
			  tags:: [[TUI]]
			  alias:: [[Spotify TUI Options]]

			  - # Spotify [[TUI]] Projects
			  ```
		- Logseq renders: the page title "Spotify/Tool/TUI" (36px, weight 300, colour `#000000`, line-height 1.3, 8px below), then a grey box with two rows: `tags : TUI` and `alias : Spotify TUI Options`. The key is a link, the value is a link without brackets.
		- ```html
		  <div class="page-properties">
		    <div><a class="page-property-key">tags</a>:<span class="page-property-value"><a>TUI</a></span></div>
		    <div><a class="page-property-key">alias</a>:<span class="page-property-value"><a>Spotify TUI Options</a></span></div>
		  </div>
		  ```
		- DOM and rules:
			- Leading `key:: value` lines before the first `-` form the pre-block, drawn as `div.page-properties`.
			- Box style: background `#f8f9fa`, padding `4px 8px`, margin `4px 0`, radius 4.
			- Each row is `key`, `:`, `.page-property-value`.
			- The key is itself a link to the property's page (`.page-property-key`, weight 500, secondary text colour).
			- `tags` and `alias` values are split on commas and each becomes a page link without brackets.
	- ## 2 · Other values keep their inline markup
		- Garden source:
			- ```markdown
			  created-by:: [[Person/Nicholas Marriott]]
			  logseq-entity:: [[Logseq/Entity/Software/Project]]
			  ```
		- Logseq renders: a grey box with two rows. Each value shows grey brackets (`[[` and `]]`, colour `#6b7280`) around a link.
		- ```html
		  <div class="page-properties">
		    <div><a class="page-property-key">created-by</a>:<span class="br">[[</span><a>Person/Nicholas Marriott</a><span class="br">]]</span></div>
		    <div><a class="page-property-key">logseq-entity</a>:<span class="br">[[</span><a>Logseq/Entity/Software/Project</a><span class="br">]]</span></div>
		  </div>
		  ```
		- DOM and rules:
			- Values that are not comma-split render as inline text, so `[[…]]` keeps its grey brackets (`:ui/show-brackets?` defaults to true).
			- Quoted values lose their quotes.
			- Integers and dates print as-is.
	- ## 3 · Block properties and hidden keys
		- Garden source:
			- ```markdown
			  - **Zoom pane (toggle)** #card
			    card-last-interval:: 10.24
			    card-repeats:: 3
			    card-next-schedule:: 2026-07-04T12:38:29.316Z
			    id:: 6a3f6744-5403-429d-ad4a-4391e01224ea
			    source:: [[tmux/man]]
			  ```
		- Logseq renders: a bullet with bold "Zoom pane (toggle)" and a `#card` tag (font-size 0.9em, opacity 0.7, padding `0 2px`). Under it a grey box with one row only: `source : [[tmux/man]]` (brackets shown). The `card-*` keys and `id` are not shown.
		- DOM and rules:
			- Indented `key::` lines under a block render inside it as `div.block-properties` (same box as page properties).
			- Never shown: `id`, `collapsed`, `heading`, `background-color`, `created-at`, `updated-at`, `query-*`, `hl-*`, `ls-type`, task markers, and the `card-*` scheduling keys.
			- A block with only hidden keys gets no box at all.
	- ## 4 · Keys that change the page instead of showing
		- Garden source:
			- ```markdown
			  title:: tmux
			  public:: false
			  alias:: [[Spotify TUI Options]]
			  ```
		- Logseq renders: no properties box for these keys. The board lists three effects:
			- `title::` replaces the filename-derived name as the page title and the name every link resolves to.
			- Setting `public:: false` withholds the whole page from the published site (with `:publishing/all-pages-public? true`).
			- Visiting an `alias` opens the original page; the alias also collects its own backlinks into that page's Linked References.
		- DOM and rules:
			- Page name precedence: `title::`, then the filename with `___` read as `/` (URL-decoded).
			- Names compare case-insensitively.
			- Routes use the lower-cased name: `#/page/spotify%2Ftool%2Ftui`.
