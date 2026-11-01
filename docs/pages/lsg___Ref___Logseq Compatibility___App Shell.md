tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Page View]], [[lsg/Ref/Logseq Published Site/Search and More Menu]], [[lsg/Ref/Site Navigation]]

- # App Shell Compatibility
	- `lsg` wraps every page in Logseq's frame: a 48px sticky header with Toggle left sidebar, Home, Graph, More and Toggle right sidebar; a 246px left sidebar that starts closed and lists Journals, Flashcards with its due count, Graph view, All pages and the garden's favorites in config order; Logseq's indigo colours in light and dark; an 810px main column with a 1280px wide mode; and Logseq's shell shortcuts. It leaves out Whiteboards, Export page and Export graph, follows the system's light or dark setting until the reader chooses one and adds Use system theme to the More menu, places Search on the right of the header, and names Source Sans 3 without shipping the font.
	  id:: a1be0492-f599-40e8-8508-0240660ae635
	- Measured on 2026-10-24 against the `gh-pages` build of garden commit `0ce74d6` (served locally at 1280×800 and 390×844) and an `lsg` build of the whole garden from `abb8905` (`mise run parity:build`). The OpenSpec change [match-logseq-app-shell](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-25-match-logseq-app-shell) then built the frame, and [published-app-shell](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-app-shell/spec.md) and [published-system-themes](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-system-themes/spec.md) hold its requirements.
	- ## What Logseq's published site shows
		- Line numbers are from Logseq 0.10.9 (`frontend` = `src/main/frontend`). The live build agrees with the source except where noted.
		- ### Header
			- [`components/header.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.cljs) lines 191–288.
			- **Left group:** Toggle left sidebar (`t l`, lines 56–63) and Search (`mod+k`, lines 232–241).
			- **Right group, in order:**
				- **Home**, only when the route is not the home route (lines 249–251, 27–37). It is absent on `#/` and present on `#/page/tmux`.
				- **Graph**, a text link to `#/graph` shown only when publishing (lines 278–280).
				- **More** (`...`), a dropdown (line 282).
				- **Toggle right sidebar** (`t r`, line 286), hidden below 640px ([`right_sidebar.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/right_sidebar.cljs) lines 27–34).
			- Login, sync, plugins, back/forward and Add graph are hidden on a published site. There is no theme button in the header.
			- Height `--ls-headbar-height: 3rem` (48px), sticky ([`header.css`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.css) lines 1–17).
		- ### Left sidebar
			- [`components/container.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/container.cljs) lines 334–579.
			- **Items, in order:** Journals (`g j`), Whiteboards, Flashcards with a count badge (430 on the live build), Graph view (`g g`), All pages (lines 419–470). Whiteboards and Flashcards disappear only when `:feature/enable-whiteboards?` or `:feature/enable-flashcards?` is `false`; the encode garden sets neither. The reference boards omit Graph view; the live build shows it.
			- **Favorites:** an uppercase FAVORITES label, then one link per entry of `:favorites`, in config order, each shown with the page's original title (lines 202–227, 474). The encode garden lists 19. `github/cli/extension/gh-dash` is shown as `GitHub/CLI/Extension/gh-dash`.
			- Recent pages and the New page footer are hidden when publishing (lines 476–489).
			- **Default state:** closed. The open state is saved in `localStorage` under `ls-left-sidebar-open?` and restored on the next visit ([`state.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/state.cljs) lines 89, 1477–1484). The reference prototype opens it by default; the source and live build do not.
			- **Width:** 246px (`--ls-left-sidebar-width`), drag-resizable from 240 to 460px.
			- **At 640px and wider** the open sidebar pushes the main column right (on the live build, the content moved from x=235 to x=358 at 1280px).
			- **Below 640px** the sidebar is an overlay 74vw wide (289px at 390px) that slides in over the page, with a shade behind it that closes it on click. Choosing a favorite closes it (lines 401–404, 490–495).
		- ### More menu
			- [`header.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/header.cljs) lines 82–151 and [`page_menu.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/page_menu.cljs) lines 83–178.
			- On a page route the live build shows, in order: **View as slides**, **Export page**, a divider, **Export graph**, **Toggle theme**. On the home route it shows only Export graph and Toggle theme. Toggle theme appears only when publishing (lines 135–138). Export graph opens a dialog with EDN, JSON, Markdown and OPML ([`export.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/export.cljs) lines 17–40). The reference boards omit Export page.
		- ### Theme
			- Logseq has two themes, light (the default) and dark. Toggle theme and `t t` switch between them, and the choice is saved in `localStorage` under `theme` (`state.cljs` lines 90, 1268–1270). There is no System choice; the `system-theme?` key exists but the published menu offers no way to set it.
		- ### Layout and colours
			- Main content max width 810px, 1280px in wide mode (`t w`, saved in `localStorage`); horizontal padding 16px below 640px and 32px above ([`container.css`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/container.css) lines 57–58, 526–540).
			- Body font Source Sans 3 (a variable font shipped with the app).
			- The live build uses the indigo accent (`data-color="indigo"`), so its colours differ from the classic defaults in `vars-classic.css`. Computed values on `#app-container`:
			- | Token | Light | Dark |
			  | --- | --- | --- |
			  | `--ls-primary-background-color` | `#fbfcfd` | `#151718` |
			  | `--ls-secondary-background-color` (sidebar) | `#f8f9fa` | `#1a1d1e` |
			  | `--ls-tertiary-background-color` | `#f1f3f5` | `#202425` |
			  | `--ls-border-color` | `#e6e8eb` | `#2b2f31` |
			  | `--ls-primary-text-color` (muted, icons) | `#687076` | `#9ba1a6` |
			  | `--ls-secondary-text-color` (body text) | `#11181c` | `#ecedee` |
			  | `--ls-link-text-color` | `#3451b2` | `#849dff` |
			- The token table on [[lsg/Ref/Logseq Published Site/Clickable Prototype]] lists a fuller set (cards, overlays, header, brackets) and links `#106ba3`; where it differs from the table above, the live build wins.
		- ### Keyboard shortcuts
			- [`modules/shortcut/config.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/modules/shortcut/config.cljs) binds these outside editing, with no publishing guard: `t l` left sidebar (line 481), `t r` right sidebar (478), `t t` theme (487), `t w` wide mode (516), `g j` journals (384), `g h` home (442), `g a` all pages (445), `g g` graph (448), `mod+k` search (364).
		- ### Phone width
			- At 390px the header keeps Toggle left sidebar, Search, Home and More; the right sidebar toggle is hidden. The main column is 362px wide with a 14px left edge. The page is 394px wide, so it scrolls sideways by 4px.
	- ## What lsg publishes
		- [[lsg/Ref/Site Navigation]] describes the header and sidebars for readers.
		- ### Header
			- Every page starts with a sticky header 48px tall, with the class `cp__header`. Toggle left sidebar sits at the left. Home, Graph and More sit at the right edge, followed from 640px wide by Toggle right sidebar with the `layout-sidebar-right` icon and `aria-expanded`.
			- Home links to `index.html` and is absent from `index.html`.
			- Graph is an `a.button` linking to `graph.html#/graph`, with the `hierarchy` icon and the accessible name Graph, so Logseq's selector `.cp__header a.button[href$="#/graph"]` matches it and a stylesheet written for Logseq's header, such as the encode garden's, hides it.
			- With `--search`, a Search button sits between Graph and More, and Cmd/Ctrl+K opens the same dialog.
			- Each control has an accessible name and visible keyboard focus, and the header holds only controls whose destination exists in the site.
		- ### Left sidebar
			- The sidebar lists Journals when a journals feed is published, Flashcards unless `logseq/config.edn` sets `:feature/enable-flashcards? false`, Graph view, All pages, then a FAVORITES heading and one link per favorite.
			- Journals links to the feed's first batch and is marked current on every feed batch. Flashcards carries the `infinity` icon, the `g f` hint and a rounded badge with the number of cards due at build time, left out when none is due. Graph view links to `graph.html` and carries the `g g` hint when that shortcut is on. All pages links to the page index, `pages.html` when a homepage or journals feed takes `index.html`.
			- Favorites come from `:favorites` in `logseq/config.edn`, in config order. Each resolves through page titles and aliases and shows the title or alias as the author spelled it, so `github/cli/extension/gh-dash` shows as `GitHub/CLI/Extension/gh-dash` and the encode garden lists 19 favorites. The current page's link carries `aria-current="page"` and is highlighted. The FAVORITES heading is absent when no favorite resolves.
			- The sidebar starts closed. The header button or `t l` toggles it, and the state is saved under `logseq-gardener-left-sidebar` in browser localStorage and applied before first paint, so a page opens with the main column already shifted.
			- From 640px wide the open sidebar is 246px and pushes the main column right. Below 640px it opens as a 74vw overlay over a shade; the shade, a sidebar link or Escape closes it, and each page load starts with the overlay closed.
		- ### More menu
			- On page routes from 640px wide, More lists View as slides first. Toggle theme follows, and Use system theme appears while a light or dark choice is saved.
			- View as slides is absent from the home page, from All pages and below 640px.
			- The menu opens from the keyboard with focus on its first item, moves between items with the arrow keys, and closes on Escape, on a click outside it and after a choice, returning focus to More.
		- ### Theme
			- The site starts in the system's light or dark appearance. Toggle theme and `t t` switch between light and dark and save the choice under `logseq-gardener-theme`; Use system theme removes the saved choice. `window.logseqGardenerInitTheme(root)` still sets up an Appearance control that a host inserts, as [[lsg/Ref/Host Integration API]] describes.
		- ### Layout and colours
			- `theme.css` takes Logseq's indigo values as its defaults under the `--lsg-*` variable names: background `#fbfcfd`/`#151718`, sidebar `#f8f9fa`/`#1a1d1e`, surface `#f1f3f5`/`#202425`, border `#e6e8eb`/`#2b2f31`, muted text `#687076`/`#9ba1a6`, text `#11181c`/`#ecedee`, link `#3451b2`/`#849dff`.
			- The main column is at most 810px wide, or 1280px in wide mode, with 16px side padding below 640px and 32px from 640px. `t w` toggles wide mode and saves it under `logseq-gardener-wide`.
			- Body text uses `"Source Sans 3", "Source Sans Pro", system-ui, sans-serif` at 16px with line height 1.5. The page title's size and weight belong to [[lsg/Ref/Logseq Compatibility/Page Outline]].
		- ### Keyboard shortcuts
			- Pages handle `t l`, `t r` (from 640px wide), `t t`, `t w`, `g h` (the site root's `index.html` from any page address), `g a`, `g g` unless `logseq/config.edn` sets `:shortcuts {:go/graph-view false}`, `alt+shift+c` (the Contents card, from 640px wide), `g f` and `t c` (the flashcard review) unless flashcards are turned off, and `g j` when a journals feed is published. Search adds Cmd/Ctrl+K.
			- The second key of a two-key shortcut must follow the first within one second. Shortcuts are ignored while focus is in an input, textarea, select or editable element, or while Control or Meta is held, and the two-key shortcuts also while Alt is held.
		- ### Phone width
			- At 390px the header stays on one line, Toggle right sidebar is hidden, and no page scrolls sideways.
		- ### Cost per page
			- The shell's behaviour is one shared `lsg-shell.js` at the site root, loaded with `defer` by every page, including the loader that imports the flashcard review on first use. The `[shell]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) limits it to 6.9 KB uncompressed, and the inline shell styles plus the header and sidebar markup to 6 KB per page with the encode garden's favorites. `parity:budgets` checks both in CI and checks that pages keep their size when 2,000 unrelated pages are added.
			- The shell's styles stay inline because the header must lay out before first paint. Without JavaScript the sidebar stays closed, its toggle button, More and Toggle right sidebar are hidden, and Home and the page content remain usable.
			- On 2026-10-26 (`lsg` `b490d68`), every fixture page carried 14.4 to 14.6 KB of script before its title, within the 16 KB budget. [[lsg/Ref/Logseq Compatibility/Performance]] holds every row.
		- The SDK's `garden_settings` module reads `:favorites`, `:default-home {:sidebar …}` and the `:feature/enable-*` keys with the bounded EDN reader, each flag defaulting to true, and resolves favorites through the same title and alias lookup that page links use, with a diagnostic for each favorite that names no published page. The publisher renders the header, sidebar and menu from those answers.
	- ## Differences from Logseq
		- Whiteboards is left out of the left sidebar. `lsg` publishes no whiteboards, and the item would open an empty list.
		- Export page and Export graph are left out of More. Exporting the graph as EDN or JSON would ship the whole graph to every reader, and a reader can save the static page.
		- The starting appearance follows the system, as the published-system-themes specification requires, so a reader whose system is dark sees no flash of light colours. Logseq starts light. Use system theme, which Logseq lacks, appears in More while a light or dark choice is saved.
		- The Search button sits on the right of the header, between Graph and More. Logseq places it in the left group after Toggle left sidebar.
		- `lsg` names Source Sans 3 first in its font stack and ships no font file, so readers without the font see the system font. A font file would add to the data loaded before first paint on every first visit and needs a licence notice.
		- A favorite that names no published page is left out and reported as a diagnostic. Logseq links it to an empty page; `lsg` would have no file to link.
		- At 390px no page scrolls sideways. Logseq's page scrolls by 4px.
		- The left sidebar is 246px wide. Logseq's can be dragged to any width from 240 to 460px.
		- There is no graph switcher.
	- ## Before the change
		- The `lsg` build measured on 2026-10-24, from `abb8905`, wrote every page from `html_document` in `crates/logseq-gardener-sdk/src/site/mod.rs` (lines 186–210) as `<nav>{links}{theme control}</nav><main><h1>{title}</h1>{body}</main>`.
		- **Links:** "Garden index" to `index.html`, or, with `--homepage`, "Home" and "All pages" (`mod.rs` lines 187–191).
		- **Theme:** an Appearance `<select>` with System, Light and Dark, saved in `localStorage` as `logseq-gardener-theme` ([published-system-themes](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-system-themes/spec.md); `site/theme.js`).
		- **No header bar, sidebars, favorites, search button, Graph link or More menu.** `lsg` read neither `:favorites` nor the `:feature/enable-*` keys. It read six `config.edn` keys, all about page content (`site/config.rs`).
		- **Layout:** `body{max-width:55rem; font:18px/1.6 system-ui}`, no width breakpoints. Colours were `--lsg-*` tokens (`site/theme.css`), for example text `#20252b`, background `#fafafa`, link `#1855a3` in light.
		- **Phone width:** the page fit at 390px; images and code scrolled inside their boxes.
		- **Size:** CSS (about 4 KB) and script (8.4 KB) were inlined into every page, against the 16 KB budget for script before the title.
	- ## Fixture pages
		- `parity:screenshots` captures these pages at 1280×800 and at 390×844.
		- | Page | Why |
		  | --- | --- |
		  | `tmux` | A favorite (highlighted in the sidebar); Home shown; More shows page items |
		  | Home route | Home absent; More shows no page items |
		  | `gh-dash` | Favorite reached through the alias `GitHub/CLI/Extension/gh-dash` |
		  | `Lazygit` | Last favorite in the list; flashcards on the page |
