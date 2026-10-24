# Spec: outline and blocks

This board specifies how Logseq renders blocks: every line is a block with a bullet, and indentation is structure.

- Gardener specs: `authored-outline-collapse`, `selected-block-navigation`.
- Logseq source: `block.cljs › block-container, breadcrumb`; `block.css .bullet-container, .block-children-container, .ls-block h1–h6`.
- Board bullet CSS: `.dot span` 6px circle, `#8b8d98` at opacity .8; `.kids` margin-left 7px, padding-left 22px, border-left `1px solid #e9ecef` (the board's approximation of Logseq's 29px children margin).

## 1 · Bullets and nesting

Garden source:

```markdown
- ideas
	- [[tmux/Idea/Toggl Track status in status line]]
- learning
	- [[tmux/man]]
```

Logseq renders: two top-level bullets "ideas" and "learning", each with one indented child showing a page link with grey brackets.

DOM and rules:

- `div.ls-block` contains `.bullet-container` (16×16, round) holding `.bullet` (6×6, gray-08 at 80% opacity).
- Body text 16px, line-height 1.6.
- Children sit in `.block-children-container` (margin-left 29px) with a 1px `#e9ecef` guideline (`.block-children-left-border`, darkens on hover).
- Clicking a bullet zooms into the block.

## 2 · Collapsed blocks

Garden source:

```markdown
- extensions
  collapsed:: true
	- [[oh-my-tmux]] — by [[Person/Grégory Pakosz]]
```

Logseq renders: a bullet "extensions" with a halo around the dot (`box-shadow: 0 0 0 5px rgba(0,0,0,0.08)` on the board). Children are not drawn. The board's note: "Children not drawn. Hovering shows a disclosure arrow left of the bullet that expands them in place." (13px, `#687076`).

DOM and rules:

- `collapsed:: true` (hidden property) gives `.bullet-container.bullet-closed`: the 16px circle fills with gray-04-alpha, a halo round the dot.
- Children are omitted from the DOM until expanded.
- A static build should ship children and toggle them client-side.

## 3 · Heading blocks

Garden source:

```markdown
- # Spotify [[TUI]] Projects
- ## [[spotify-player]]
- ### Agent skill
- #### Context
```

Logseq renders four bullets, each centred on its heading line with line-height 1.5:

| Source | Size on board |
| --- | --- |
| `# Spotify [[TUI]] Projects` | 32px |
| `## [[spotify-player]]` | 24px |
| `### Agent skill` | 19.2px |
| `#### Context` | 16px |

DOM and rules:

- `.ls-block h1` 2em, `h2` 1.5em, `h3` 1.2em, `h4` 1em, `h5` 0.83em.
- The bullet stays and centres on the heading line.
- The page title is separate: `h1.title`, 36px measured.

## 4 · Zoomed-in block

Garden source:

```markdown
# URL: #/page/6a3f6744-5403-429d-ad4a-4391e01224ea
```

Logseq renders: a breadcrumb "spotify-player" (link, colour `#687076`) then a 20px chevron-right (opacity 0.5, margin `0 4px`) then "Features" (14px, `#687076`, 10px below). Then the block "Features" as a 24px heading line with a bullet, and one child "configurable" (link).

DOM and rules:

- Route `#/page/<uuid>`.
- Breadcrumb: page name, then each ancestor's title, joined by a 20px `chevron-right` at 50% opacity with `mx-1`.
- The block becomes the root of the outline; no page title, properties or references below it.

## 5 · Tasks and numbered text

Garden source:

```markdown
- TODO [task text]
- DONE [task text]
- 1. First item
```

Logseq renders:

- TODO: unchecked 16px checkbox (aria-label "TODO", gap 8px) and "[task text]".
- DONE: checked checkbox (aria-label "DONE"), text struck through, whole row at opacity 0.6.
- "1. First item" as plain text.

DOM and rules:

- Leading `TODO`/`LATER` gives an unchecked box.
- `DONE` gives a checked box, struck through, dimmed.
- `NOW`/`DOING` show a marker label (`.block-marker`, 85%, opacity .7).
- Checkboxes are read-only when published.
- The garden writes ordered lists as literal `1.` text under a bullet, so that renders verbatim.
