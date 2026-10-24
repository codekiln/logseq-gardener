# Search and More menu

This board shows two overlays on a dark backdrop: the Search dialog with the query "tmux", and the More menu as it appears in publishing mode.

Overall: board 1440x820, padding 64px, background `#3a3f44`, font 'Source Sans 3', text `#11181c`, two columns side by side with gap 64px.

## Search dialog

Column label: "SEARCH — HEADER BUTTON OR ⌘K" (14px, weight 600, letter-spacing 0.06em, colour `#e9ecef`).

Dialog (`role="dialog"`, aria-label "Search"): width 720px, background `#ffffff`, radius 12px, box-shadow `0 24px 64px rgba(0,0,0,0.35)`, overflow hidden.

### Input row

- Flex, gap 12px, padding `16px 20px`, border-bottom `1px solid #e9ecef`.
- Magnifier icon 20px, stroke `#687076`.
- Input (visually hidden label "Search the garden"), value "tmux", 20px, colour `#11181c`, no border, transparent.

### Results

Container padding `8px 8px 12px`. Group labels: 12px, weight 600, letter-spacing 0.06em, colour `#687076`; padding `8px 12px 4px` (PAGES) and `14px 12px 4px` (BLOCKS).

Group "PAGES" (row: flex, gap 10px, padding `8px 12px`, radius 8px, page icon 16px):

1. "tmux" (selected row: background `#edf2fe`, icon stroke `#3e63dd`, text 16px weight 600), with the hint "↵ open" on the right (12px, `#687076`).
2. "**tmux**/Config" (matched text bold, 16px).
3. "**tmux**/Cheat Sheet".
4. "**tmux**/Tutorial/Use **tmux** on Mac".

Group "BLOCKS" (row: flex, gap 10px, padding `8px 12px`, radius 8px; 6px round dot `#9ba1a6`, margin-top 9px; text 15px, line-height 1.5):

1. "…opened the PR in my browser from GitHub/CoPilot/CLI in Ghostty in **tmux**" with the page line "2026-09-15 Tue" (13px, `#687076`).
2. "if there is a way that Claude/Code in **tmux** could surface links which, if clicked, would open nvim in an adjacent pane" with "2026-08-25 Tue".

Other pages' icon stroke is `#687076`. The full search behaviour (groups, filters, keys) is in `Prototype.md`.

## More menu (publishing mode)

Column label: "MORE MENU — PUBLISHING MODE" (same style as above).

Menu (`role="menu"`, aria-label "More"): width 280px, background `#ffffff`, radius 10px, box-shadow `0 16px 48px rgba(0,0,0,0.3)`, padding 6px.

Items (`role="menuitem"`, full width, flex, gap 10px, padding `10px 12px`, 15px, radius 6px, min-height 44px, left aligned, icon 18px stroke `#687076`):

1. "Export graph" (database-with-arrow icon). Shown highlighted: background `#f1f3f5`.
2. "Toggle theme" (light-bulb icon).

Note under the menu (280px wide, 14px, line-height 1.5, colour `#c1c8cd`): "Settings, Plugins, Themes, Import, Join the community and Report bug are left out when the app is built for publishing."

## Notes

- The Search label mentions ⌘K. `Prototype.md` records that ⌘K is not bound in the prototype.
