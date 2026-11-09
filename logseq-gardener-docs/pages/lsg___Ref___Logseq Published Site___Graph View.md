tags:: [[Diataxis/Reference]]

- # Graph view
	- This board shows the Graph screen: a force-directed graph of pages with a settings panel and a behaviour note.
	- Overall: board 1440x900, font 'Source Sans 3', text `#11181c`, background `#ffffff`, column layout.
	- ## Header
		- Height 48px, no shrink, flex, space-between, padding `0 12px`, border-bottom `1px solid #e9ecef`.
		- Left (colour `#687076`): two 20px icons with margin 8px: three-lines menu (left sidebar) and magnifier (search).
		- Right (14px, weight 500, colour `#687076`): "Graph" (padding `8px 10px`, colour `#11181c`, marks the current screen) and a three-dots icon (More). There is no right-sidebar toggle on this board.
	- ## Canvas
		- Background `#f8f9fa`, overflow hidden. SVG 1440x852, `role="img"`, aria-label "Force-directed graph of garden pages; tmux and nvim are large hubs with many linked pages around them".
		- Edges: stroke `#ced4da`, 1px.
		- Nodes: fill `#9ba1a6`; the hub "tmux" is `#3e63dd`.
		- Labels: 13px, 'Source Sans 3', fill `#3a3f44`, centred under the node; "tmux" is weight 600, fill `#11181c`.
		- Labelled nodes (cx, cy, r):
			- | Label | cx | cy | r |
			  | --- | --- | --- | --- |
			  | tmux | 560 | 420 | 18 |
			  | nvim | 700 | 330 | 13 |
			  | vim | 860 | 280 | 11 |
			  | LazyVim | 820 | 450 | 9 |
			  | tmux/Config | 420 | 300 | 8 |
			  | tmux/Keyshort | 470 | 560 | 8 |
			  | oh-my-tmux | 640 | 600 | 7 |
			  | Catppuccin | 380 | 450 | 6 |
			  | Keyshort | 940 | 380 | 8 |
			  | vim/Keyshort | 980 | 220 | 6 |
			  | mise | 640 | 220 | 6 |
			  | tmux/Docs | 330 | 220 | 5 |
			  | card | 1040 | 470 | 5 |
		- Unlabelled nodes have radius 3 to 6. Edges are straight lines given by this path (each `M x y L x y` is one edge, coordinates match node centres); positions are illustrative:
			- ```
			  M560 420L420 300  M560 420L470 560  M560 420L700 330  M560 420L640 600  M560 420L380 450
			  M700 330L860 280  M700 330L820 450  M860 280L980 220  M860 280L940 380  M820 450L940 380
			  M820 450L780 600  M470 560L360 640  M470 560L520 690  M420 300L330 220  M420 300L480 190
			  M380 450L280 420  M640 600L720 700  M940 380L1040 470  M980 220L1060 160  M700 330L640 220
			  M640 220L560 160  M860 280L800 180
			  ```
		- Four small extra nodes (radius 3) at (200,120), (1120,620), (240,740), (1000,740) have no edges.
	- ## Graph settings panel
		- `aside` (aria-label "Graph settings"): absolute top 16px, right 16px, width 280px, background `#ffffff`, border `1px solid #e9ecef`, radius 10px, box-shadow `0 8px 24px rgba(0,0,0,0.08)`, padding `12px 16px 16px`, column, gap 4px. Dividers: `1px solid #e9ecef`, margin `8px 0 4px`.
		- Row (`.row`): flex, space-between, gap 12px, 14px, min-height 32px. Section title (`.sec`): 13px, weight 600, colour `#11181c`, gap 6px, margin-top 6px. Switch (`.sw`): 32x18, radius 9px, off `#dee2e6`, on `#3e63dd`; knob 14px white at left 2px (off) or 16px (on). Slider: 4px high, track `#dee2e6`, radius 2px, thumb 14px `#3e63dd`.
		- Sections and controls, in order:
			- 1. Nodes
				- "Orphan pages": on.
				- "Built-in pages": off.
				- "Excluded pages": off.
				- "Show Journals": off.
			- 2. Search
				- "Click to search" (colour `#687076`) with the link "Clear All" (13px, `#106ba3`).
			- 3. Forces
				- "Link Distance" (13px, `#687076`), slider thumb at 40%.
				- "Charge Strength", slider thumb at 60%.
				- "Charge Range", slider thumb at 50%.
				- "Reset Forces" (13px, `#106ba3`).
			- 4. Export (section title only; no controls drawn).
	- ## Behaviour note (callout on the board)
		- Callout: absolute left 24px, bottom 24px, width 460px, background `#ffffff`, border `1px solid #e9ecef`, radius 10px, padding `14px 18px`, 14px, line-height 1.55, colour `#3a3f44`. Its text:
			- Header link "Graph" goes to `#/graph`.
			- Every page is a node sized by link count; edges are page references.
			- Click a node to open the page; journals are hidden by default.
			- A page's own neighbourhood appears as "Page graph" in the right sidebar.
			- Node layout is a force simulation, so positions on the board are illustrative.
