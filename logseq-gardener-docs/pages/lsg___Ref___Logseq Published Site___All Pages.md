tags:: [[Diataxis/Reference]]

- # All pages
	- This board shows the published "All pages" screen: a paged table of every page with backlink count and timestamps.
	- Overall: board min-height 900px, font 'Source Sans 3', text `#11181c`, background `#ffffff`, 16px. Links `#106ba3` (hover `#0b4f7a`).
	- ## Header
		- Height 48px, flex, space-between, padding `0 12px`, background `rgba(255,255,255,0.9)`, border-bottom `1px solid #e9ecef`.
		- Left: icon buttons "Toggle left sidebar" and "Search".
		- Right: text link "Graph" (14px, weight 500, `#687076`, padding `8px 10px`, radius 6px), icon buttons "More" and "Toggle right sidebar".
		- No left sidebar is drawn on this board.
	- ## Main column
		- `main` padding `32px 24px 64px`; wrapper max-width 1100px, centred.
		- `h1` "All pages": 36px, weight 300, colour `#000000`, line-height 1.3, margin `0 0 6px`.
		- Toolbar row (flex, wrap, space-between, gap 12px, margin-bottom 16px):
			- Left: "Total 16693 pages" (14px, `#687076`).
			- Right (gap 12px, 14px, `#687076`): "1/418" and a button "Next" (14px, weight 500, colour `#11181c`, background `#f1f3f5`, border `1px solid #e9ecef`, radius 6px, padding `6px 14px`, min-height 36px).
		- Table wrapper: `overflow-x: auto`, border `1px solid #e9ecef`, radius 8px. Table width 100%, collapsed borders.
		- ### Table
			- Column headers (`th`, `scope="col"`): "Page name", "Backlinks", "Created At", "Updated At". Style: weight 600, 14px, colour `#687076`, left aligned, padding `10px 12px`, border-bottom `1px solid #e9ecef`, no wrap.
			- Cells (`td`): padding `9px 12px`, border-bottom `1px solid #f1f3f5`, 15px, no wrap. Page name is a link. The two date cells are colour `#687076`. Date format: `YYYY-MM-DD HH:mm`.
			- Rows shown (page name, backlinks, created, updated):
				- | Page name | Backlinks | Created At | Updated At |
				  | --- | --- | --- | --- |
				  | card | 432 | 2026-10-24 06:04 | 2026-10-24 06:04 |
				  | Favorites | 0 | 2026-10-24 06:04 | 2026-10-24 06:04 |
				  | contents | 0 | 2026-10-24 06:04 | 2026-10-24 06:04 |
				  | yt-dlp/PyPI | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yt-dlp/GitHub | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | CLI/Tool/yt-dlp | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | ya | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Plugin/spotter | 2 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Sort/Time/Birth | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Sort | 2 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Sort/Time | 2 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Sort/Time/Modification | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Leader | 1 | 2026-10-23 15:49 | 2026-10-23 15:49 |
				  | yazi/Keyshort/Help | 2 | 2026-10-23 15:49 | 2026-10-23 15:49 |
			- The board does not say how rows are sorted; the shown rows are a sample, not a full page. Page size follows from the counts: 16693 pages over 418 pages gives about 40 rows per page (derived, not stated on the board).
