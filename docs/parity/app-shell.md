# App shell: header, sidebars, More menu and theme toggle

Logseq's published site wraps every page in the same frame: a 48px header with sidebar toggles, search, Home, Graph and More; a 246px left sidebar with Journals, Whiteboards, Flashcards, Graph view, All pages and the garden's favorites; and a main column at most 810px wide. `lsg` has none of this. Each `lsg` page has a plain `<nav>` with one or two links and an Appearance selector above the title, one 55rem column, and its own colour tokens.

Most of the frame can be built from what `lsg` already knows at build time. Favorites come from `:favorites` in `logseq/config.edn`; every favorite in the encode garden is a published page or an alias of one. Several frame items lead to features other issues add (search #118, Journals #110, All pages #117, graph #120, slides #121, right sidebar #119, flashcards #123). This issue adds the frame and its controls; a control whose destination does not exist yet needs a decision in the proposal, listed under [Gaps](#gaps).

Measured on 2026-10-24 against the `gh-pages` build of garden commit `0ce74d6` (served locally at 1280×800 and 390×844) and an `lsg` build of the whole garden from `abb8905` (`mise run parity:build`).

## What Logseq's published site shows

Line numbers are from Logseq 0.10.9 (`frontend` = `src/main/frontend`). The live build agrees with the source except where noted.

### Header

[`components/header.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.cljs) lines 191–288.

- **Left group:** Toggle left sidebar (`t l`, lines 56–63) and Search (`mod+k`, lines 232–241).
- **Right group, in order:**
  - 1. **Home**, only when the route is not the home route (lines 249–251, 27–37). It is absent on `#/` and present on `#/page/tmux`.
  - 2. **Graph**, a text link to `#/graph` shown only when publishing (lines 278–280).
  - 3. **More** (`...`), a dropdown (line 282).
  - 4. **Toggle right sidebar** (`t r`, line 286), hidden below 640px ([`right_sidebar.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/right_sidebar.cljs) lines 27–34).
- Login, sync, plugins, back/forward and Add graph are hidden on a published site. There is no theme button in the header.
- Height `--ls-headbar-height: 3rem` (48px), sticky ([`header.css`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.css) lines 1–17).

### Left sidebar

[`components/container.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/container.cljs) lines 334–579.

- **Items, in order:** Journals (`g j`), Whiteboards, Flashcards with a count badge (430 on the live build), Graph view (`g g`), All pages (lines 419–470). Whiteboards and Flashcards disappear only when `:feature/enable-whiteboards?` or `:feature/enable-flashcards?` is `false`; the encode garden sets neither. The reference boards omit Graph view; the live build shows it.
- **Favorites:** an uppercase FAVORITES label, then one link per entry of `:favorites`, in config order, each shown with the page's original title (lines 202–227, 474). The encode garden lists 19. `github/cli/extension/gh-dash` is shown as `GitHub/CLI/Extension/gh-dash`.
- Recent pages and the New page footer are hidden when publishing (lines 476–489).
- **Default state:** closed. The open state is saved in `localStorage` under `ls-left-sidebar-open?` and restored on the next visit ([`state.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/state.cljs) lines 89, 1477–1484). The reference prototype opens it by default; the source and live build do not.
- **Width:** 246px (`--ls-left-sidebar-width`), drag-resizable from 240 to 460px.
- **At 640px and wider** the open sidebar pushes the main column right (on the live build, the content moved from x=235 to x=358 at 1280px).
- **Below 640px** the sidebar is an overlay 74vw wide (289px at 390px) that slides in over the page, with a shade behind it that closes it on click. Choosing a favorite closes it (lines 401–404, 490–495).

### More menu

[`header.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.cljs) lines 82–151 and [`page_menu.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/page_menu.cljs) lines 83–178.

On a page route the live build shows, in order: **View as slides**, **Export page**, a divider, **Export graph**, **Toggle theme**. On the home route it shows only Export graph and Toggle theme. Toggle theme appears only when publishing (lines 135–138). Export graph opens a dialog with EDN, JSON, Markdown and OPML ([`export.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/export.cljs) lines 17–40). The reference boards omit Export page.

### Theme

Logseq has two themes, light (the default) and dark. Toggle theme and `t t` switch between them, and the choice is saved in `localStorage` under `theme` (`state.cljs` lines 90, 1268–1270). There is no System choice; the `system-theme?` key exists but the published menu offers no way to set it.

### Layout and colours

- Main content max width 810px, 1280px in wide mode (`t w`, saved in `localStorage`); horizontal padding 16px below 640px and 32px above ([`container.css`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/container.css) lines 57–58, 526–540).
- Body font Source Sans 3 (a variable font shipped with the app).
- The live build uses the indigo accent (`data-color="indigo"`), so its colours differ from the classic defaults in `vars-classic.css`. Computed values on `#app-container`:

| Token | Light | Dark |
| --- | --- | --- |
| `--ls-primary-background-color` | `#fbfcfd` | `#151718` |
| `--ls-secondary-background-color` (sidebar) | `#f8f9fa` | `#1a1d1e` |
| `--ls-tertiary-background-color` | `#f1f3f5` | `#202425` |
| `--ls-border-color` | `#e6e8eb` | `#2b2f31` |
| `--ls-primary-text-color` (muted, icons) | `#687076` | `#9ba1a6` |
| `--ls-secondary-text-color` (body text) | `#11181c` | `#ecedee` |
| `--ls-link-text-color` | `#3451b2` | `#849dff` |

The [Prototype](../logseq-reference/Prototype.md) token table lists a fuller set (cards, overlays, header, brackets) and links `#106ba3`; where it differs from the table above, the live build wins.

### Keyboard shortcuts

[`modules/shortcut/config.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/modules/shortcut/config.cljs) binds these outside editing, with no publishing guard: `t l` left sidebar (line 481), `t r` right sidebar (478), `t t` theme (487), `t w` wide mode (516), `g j` journals (384), `g h` home (442), `g a` all pages (445), `g g` graph (448), `mod+k` search (364).

### Phone width

At 390px the header keeps Toggle left sidebar, Search, Home and More; the right sidebar toggle is hidden. The main column is 362px wide with a 14px left edge. The page is 394px wide, so it scrolls sideways by 4px.

## What lsg does today

All shell code is in `crates/logseq-gardener-sdk/src/site/`. `html_document` (`mod.rs` lines 186–210) writes every page as `<nav>{links}{theme control}</nav><main><h1>{title}</h1>{body}</main>`.

- **Links:** "Garden index" to `index.html`, or, with `--homepage`, "Home" and "All pages" (`mod.rs` lines 187–191).
- **Theme:** an Appearance `<select>` with System, Light and Dark, saved in `localStorage` as `logseq-gardener-theme` ([published-system-themes](../../openspec/specs/published-system-themes/spec.md); `site/theme.js`).
- **No header bar, sidebars, favorites, search button, Graph link or More menu.** `lsg` does not read `:favorites` or the `:feature/enable-*` keys. It reads six `config.edn` keys, all about page content (`site/config.rs`).
- **Layout:** `body{max-width:55rem; font:18px/1.6 system-ui}`, no width breakpoints. Colours are `--lsg-*` tokens (`site/theme.css`), for example text `#20252b`, background `#fafafa`, link `#1855a3` in light.
- **Phone width:** the page fits at 390px; images and code scroll inside their boxes.
- **Bytes:** CSS (about 4 KB) and script (8.4 KB) are inlined into every page. [Parity performance](performance.md) budgets 16 KB of script before the title.

## Gaps

Each gap names a check the apply step can run against the fixture pages.

- 1. **Header bar.** A 48px sticky header holds, left to right: Toggle left sidebar, Search, then Home (away from the home page only), Graph, More, Toggle right sidebar. Check: on `tmux` the header controls appear in that order with accessible names; on the home page Home is absent.
- 2. **Header destinations that do not exist yet.** Search (#118), Graph (#120) and the right sidebar (#119) have no `lsg` target. The proposal decides whether each control is omitted, disabled or linked to a placeholder until its issue lands. Check: no control leads to a missing file.
- 3. **Left sidebar items.** Journals, Whiteboards, Flashcards with its count, Graph view and All pages, in that order, each hidden when its `:feature/enable-*` key is `false`. Journals needs #110, All pages needs #117 (`lsg` writes `pages.html` only with `--homepage`), Flashcards needs a card count from #123, and `lsg` has no Whiteboards. Check: the item list and order match Logseq for the encode garden's config, and a fixture config with `:feature/enable-flashcards? false` hides Flashcards.
- 4. **Favorites.** The FAVORITES list follows `:favorites` order and shows each page's original title, resolving aliases (`github/cli/extension/gh-dash` → the `gh-dash` page, shown as `GitHub/CLI/Extension/gh-dash`). The current page is highlighted. Check: the 19 encode-garden favorites appear in order and each link opens the right page; a favorite naming an unpublished page does not produce a broken link.
- 5. **Sidebar state.** Closed by default; `t l` and the header button toggle it; the state survives navigation and reloads. Check: open it on `tmux`, follow a link, and it is still open.
- 6. **Sidebar layout.** 246px and pushing the main column at 640px and wider; a 74vw overlay with a closing shade below 640px. Check: at 1280px the main column moves right by the sidebar width; at 390px it does not move and the page has no sideways scroll.
- 7. **More menu.** On a page: View as slides, Export page, divider, Export graph, Toggle theme. On the home page: Export graph, Toggle theme. View as slides needs #121; export has no `lsg` equivalent. The proposal decides which items `lsg` offers. Check: the menu opens from the keyboard, closes on Escape and outside click, and every item it shows works.
- 8. **Theme control.** Logseq offers only a light/dark toggle in More and `t t`. `lsg` offers Light, Dark and System under its published-system-themes spec. The proposal decides how to fit System into the More menu without dropping the existing spec. Check: Toggle theme in More switches theme and the choice survives a reload.
- 9. **Colours, type and widths.** The light and dark values in the table above, Source Sans 3 (or a documented fallback), page title 36px weight 300, content max width 810px with 16/32px side padding. Check: computed styles on `tmux` match the table in both themes.
- 10. **Keyboard shortcuts.** `t l`, `t t`, `t w`, `g h`, `g a` and `mod+k` (once search exists) work outside text fields. Check: each key changes the page as Logseq does on `tmux`.
- 11. **Phone width.** Header controls stay on one line at 390px, the right sidebar toggle is hidden below 640px, and the page has no sideways scroll (Logseq scrolls by 4px; `lsg` should not copy that). Check: `parity:screenshots` at 390px.
- 12. **Performance.** The shell must not grow per-page cost with the garden: the favorites list is fixed by config, and the script stays within the 16 KB budget. The shell's CSS and script are the same on every page, so they can be one cached file instead of inlined copies. Check: `parity:budgets` passes and page size does not change when 2,000 pages are added.

## Where the work belongs

Reading `:favorites` and the `:feature/enable-*` keys, resolving favorites and aliases to published pages, and counting flashcards are graph questions for the SDK's config and graph model, so the CLI and the language server get the same answers. The publisher only renders the header, sidebar and menu from those answers.

## Fixture pages

| Page | Why |
| --- | --- |
| `tmux` | A favorite (highlighted in the sidebar); Home shown; More shows page items |
| Home route | Home absent; More shows only graph items (Logseq only until #110) |
| `gh-dash` | Favorite reached through the alias `GitHub/CLI/Extension/gh-dash` |
| `Lazygit` | Last favorite in the list; flashcards on the page |

Use the existing `parity:screenshots` fixtures at 1280×800, and add a 390×844 capture for gap 11.

## Out of scope

- What search, Graph view, the right sidebar, slides, flashcards, Journals and All pages do once opened. Each has its own issue under #106.
- Export dialogs. `lsg` publishes static HTML; exporting the graph as EDN or JSON would ship the whole graph, which the performance goals in #106 rule out. The proposal records how Export items are handled.
- Drag-resizing the sidebars and the graph switcher.
- Logseq's 4px sideways scroll at phone width.
