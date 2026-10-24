# Page view

This board shows a published Logseq page ("tmux") as a reader sees it: header, left sidebar, and a main column with properties, blocks, tagged pages, Linked References, Hierarchy and Unlinked References.

Overall: board 1440 wide, min-height 1180px, font 'Source Sans 3', text colour `#11181c`, background `#ffffff`, base size 16px. Links `#106ba3` (hover `#0b4f7a`, underline), grey brackets `.br` `#6b7280`.

## Header

Header (sticky on Main), height 48px, flex, space-between, padding `0 12px`, background `rgba(255,255,255,0.9)`, border-bottom `1px solid #e9ecef`.

- Left group (gap 4px): icon button "Toggle left sidebar" (aria-label; three horizontal lines, 20px icon) and icon button "Search" (aria-label; magnifier).
- Right group (gap 4px): text link "Graph" (14px, weight 500, colour `#687076`, padding `8px 10px`, radius 6px), icon button "More" (aria-label; three dots), icon button "Toggle right sidebar" (aria-label; square with a right-hand divider at x=15).
- Icon button (`.icon`): 36x36, radius 6px, no border, colour `#687076`; hover background `#f1f3f5`, colour `#11181c`.
- There is no Home button on this board (the prototype shows Home only away from Journals; see `Prototype.md`).

## Left sidebar

`nav` (aria-label "Left sidebar"): flex `1 1 246px`, max-width 100%, background `#f8f9fa`, border-right `1px solid #e9ecef`, padding `12px 8px`, column, gap 2px, min-height 1132px.

Nav button (`.navbtn`): flex, gap 10px, padding `6px 10px`, radius 6px, colour `#11181c`, 15px, min-height 32px; hover background `#e9ecef`. Icons 18px, stroke `#687076`.

Items in order:

1. "Journals" (calendar icon; links to the Journals board).
2. "Whiteboards" (chart-in-frame icon).
3. "Flashcards" with a count badge "430" (12px, weight 600, background `#e9ecef`, colour `#3a3f44`, radius 9px, padding `1px 7px`).
4. "All pages" (page icon; links to the All pages board).

Then the label "FAVORITES" (12px, weight 600, letter-spacing 0.06em, colour `#687076`, margin `18px 10px 6px`) and favorites (14px, each with a 6px round dot `#9ba1a6`), in this order: fd, gh-dash, tuicr, LazyVim, Obsidian, chezmoi, mise, nvim, tmux, yazi, Lazygit.

The current page "tmux" is highlighted: background `#e9ecef`, dot colour `#3e63dd`.

## Main column

`main`: flex `999 1 560px`, padding `32px 24px 64px`; content wrapper max-width 810px, centred.

### Page title

`h1` "tmux": 36px, weight 300, colour `#000000`, line-height 1.3, margin-bottom 16px.

### Page properties box

Background `#f8f9fa`, radius 4px, padding `4px 32px 4px 8px`, 12.8px, line-height 1.6, margin-bottom 12px. Keys weight 500.

- `created-by:` [[Person/Nicholas Marriott]]
- `logseq-entity:` [[Logseq/Entity/Software/Project]]

### Blocks

Block row: flex, gap 8px, line-height 1.6, margin `2px 0`. Bullet cell 16px wide, 25px high; dot 6px, colour `#9ba1a6`. Children indent: margin-left 7px, padding-left 22px, border-left `1px solid #e9ecef`.

Outline (top block is a 28px, weight 300, line-height 1.5 heading):

```
tmux                                  (28px, weight 300)
  [[GitHub/Star]]s: 47,343 as of [[2026-07-04 Sat]]
  A terminal multiplexer: run many terminal sessions in one screen, detach and reattach them, and script window and pane layouts.
  ideas
    [[tmux/Idea/Pomodoro with status line and silent flash]]
    [[tmux/Idea/Toggl Track status in status line]]
  customization
    [[tmux/Config]]
    [[Catppuccin]]
  extensions
    [[oh-my-tmux]] — by [[Person/Grégory Pakosz]]
  learning
    [[tmux/Docs/Getting Started]]
    [[tmux/man]]
```

### Pages tagged with "tmux"

Section margin-top 40px. Heading `h2` 'Pages tagged with "tmux"' (16px, weight 700, margin-bottom 8px). Links (15px, gap 4px, padding-left 4px):

- Person/Brian P. Hogan/Book/25/tmux 3
- tmux/Tutorial/Use tmux on Mac

### Linked References

Section margin-top 32px. Header button: chevron-down (14px, stroke `#687076`) and "795 Linked References" (16px, weight 700, min-height 32px).

One group (margin-top 10px, background `#f8f9fa`, radius 6px, padding `12px 16px`): page link "2026-09-15 Tue" (15px, weight 600), then block: "I was pleasantly surprised to find this morning that clicking on `Opened PR #101` in [[GitHub/CoPilot/CLI]] in [[Ghostty]] in [[tmux]] actually opened the PR in my browser". The inline code is 14px, background `#e9ecef`, radius 3px, padding `1px 4px`.

### Hierarchy

Section margin-top 32px. `h2` "Hierarchy" (16px, weight 700). Rows (15px, gap 4px); segments are links, separator "/" in `#6b7280`, no brackets on this board:

- tmux/Cheat Sheet
- tmux/Command/new-session
- tmux/Config
- tmux/Docs/Getting Started
- tmux/How To/Reload

### Unlinked References

Section margin-top 32px. Collapsed header button: chevron-right (14px) and "Unlinked References" (16px, weight 700, min-height 32px).

Section order: properties, blocks, Pages tagged with, Linked References, Hierarchy, Unlinked References.

## Notes

- The Hierarchy list here has five rows and includes `tmux/How To/Reload`; the Hierarchy spec board (`Hierarchy.md`) lists six rows with different members and shows brackets. This board draws a simplified list; use the spec board for rules.
