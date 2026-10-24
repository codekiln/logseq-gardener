# Reference sheet

This board explains how the published garden is built and drawn: the build pipeline, what publishing mode keeps and drops, design tokens, and the DOM hooks.

Header text: "LOGSEQ-ENCODE-GARDEN · STATIC SITE", title "How the published garden is built and drawn". Summary: the site is the Logseq web app compiled in publishing mode; the whole graph ships as data inside one page, and the app renders it read-only on GitHub Pages.

## 1 · How it ships

Workflow: `.github/workflows/gh-pages.yml`, on every push to main. Five steps:

1. **Guard the sentinel.** Fails the build if `pages/` or `journals/` contain a literal `logseq____&amp;`-style token, which would corrupt the embedded JSON and blank the site. (The board's HTML source shows the token as `logseq____&amp;amp;`, which displays as `logseq____&amp;`.)
2. **logseq/publish-spa.** Builds the app from graph `.` into `www/` with accent color `indigo`. All pages are public: `:publishing/all-pages-public? true`.
3. **Add .nojekyll.** Stops GitHub Pages from running Jekyll over the build, so paths with underscores are served as-is.
4. **Compress the graph.** `compress_logseq_db.py` moves the inline `window.logseq_db` into `static/js/db.gz` and swaps in a loader, keeping `index.html` under GitHub's 100 MB limit.
5. **Deploy to gh-pages.** Pushes `www/` to the `gh-pages` branch with a clean deploy. To preview locally, extract that branch and serve it over http on localhost.

## 2 · What publishing mode keeps

- Journals home
- All pages table (16,693 pages)
- Flashcards (430 cards)
- Favorites in the left sidebar
- Search (pages and blocks)
- Graph view
- Linked and unlinked references
- Namespace hierarchy
- More › Export graph
- More › Toggle theme

## What it drops

- Block and page editing
- Settings
- Plugins and Themes
- Import
- Join the community
- Report bug
- Pages under `:hidden` in config.edn
- Pages with `public:: false`
- Flashcard scoring write-back
- Saved reference filters

## Feature map

Where each feature is specified on the canvas (and in this folder):

| Feature | Covers | Markdown file |
| --- | --- | --- |
| Page view | layout, header, sidebar | `Main.md` |
| Journals | home feed | `Journals.md` |
| All pages | table, paging | `AllPages.md` |
| Search | "⌘K dialog" (board wording) | `Overlays.md`, `Prototype.md` |
| Flashcards | review modal | `Flashcards.md` |
| Graph view | settings panel | `Graph.md` |
| Frontmatter | page and block properties | `Frontmatter.md` |
| Transclusion | block refs, embeds | `Transclusion.md` |
| Backlinks | linked, filters, unlinked | `Backlinks.md` |
| Hierarchy | namespace paths | `Hierarchy.md` |
| Outline | bullets, collapse, zoom, tasks | `Outline.md` |
| Inline | refs, tags, links, macros | `Inline.md` |

## 3 · Tokens

Measured from the live build, light theme (board wording).

### Colours

| Hex | Role |
| --- | --- |
| `#ffffff` | primary-background (swatch has border `#dee2e6`) |
| `#f8f9fa` | secondary-background, properties (swatch has border `#dee2e6`) |
| `#e9ecef` | tertiary-background, border, guideline |
| `#11181c` | body text |
| `#687076` | muted text, icons |
| `#6b7280` | `[[ ]]` brackets |
| `#106ba3` | link, page-ref, tag |
| `#3e63dd` | indigo accent (`rx-indigo-09`) |

### Type

| Element | Size / weight |
| --- | --- |
| Page title | 36 / 300, colour `#000000` (sample "tmux") |
| h1 block | 32 (2em) / 300 (sample "ideas") |
| Section heading | 16 / 700 (sample "795 Linked References") |
| Block | 16 / 400, line-height 1.6 |
| Property | 12.8 / 500 key (sample `created-by: [[Person/Nicholas Marriott]]`) |

Font family: Source Sans 3 (variable), fallbacks Source Han Sans, sans-serif.

### Layout

| Part | Value |
| --- | --- |
| Header | 48 px, `rgba(255,255,255,0.9)` |
| Left sidebar | 246 px (`--ls-left-sidebar-width`) |
| Content column | 810 px max (`--ls-main-content-max-width`), centered |
| Properties box | `#f8f9fa`, radius 4, padding 4 32 4 8 |
| Bullet | 6 px dot in a 16 px column; children indent behind a 1 px `#e9ecef` guideline |

## 4 · Anatomy

DOM hooks for custom.css or tests:

| Selector | Meaning |
| --- | --- |
| `#head` | header bar |
| `#left-sidebar` | navigation and favorites |
| `#main-content-container` | scrolling page |
| `.page-title .title` | page name |
| `.page-properties` | first-block properties |
| `.ls-block .bullet-container` | outline |
| `.references.page-tags` | pages tagged with |
| `.references.page-linked` | linked references |
| `.page-hierarchy` | namespace tree |
| `.references.page-unlinked` | unlinked references |
| `window.logseq_db` | graph data, loaded from `db.gz` |
| `#/page/<name>`, `#/all-pages` | hash routes |

## Notes

- Block h1 is 32px here (2em) and 28px on Main/Journals for the top block heading; those boards draw their own top blocks at 28px, weight 300. Treat 2em as the Logseq rule and 28px as the board's sample.
