# Flashcards review

This board shows the Flashcards review modal over a dimmed page, with a behaviour note for how cards are found and scored.

Overall: board 1440x900, font 'Source Sans 3', text `#11181c`, backdrop `rgba(17,24,28,0.45)`, modal centred.

## Modal

`role="dialog"`, aria-label "Flashcards": width 760px, background `#ffffff`, radius 12px, box-shadow `0 24px 64px rgba(0,0,0,0.3)`, padding `20px 28px 24px`, column, gap 16px.

- Close button (aria-label "Close"): absolute top 10px, right 10px, 36x36, radius 6px, X icon 18px, colour `#687076`.
- Top row (flex, space-between, padding-right 40px):
  - Left: deck button "All" with an infinity icon (20px) before the label and a chevron-down (14px, `#687076`) after it. 14px, weight 500, padding `6px 4px`, min-height 36px.
  - Right (gap 4px, 14px, `#687076`):
    - "1/430" (title "Current/Total", margin-right 8px).
    - "[overdue]/430" (title "Overdue/Total", margin-right 8px). `[overdue]` is a placeholder for the overdue count.
    - Icon button, aria-label "Toggle preview mode" (36x36, 18px icon).
    - Icon button, aria-label "Toggle random mode" (36x36, 18px icon).
- Card area: border-top `1px solid #e9ecef`, padding-top 12px, 16px.

### Card content shown

```
- **Zoom pane (toggle)** #card
	- Shortcut: `<prefix> z` (same chord toggles zoom **off**; tmux calls this key the **prefix**)
	- [[My Notes]] : **Zooms** the active [[tmux/Pane]] it **fills the window**; other panes in that window are **hidden until you press the same shortcut again** to restore the split layout.
```

The `#card` tag is drawn at 0.9em, opacity 0.7, padding `0 2px`, radius 4px. Inline code is 14px, background `#e9ecef`, radius 3px, padding `1px 4px`.

### Buttons row

Flex, wrap, centre, gap 8px, margin-top 4px. Each button: flex, gap 8px, 14px, weight 500, padding `8px 14px`, min-height 40px, radius 6px. Each has a key hint (`.kbd`: mono 11px, border `1px solid currentColor`, radius 3px, padding `0 4px`, opacity .7).

| Button | Key hint | Style |
| --- | --- | --- |
| Hide answers | `s` | left; text `#11181c`, background `#f1f3f5`, border `1px solid #dee2e6` |
| Forgotten | `f` | right group; white text, background `#ce2c31` |
| Took a while to recall | `t` | right group; white text, background `#3e63dd` |
| Remembered | `r` | right group; white text, background `#18794e` |

A flexible spacer separates "Hide answers" from the three scoring buttons.

## Behaviour note (callout on the board)

Callout: absolute, left 40px, bottom 32px, width 520px, background `#ffffff`, radius 10px, padding `16px 20px`, 14px, line-height 1.55, colour `#3a3f44`. Its text:

- Opened from Flashcards in the left sidebar (count = cards in the graph).
- A card is any block tagged `#card`; its `card-*::` properties are hidden.
- First phase shows only the front block; "Show answers" (`s`) reveals children; `{{cloze}}` adds a third phase.
- Scoring writes back to the block, which a static build cannot persist.

## Notes

- The button reads "Hide answers" on this board, while the callout calls the toggle "Show answers"; the label presumably flips with the phase. `Prototype.md` documents the button labels per phase.
