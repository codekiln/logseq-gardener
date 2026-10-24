# Logseq reference boards

This folder is the reference for how Logseq's published static site looks and behaves. The parity issue [codekiln/logseq-gardener#106](https://github.com/codekiln/logseq-gardener/issues/106) and its child issues compare the `lsg` output against it.

The target is Logseq 0.10 "publish-spa", as built for `codekiln/logseq-encode-garden`.

## How to use it

- Read the Markdown files (`<Name>.md`). Each one is a text copy of one HTML board, with every measurement, class name, string and example from the board.
- The HTML boards are in [`html/`](html/). They come from a Claude Design canvas and need that canvas's runtime (`support.js`, not in this folder) to render or run. Without it the files are plain markup and script. Read the source or the Markdown instead.
- `html/canvas.json` is the canvas layout (board positions and titles). It carries no behaviour.
- For behaviour (search, sidebars, flashcards, queries, slides), start with `Prototype.md`. It documents the script of the clickable prototype and includes the exact text of the 24 page sources the prototype renders.
- For how a garden file turns into DOM, start with the six spec boards (`Frontmatter`, `Transclusion`, `Backlinks`, `Hierarchy`, `Outline`, `Inline`). Each section gives the garden source, what Logseq renders, and the DOM and rules.

## Boards

| HTML file | Markdown file | Covers |
| --- | --- | --- |
| `html/Prototype.dc.html` | [Prototype.md](Prototype.md) | Clickable prototype: routes, header, sidebars, More menu, export, theme tokens, search, flashcards, filter dialog, slides, Markdown renderer rules, queries, known gaps, page sources |
| `html/Main.dc.html` | [Main.md](Main.md) | Page view: header, left sidebar, title, properties, blocks, tagged pages, references, hierarchy |
| `html/Journals.dc.html` | [Journals.md](Journals.md) | Home route: stacked journal day pages |
| `html/AllPages.dc.html` | [AllPages.md](AllPages.md) | All pages table and paging |
| `html/Overlays.dc.html` | [Overlays.md](Overlays.md) | Search dialog and the More menu in publishing mode |
| `html/Flashcards.dc.html` | [Flashcards.md](Flashcards.md) | Flashcards review modal |
| `html/Graph.dc.html` | [Graph.md](Graph.md) | Graph view and its settings panel |
| `html/Frontmatter.dc.html` | [Frontmatter.md](Frontmatter.md) | Spec: page and block properties, hidden keys, keys that change the page |
| `html/Transclusion.dc.html` | [Transclusion.md](Transclusion.md) | Spec: block refs, block and page embeds, broken targets |
| `html/Backlinks.dc.html` | [Backlinks.md](Backlinks.md) | Spec: Linked References, filter dialog, Unlinked References |
| `html/Hierarchy.dc.html` | [Hierarchy.md](Hierarchy.md) | Spec: namespace pages and the Hierarchy section |
| `html/Outline.dc.html` | [Outline.md](Outline.md) | Spec: bullets, collapse, headings, zoom, tasks |
| `html/Inline.dc.html` | [Inline.md](Inline.md) | Spec: page refs, tags, links, code, emphasis, images, macros |
| `html/Reference.dc.html` | [Reference.md](Reference.md) | Build pipeline, kept and dropped features, tokens, DOM anatomy, feature map |

## Where the values come from

The boards say which values they measured and which they took from Logseq's source. The Markdown keeps those statements.

Measured on the live published build:

- Design tokens in `Reference.md`: colours, type sizes and weights, layout sizes. The board says "measured from the live build, light theme".
- The section order on a page (blocks, Pages tagged with, Linked References, Hierarchy, Unlinked References), "measured on the published `tmux` page" (`Hierarchy.md`).
- Page title size 36px (`Outline.md`), and heading sizes h1 to h4 (the `Prototype/Fixture/Headings` page in `Prototype.md`).
- Counts: 16,693 pages, 430 flashcards, 795 Linked References on `tmux`.

Taken from Logseq's source (file names are on each board):

- Spec boards: `block.cljs`, `block.css`, `reference.cljs`, `reference.css`, `hierarchy.cljs`, `common.css`, `dicts/en.edn`.
- h5 and h6 sizes (`block.css`, per the headings fixture).
- Filter dialog width (`.ls-filters`, max-width 704px) and the strings from `dicts/en.edn`.

Illustrative or placeholder values, marked as such on their boards:

- Filter chip counts on the Backlinks board.
- Node positions on the Graph board (the real layout is a force simulation).
- The `[overdue]` count on the Flashcards board.
- Image and video placeholders on `Inline.md` and `Prototype.md`.

## Conflicts between boards

`Prototype.md` ends its "Known gaps" section with a list of differences between the prototype and the other boards (favorites count, dialog widths, search group names). Read it before treating any one board as the single answer.
