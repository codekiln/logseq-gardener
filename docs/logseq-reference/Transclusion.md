# Spec: transclusion

This board specifies how Logseq renders block references, block embeds, page embeds and broken targets: refs show text inline, embeds draw the source in a grey well.

- Gardener specs: `selected-content-transclusion`, `selected-block-navigation`.
- Logseq source: `block.cljs › block-reference, block-embed, page-embed`; `block.css .block-ref, .color-level, .embed-page`.

## 1 · Block reference

Garden source:

```markdown
- Video coverage: ((6aa57707-6afe-46ec-bf96-766804d117dc)) in [[Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs]]

# target block, elsewhere:
- ### {{youtube-timestamp 620}} Agent skill
  id:: 6aa57707-6afe-46ec-bf96-766804d117dc
```

Logseq renders: "Video coverage: 10:20 Agent skill in [[Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs]]". The ref text "10:20 Agent skill" is link-coloured with a thin underline; the page link keeps grey brackets. (Colours on the board: link `#106ba3`, brackets `#6b7280`.)

```html
<span class="block-ref">10:20 Agent skill</span>
```

DOM and rules:

- `span.block-ref`: the target block's title rendered inline (its own markup, minus properties and children).
- Style: link colour (`#106ba3`), `border-bottom: 0.5px solid` (same colour), `padding: 2px 0`, `cursor: alias`, no brackets.
- Click opens the target block zoomed in (`#/page/<uuid>`).
- Hover for 1 s shows a popover with the block and its children.
- The target needs an `id::` property, which stays hidden.

## 2 · Block embed

Garden source:

```markdown
- ## [[spotify-player]]
	- {{embed ((6a3f6744-5403-429d-ad4a-4391e01224ea))}}

# target, in spotify-player.md:
- ## Features
  id:: 6a3f6744-5403-429d-ad4a-4391e01224ea
	- [configurable](https://github.com/…/config.md)
	- To enable fuzzy search, build with the `fzf` feature (disabled by default).
```

Logseq renders: a bullet with the heading "[[spotify-player]]" (24px, weight 400, line-height 1.5, brackets grey). Its child block holds a grey well containing the target block "Features" (24px, line-height 1.5) with its own bullet, and below it two child bullets: a link "configurable", and "To enable fuzzy search, build with the `fzf` feature (disabled by default)." (`fuzzy search` is a link; `fzf` is inline code at 14px).

DOM and rules:

- `div.color-level.embed-block` inside the host block's content.
- Padding `4px 12px 8px` (`px-3 pt-1 pb-2`), background gray-02 (`#f8f9fa` on the board).
- Contains the target block and all its children as a full outline with their own bullets.
- Nested embeds step through `--color-level-2…6`, so each well is a shade darker. On the board: `.embed` is `#f8f9fa`, `.embed .embed` is `#f1f3f5`; the `.embed` rule is `background:#f8f9fa;border-radius:4px;padding:4px 12px 8px;margin:4px 0`.

## 3 · Page embed

Garden source:

```markdown
- # Logseq Frontmatter
	- {{embed [[Logseq/Frontmatter/alias]]}}
```

Logseq renders: a grey well (padding 8px) with a header row (16px page-file icon in `#687076`, gap 12px, padding 4px, weight 600, 5px gap below) holding the link "Logseq/Frontmatter/alias". Below it every top-level block of that page: first block "`alias::` for more convenient or alternate names" (24px, line-height 1.5; the code is 20px), with a child: "In Logseq, if page [[Super Long Name]] has frontmatter `alias:: [[Shortname]]`, then clicking [[Shortname]] anywhere in the graph navigates to [[Super Long Name]]." (inline code 14px).

DOM and rules:

- `div.color-level.embed.embed-page`, padding 8px, margin 8px 0.
- Header is `section.embed-header`: page icon plus page link, weight 600, 5px gap below.
- Then every top-level block of the page.
- Embedding the page you are on (or one already above in the chain) draws the header only, which stops recursion.

## 4 · Broken targets

Garden source:

```markdown
- see ((00000000-0000-0000-0000-000000000000))
```

Logseq renders: "see " followed by the raw text `((00000000-0000-0000-0000-000000000000))` in colour `#b54708` with a 4px right margin, with tooltip "Block ref invalid".

```html
<span class="warning" title="Block ref invalid">((00000000-0000-0000-0000-000000000000))</span>
```

DOM and rules:

- An unknown uuid renders the raw text in `span.warning` with title "Block ref invalid".
- For a static build that withholds pages, a ref into a withheld page must not leak its text: show the warning form or nothing.
