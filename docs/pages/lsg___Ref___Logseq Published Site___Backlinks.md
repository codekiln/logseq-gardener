tags:: [[Diataxis/Reference]]

- # Spec: backlinks
	- This board specifies Linked References, the filter dialog and Unlinked References on a Logseq page: every block that points to the page, grouped by referencing page.
	- Board title: "Linked References: every block that points here, grouped by page".
	- Gardener spec: `published-linked-references`.
	- Logseq source: `reference.cljs › references, filter-dialog`; `reference.css`; strings in `dicts/en.edn`.
	- All examples are shown on the page "tmux".
	- ## 1 · Section header and count
		- Garden source (a description, not a file):
			- ```markdown
			  # any block, on any page, that
			  # contains [[tmux]], #tmux,
			  # tags:: tmux, or a ref to an
			  # alias of tmux
			  ```
		- Logseq renders: a header button with a chevron-down icon (14px, stroke `#687076`) and the text "795 Linked References" (16px, weight 500, colour `#11181c`, min-height 32px), and on the right a filter icon (title "Filter", 22px, grey `#687076` at 60% opacity, padding 6px). With a filter on, the header reads "2 of 5 Linked References" (16px, weight 500) and the filter icon is green `#18794e`.
		- DOM and rules:
			- `h2.font-medium` in a foldable header; the chevron appears on hover.
			- Text: `"N Linked Reference(s)"`, or `"F of N …"` while a filter is on.
			- N counts referencing blocks (via the page or any alias), not pages.
			- Filter icon is `a.filter`, 22 px: grey at 60% with no filter, green if all includes, red if all excludes, orange if mixed.
			- Colours seen on the board: green `#18794e`, red `#ce2c31` (chip colour in section 3).
	- ## 2 · A reference group
		- Garden source:
			- ```markdown
			  # journals/2026_08_25.md
			  - # I wonder
			  	- if there is a way that [[Claude/Code]] in [[tmux]] could surface links which, if clicked, would open [[nvim]] in an adjacent pane.
			  ```
		- Logseq renders: a group on a `#f8f9fa` background with no border. The page link "2026-08-25 Tue" (16px, weight 600). Below it a breadcrumb with one crumb "I wonder" (14px, colour `#687076`, margin-left 24px, margin-top 4px). Then the block with a bullet: "if there is a way that `[[Claude/Code]]` in `[[tmux]]` could surface links which, if clicked, would open `[[nvim]]` in an adjacent pane." (brackets grey `#6b7280`).
		- DOM and rules:
			- One group per referencing page: page link, then each matching block.
			- A block that is not top-level gets a breadcrumb of its ancestors' titles above it: `.references-blocks .breadcrumb`, margin-left 1.5rem, muted, joined by 20px `chevron-right` icons.
			- Clicking a crumb zooms to that ancestor.
			- The block renders with its whole subtree.
			- Within a group, blocks whose parent is the page come first.
	- ## 3 · Filter dialog
		- Garden source:
			- ```markdown
			  # per-page filter state is saved as
			  # a page property:
			  filters:: {"nvim" true, "card" false}
			  ```
		- Logseq renders a modal with:
			- Title row: funnel icon (20px, `#687076`) and "Filter" (18px, weight 500).
			- Hint (14px, `#687076`): "Click to include and shift-click to exclude. Click again to remove."
			- "Includes:" (14px, label weight 500) followed by chip "nvim 4" in green `#18794e`.
			- "Excludes:" followed by chip "card 12" in red `#ce2c31`.
			- Input with placeholder "Search in linked pages" (visually hidden label "Search in linked pages"; 15px, padding `8px 10px`, border `1px solid #dee2e6`, radius 6px).
			- Chips of other pages with counts (counts in `#687076`): "Keyshort 40", "tmux/Keyshort 31", "oh-my-tmux 9", "Ghostty 6".
			- Chip style: inline-flex, gap 6px, 14px text, padding `2px 8px`, radius 4px, background `#f1f3f5`.
		- DOM and rules:
			- Modal `.ls-filters`, max-width 704px.
			- Chips list the other pages co-referenced by the linked blocks, with counts, most frequent first; fuzzy search narrows them.
			- In the published build filters work for the session, but the `filters::` write-back cannot persist.
			- The chip counts on the board are illustrative.
	- ## 4 · Unlinked References
		- Garden source:
			- ```markdown
			  - [any block that says tmux as plain text, no brackets or #]
			  ```
		- Logseq renders: a collapsed header button with a chevron-right icon (14px) and the text "Unlinked References" (16px, weight 500, min-height 32px).
		- DOM and rules:
			- `.references.page-unlinked`, collapsed by default.
			- The full-text search for the page name (and aliases) runs only when expanded; the title then becomes `"N Unlinked Reference(s)"`.
			- Same group layout as linked refs.
			- Section order on a page: blocks, Pages tagged with, Linked, Hierarchy, Unlinked.
