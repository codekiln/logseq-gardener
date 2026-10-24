## Why

Logseq's published site wraps every page in the same frame: a header with a left-sidebar toggle, Home and a More menu; a left sidebar with All pages and the garden's favorites; and a main column at most 810px wide in Logseq's colours. An `lsg` page has a plain link row and an Appearance selector above a 55rem column, so readers who know the encode garden's Logseq site lose the favorites and the familiar layout ([app shell analysis](../../../docs/parity/app-shell.md), [Issue #109 — Match Logseq's header, sidebars, More menu and theme toggle](https://github.com/codekiln/logseq-gardener/issues/109)).

## What Changes

- Every generated page gets a sticky 48px header. Left: Toggle left sidebar. Right: Home (on every page except `index.html`) and More. Search, Graph and Toggle right sidebar join the header when their issues ([#118](https://github.com/codekiln/logseq-gardener/issues/118), [#120](https://github.com/codekiln/logseq-gardener/issues/120), [#119](https://github.com/codekiln/logseq-gardener/issues/119)) give them a destination.
- A left sidebar lists All pages, then a FAVORITES section built from `:favorites` in `logseq/config.edn`, in config order, each favorite resolved through titles and aliases to a published page and shown with the page's original title. The current page is highlighted. Favorites that name no published page are left out and reported as diagnostics. Journals, Flashcards and Graph view join the list when [#110](https://github.com/codekiln/logseq-gardener/issues/110), [#123](https://github.com/codekiln/logseq-gardener/issues/123) and [#120](https://github.com/codekiln/logseq-gardener/issues/120) land, each hidden when its `:feature/enable-*` key is `false`.
- The left sidebar starts closed, opens from the header button or `t l`, and keeps its state across pages and reloads. At 640px and wider it is 246px wide and pushes the main column; below 640px it is a 74vw overlay with a shade that closes it.
- The More menu holds Toggle theme, which switches between light and dark as Logseq's does, and Use system theme, shown while a light or dark choice is saved. View as slides joins it with [#121](https://github.com/codekiln/logseq-gardener/issues/121). The Appearance selector leaves the default page; host pages can still insert one.
- Default colours, body type and widths follow the live Logseq build: the indigo light and dark tokens, a 810px content column (1280px in wide mode, `t w`), 16px side padding below 640px and 32px above.
- `t l`, `t t`, `t w`, `g h` and `g a` work outside text fields.
- The shell's behaviour ships as one shared script file that every page references, so its size is paid once per visit. The shell's styles and the theme snippet that runs before paint stay inline.
- The SDK gains a public operation that reads the shell settings (`:favorites` and the `:feature/enable-*` keys) and resolves favorites to published pages, for the CLI, the language server and the publisher alike.

## Capabilities

### New Capabilities

- `published-app-shell`: the header, left sidebar, favorites, More menu, layout widths, keyboard shortcuts and phone-width behaviour of every generated page, plus the SDK operation that supplies favorites and feature settings.

### Modified Capabilities

- `published-system-themes`: appearance moves from an on-page Appearance selector to Toggle theme and Use system theme in the More menu and `t t`, with Logseq's indigo palette as the default colours. System stays the starting choice, and host-inserted selectors keep working.

## Impact

- SDK: a new garden-settings module for `:favorites` and the `:feature/enable-*` keys, built on the existing EDN reader in `crates/logseq-gardener-sdk/src/site/config.rs`, and favorite resolution through the selected title and alias lookup.
- Publisher: `html_document` in `crates/logseq-gardener-sdk/src/site/mod.rs` renders the header, sidebar and More menu; new `shell.js` and `shell.css`; `theme.js` and `theme.css` change.
- Output: one new shared file, `lsg-shell.js`, at the site root.
- Parity harness: the fixture garden gains favorites, `scripts/parity/budgets.toml` gains a shell-script budget, and `parity:screenshots` gains a 390×844 capture.
- Gitpa, which inserts its own navigation and calls `window.logseqGardenerInitTheme`, keeps working; its own selector stays supported.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
