# Generate a local HTML garden

Generate standalone HTML pages from selected Logseq namespaces with `lsg publish`, using the shared Rust SDK. The output includes an index, relative page links, and supported referenced assets. Open `index.html` in a browser to read the garden.

Each page ends with an expanded **Hierarchy** section. Its flat list starts with the current complete namespace path, then shows every published descendant path at all levels. Each available slash-separated segment links to its cumulative namespace destination. A leaf still includes its own full path, so its ancestors remain reachable from the bottom of the page. The native Hierarchy disclosure collapses or expands the whole list and supports keyboard operation.

For example, `A/B/C` lists `A/B/C`, `A/B/C/D`, `A/B/C/E`, and `A/B/C/E/F` when those destinations are available. Every row displays the full path, with separate links for `A`, `A/B`, and the remaining prefixes. Missing intermediate namespaces receive their own navigation pages, so a selected `Instruments/Synth/Preset` can be reached through `Instruments` and `Instruments/Synth` without creating those Markdown files.

[All pages](#all-pages) lists these namespace destinations with their full logical names. The reported `pages` count continues to count selected source documents. Generated namespace pages contain navigation only: they contribute no source aliases, blocks, embeds, or homepage targets.

For a complete checkout, build, and browser walkthrough, use the [weekend trial](weekend-trial.md).

## Try the public garden

From this repository's publishing worktree, run:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
mise exec -- cargo run --bin lsg -- publish \
  --graph "$garden_root" --output /private/tmp/garden-site-preview \
  --filename-format triple-lowbar \
  --include My/AI --exclude My/AI/Agent
```

Open `/private/tmp/garden-site-preview/index.html`. The command selects `My/AI` and its descendants and excludes `My/AI/Agent` and its descendants. Repeat `--include` and `--exclude` for additional roots. An empty include list produces an empty index. Selection is case-sensitive, and exclusions always win.

Choose a fresh output directory with an existing parent outside the source garden on each run. Existing output is refused, which also prevents excluded files from a previous run remaining in the site. The publisher leaves source files unchanged. A filesystem write failure may leave partial output in the newly created directory; use a fresh destination after resolving the error.

Use `triple-lowbar` when `logseq/config.edn` sets `:file/name-format :triple-lowbar`; use `legacy` for the default filename format. The command reports publication counts and the index path on standard output and unsupported constructs on standard error. Add `--format json` for a versioned result with `pages`, `journals`, `assets`, `withheld_pages`, `skipped_journals`, `diagnostic_count`, `output_directory`, and `index_path`. Completed generation with diagnostics exits zero; fatal generation failure exits one with empty stdout, and invalid arguments or selectors exit two. Run `lsg publish help` for offline command documentation. To keep the diagnostic report, append `2> /private/tmp/garden-site-report.txt`.

## Visibility and references

A page without a `public` property is published when its title matches the namespace policy. The publisher withholds an entire page containing any parsed `public:: false` property, including a private child in an outline. `public:: true` preserves the namespace policy. Journals are counted and skipped pending configured date naming. Deeply nested content beyond the renderer's visibility limit also withholds its page.

Hierarchy navigation uses only selected visible original titles. An ordinary unselected ancestor may have a navigation-only page, but its body, aliases, properties, and assets stay outside the publication. A stored private or visibility-withheld ancestor receives no generated destination, even when it is outside the include roots. Its segment remains plain text in each visible full-path row while available prefixes retain their links. A generated prefix that conflicts with a selected alias is also left unlinked. Aliases retain their existing source lookup behavior and do not create namespace branches.

Page references can use a selected page's title or leading aliases. When matching names, the publisher ignores letter case, surrounding whitespace, and a slash at either end. It also normalizes equivalent Unicode spellings of accented letters using Normalization Form C (NFC). Duplicate selected titles fail generation before output is created. Alias ambiguity keeps a visible fallback and local diagnostic. Missing and excluded destinations retain labels from the selected source. Markdown links to page filenames require later lookup work.

Competing visible claims, including alias/title collisions, keep the source label and receive a local ambiguity diagnostic. Claims from withheld pages never enter lookup. Only the last leading `alias` property is used; comma-separated plain names and direct page references are supported. Quoted or wholly literal values retain their literal spelling; nested properties and `aliases` metadata do not supply page aliases. Routes and namespace selection retain the original logical title.

References to unique explicit UUID IDs on published Markdown outline items link to the target note. The default link label is a short plain-text excerpt of the selected target; an explicit source label takes precedence. Ordinary block references provide navigation. Page and block embed macros insert selected visible content, including block descendants, using the embedded document’s own asset paths. Expansion stops at cycles, an active expansion chain of 16, 128 expansions, or 10,000 embedded AST nodes per output document; repeated copies omit original IDs so block links continue to open the original page. Missing, excluded, private, duplicate, invalid, and unsupported targets remain literal with diagnostics. IDs in root page properties are not outline targets. Unavailable or ambiguous embeds and unsupported queries retain their literal fallbacks.

## Rendering and assets

The publisher renders outlines, headings, paragraphs, emphasis, literal code, quotes, and tables. Raw HTML and Hiccup appear as escaped literal content. Unsupported syntax receives a visible fallback and a local diagnostic. Ordinary HTTP, HTTPS, and mailto anchors remain links. Email autolinks such as `<reader@example.org>` become mailto links; addresses in literal code stay literal. Email addresses inside another link’s label remain plain label text. Supported HTTPS raster images render directly. The browser loads remote media when the site is viewed; generation performs no remote fetch. HTTP media and unsupported remote types receive a fallback.

Local asset URLs resolve relative to their source document and must point beneath the graph's `assets/` directory. The publisher copies only assets requested by selected rendered content. It rejects symlinks, traversal outside assets, absolute URLs, query/fragment suffixes, and unsupported file types. Supported types are PNG, JPEG, GIF, WebP, AVIF, ICO, PDF, MP3, MP4, OGG, WAV, and WebM. Raster image syntax renders an image. MP3, WAV, and OGG image syntax renders labelled audio controls and a link to the recording. Local recording links request a download. Hosted recording links may open the recording in the browser; download behavior depends on the source host and browser. Browser playback depends on the recording codec. PDF and video image syntax renders a link. Ordinary Markdown links remain links. HTTPS media uses a supported path extension before any query or fragment; URL credentials, controls, backslashes, and missing authorities receive fallbacks.

Pages have deterministic fixed-length routes derived from their titles; the publisher checks for route collisions. Each page contains its own styling and uses relative routes, so it can be read from disk or served by an ordinary static file server.

## Try a podcast selection

Publish retained GitP sessions directly from the public garden:

```sh
mise exec -- cargo run --bin lsg -- publish \
  --graph "$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)" \
  --output /private/tmp/gitp-session-preview \
  --filename-format triple-lowbar \
  --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu
```

Open the generated index, then `GitP/A/Session/24/11/19-Tue`. Its artwork and recording reference HTTPS media; viewing artwork and listening require network access. The exclude selector removes the September 24 session and its asset namespace. Retained production notes remain visible unless their page is withheld; finer private-outline selection and homepage query evaluation remain future work.

## Evidence and next work

[The CLI demonstration record](../openspec/changes/archive/2026-10-08-cli-static-publishing/verification.md) records browser inspection, source preservation, and the fixture checks against excluded page, block, and asset content entering output. [Issue #20 — Publish through lsg](https://github.com/codekiln/logseq-gardener/issues/20) tracks the command under [Issue #11 — Selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11).

The next product work is to improve reference resolution and precise private-block handling. Use the local diagnostic report to choose the syntax and reference gaps that matter for your garden.

## Customize appearance

Supply a local CSS file to style the index and every selected page:

```sh
lsg publish --graph /path/to/garden --output /private/tmp/styled-site --filename-format triple-lowbar --include GitP/A/Session --stylesheet /path/to/theme.css
```

The publisher copies the file unchanged to `custom.css` beside the HTML files and loads it after the built-in styles. Standard CSS specificity and cascade rules apply. Omitting `--stylesheet` keeps the built-in presentation. The stylesheet does not contribute to the reported graph asset count.

The path must name a readable regular file with a `.css` extension (case-insensitive) and have no symlink components. Use the physical path if a parent directory is a symlink, such as `/private/tmp` on macOS. Invalid stylesheet input fails before the output directory is created. Choose CSS you trust: the browser can request resources referenced by `url()` and `@import`. Relative resource URLs resolve from the copied stylesheet in the output root. Only the specified CSS file is copied; arrange its other resources separately.

SDK callers can use `site::publish_site_with_options` with `SiteOptions { stylesheet: Some(path) }`. The existing `site::publish_site` function continues to use built-in presentation. The `publish_site` SDK example accepts the same optional `--stylesheet PATH` flag.

## Publish journals

Add `--journals` to publish every file under `journals/` as a page. Each journal is titled with `:journal/page-title-format` from `logseq/config.edn` (Logseq's default is `MMM do, yyyy`), so the encode garden's `journals/2026_10_23.md` is `2026-10-23 Fri`. A journal marked `public:: false` is withheld like a page, and a journals file whose name is not a date is skipped with a diagnostic. Links to a journal resolve in the configured format and in `MMM do, yyyy`, `yyyy-MM-dd` and `yyyy_MM_dd`, so `[[Feb 1st, 2025]]` opens `2025-02-01 Sat` and keeps its text. Without `--journals`, journals are counted and skipped.

With journals published, `index.html` becomes Logseq's journals feed and the page list moves to `pages.html`. The feed lists journals dated on or before the build date, newest first: three in `index.html`, then seven in each `journals-2.html`, `journals-3.html` and so on. The build date is the UTC date of `SOURCE_DATE_EPOCH` when it is set and of the system clock otherwise; later-dated journals keep their pages but stay out of the feed. As the reader nears the bottom, `lsg-journals.js` appends the next file's journals; without JavaScript, Earlier opens it. Each journal's Linked References show their page count and load from the journal's page when opened. With `--homepage`, the feed starts at `journals.html`. A garden with `:feature/enable-journals? false` gets journal pages and no feed.

## Choose a homepage

Add `--homepage "Ghost in the Patch"` to render a uniquely selected visible page or alias at `index.html`. The original page route remains available and the complete page list moves to `pages.html`. The left sidebar's All pages link opens `pages.html`. An unavailable or ambiguous homepage fails before output is created. Omitting `--homepage` retains the generated page index.

## Collapsed outlines

Each expandable source block has its own 24px disclosure button beside the bullet. Closed controls stay visible; keyboard focus and touch pointers expose controls before hover. The button keeps its owner when moving across nested gutters. Hosts replacing page content call `window.logseqGardenerInitOutline(importedMain)` before route and focus hooks. Repeated initialization preserves controls and collapse state; appended journal outlines initialize automatically.

An outline parent with an attached `collapsed:: true` property starts closed.
Readers can open its summary with a click or the keyboard to reveal the complete
outline. Nested closed parents open independently. False or absent flags keep the
outline expanded, and a leaf stays visible without an empty disclosure.

Formatting and links remain in the summary. When a parent contains a page or block
embed, its text labels the summary and the complete rich content appears inside.
Media, transclusions and publication visibility keep their existing behavior.

Links to blocks inside closed outlines reveal the containing disclosures, including
on the initial page load. Sites that replace document content during navigation can
call `window.logseqGardenerRevealFragment(hash, {scroll: false})` before their own
fragment focus and scrolling. The helper accepts an encoded fragment beginning with
`#`, opens only the target's containing disclosures, and returns the target element
or `null` when unavailable.

## Reference sections

Below the outline, each page shows the sections that apply in Logseq's order: **Pages tagged with**, **Linked References**, **Hierarchy** and **Unlinked References**. The page holds only each section's header; the groups load from fragment files in `_refs/` when a reader opens the section, so a page's first paint does not grow with the number of pages that mention it.

- **Pages tagged with "name"** lists the pages whose `tags::` name the page or one of its aliases, by lower-cased title. Pages that no page tags have no section.
- **Linked References** counts referring blocks: `N Linked References`, or `F of N Linked References` while a filter is active. A reference to `A/B/C` also counts for `A/B` and `A`, as in Logseq, even when `A/B/C` has no page. A block counts once however often it names the page or its aliases. Groups list journals newest first, then other pages by title. A referring block below the top level shows up to three ancestors as a breadcrumb, with `⋯` for older ones; each crumb and each copied bullet opens the block on its own page. Pages with no linked references have no section.
- **Unlinked References** lists blocks that mention the page's title or an alias as plain text, using Logseq's rule: the name is not preceded by a letter, digit, `#` or `[[` and not followed by a letter or digit. Logbooks are ignored. The header shows its count once opened.

The funnel in the Linked References header opens the filter dialog. Click or Enter includes a page, shift-click or Shift+Enter excludes it, and activating it again removes it. A block stays when its path (the block, its ancestors and its page) names every included page and none of the excluded ones. The options and their counts follow the blocks the filter keeps. A page's `filters::` property sets its starting filter, and a reader's changes last for the visit through `sessionStorage`.

When page properties are enabled, property references show their safe relevant fields and link to the original Properties disclosure; otherwise they use a short page-reference link. Only selected visible source content supplies references, filter options and mentions. Pure block UUID references retain block navigation and do not create page references.

The publisher reads these top-level integer settings from `logseq/config.edn`:

- `:ref/linked-references-collapsed-threshold` defaults to `100`. A Linked References section starts closed when its referring-block count reaches this threshold. Set it to `0` to start every section closed.
- `:ref/default-open-blocks-level` defaults to `2`. Copied parents at this depth or deeper start closed. Authored collapsed parents also stay closed.

Missing settings use defaults. Unusable config receives a local diagnostic; reading is bounded and never evaluates EDN or follows config-path symlinks. Without JavaScript, each section offers a link to its first fragment file, which is a readable page linking to the next one.

`lsg-references.js` loads after the page's `load` event. A host that replaces page content can call `window.logseqGardenerInitReferences(root)` (also available as `window.logseqGardenerInitBacklinks`) after inserting the new content. The call is idempotent and separate from `logseqGardenerRevealFragment`; it loads sections as they open and installs the filter dialog without changing navigation or media playback.

## All pages

The page index is Logseq's All pages table, titled `All pages`: a total, a `Search page names` box, a journals toggle, paging, and the columns `Page name`, `Backlinks`, `Created At` and `Updated At`. It opens at `index.html`, or at `pages.html` when a homepage or journals feed takes `index.html`.

- **Backlinks** counts the blocks that refer to the page, including references to its namespace children and blocks on the page itself, as Logseq's column does. The page's Linked References count leaves out its own blocks, so the two can differ.
- **Created At** and **Updated At** are the times of the first and last git commits that touched the page's file, shown in the reader's time zone (UTC without JavaScript). Generated namespace pages have no file and no dates. A garden that is not a git repository gets no date columns, sorts by page name and reports one diagnostic; a shallow clone gets dates from the history it holds and also reports a diagnostic. Reading the history runs alongside rendering.
- The default view hides journals and lists pages newest first, 40 rows at a time. `pages-2.html`, `pages-3.html` and so on hold the rest of that view, so every page is reachable without JavaScript.
- Clicking a column title sorts by that column and flips the direction shared by all columns. Search runs on Enter with Logseq's fuzzy match on page names and shows at most 20 rows; Escape clears it. The journals toggle adds journals. All three load `_pages/list.json`, which holds every entry, the first time a reader uses one of them; paging through the default view never loads it.
- Below 640px wide, only the icon and `Page name` columns show. Pages with an `icon::` property show the icon before the name.

`lsg-all-pages.js` loads only on All pages screens. A host that swaps an All pages screen into a page calls `window.logseqGardenerAllPages.init(root)` after inserting it. Logseq's editing controls (row checkboxes, `Delete`, the whiteboard toggle and the `⋯` menu) are not drawn, because they change nothing that lasts in a published site.

## Namespace destinations and page sequences

Generated namespace pages show a Namespace notice. Their hierarchy entries carry a Namespace badge, their All pages rows do not, and links to these destinations use a dotted underline. Selected source pages keep ordinary page labels, including empty source pages. Namespace status describes the published destination; an unselected source ancestor has the same status as a missing ancestor.

Add leading `prev:: [[Page title]]` or `next:: [[Page title]]` properties to publish Previous and Next navigation after the page content. Each value must be one direct page reference. Titles and unique aliases resolve to the canonical selected source page. Private, excluded, missing, ambiguous, self, and generated-only targets produce no navigation link. The last leading value for each key applies; nested properties and literal code do not supply sequence links. Authors choose the order, and a page can have either neighbor or both.

## Outlines, collapse and block zoom

Page and journal outlines look like Logseq's: small bullets, a guide line down each level of children, Logseq's heading sizes, and task checkboxes with their marker (TODO, DOING and so on). The checkboxes are read-only. DONE and CANCELED text is struck through.

Readers can collapse any block with children with the arrow that appears beside its bullet, or by clicking its guide line; blocks written with `collapsed:: true` start closed and open without JavaScript. Clicking a bullet shows that block on its own at `<page>#zoom/<id>`, with a breadcrumb back to the page and its parent blocks. `<id>` is the block's `id::` UUID when it has one, otherwise its position among its siblings at each level, such as `3.1.2`; the address keeps working when the unchanged garden is published again. Back returns to the page. Collapse and zoom stay in the reader's open tab.

## Header and left sidebar

Every page has Logseq's header: Toggle left sidebar on the left, Home (on every page except `index.html`), Graph, More and, from 640px wide, Toggle right sidebar on the right. The left sidebar lists Journals when a journals feed is published, Graph view, All pages, then the garden's favorites from `:favorites` in `logseq/config.edn`, in config order. Each favorite resolves through page titles and aliases and shows the title or alias as the author spelled it; the current page is highlighted. A favorite that names no published page is left out and reported as a diagnostic.

The sidebar starts closed. The header button or `t l` toggles it, and the state is saved under `logseq-gardener-left-sidebar` in browser localStorage. From 640px wide the open sidebar is 246px and pushes the page right; on narrower screens it opens over the page and closes when the reader taps the shade, follows a link or presses Escape. `t w` switches between the 810px and 1280px page widths, saved under `logseq-gardener-wide`. `g h` opens Home, `g a` opens All pages, `g j` opens the journals feed and `g g` opens the graph view, unless `logseq/config.edn` sets `:shortcuts {:go/graph-view false}`. Shortcuts are ignored while typing in a field.

The header and sidebar behaviour is in `lsg-shell.js` at the site root, which every page loads with `defer`. After the page has loaded it also loads `lsg-outline.js` on every page and `lsg-journals.js` on feed pages. Without JavaScript the sidebar stays closed and its button and More stay hidden.

## Right sidebar

From 640px wide, the Toggle right sidebar button at the right end of the header, or `t r`, opens a column of cards beside the page. Shift-click a page link, tag or breadcrumb to add a card for that page, a bullet to add a card for that block, or a block reference to add a Block references card. The page itself stays where it is. The top bar's Page graph button adds a card that draws the current page's neighbourhood, as described under [Graph view](#graph-view). A page card shows the page without its title or Unlinked References; Linked References load when opened. An empty sidebar shows the garden's `contents` page; `alt+shift+c` adds or removes the Contents card, and the top bar adds Contents and Help cards.

Each card has a header that collapses it, a Card menu (close, collapse and expand this card or the others, Open as page) and a Close button; a middle click on the header also closes it. Drag the column's left edge, or focus it and press Left or Right, to resize it. The cards, their order, collapsed flags, the width and whether the sidebar is open are saved under `logseq-gardener-right-sidebar` in browser localStorage and restored on every page. On a first visit, `:default-home {:sidebar [..]}` in `logseq/config.edn` chooses the starting cards: `"Contents"` or page names, with a diagnostic for a name that matches no published page.

The behaviour is in `lsg-sidebar.js` and `lsg-sidebar.css`, which load the first time a reader uses the sidebar, or after the page loads when the sidebar was left open. Cards fetch the published page files. Below 640px there is no sidebar, and Shift-click follows the link.

## View as slides

On a page or journal page, from 640px wide, More starts with View as slides. It adds a Slides card for the page to the top of the right sidebar: each top-level block is a column, and the block and then each of its descendants is a slide below it. Blocks that show nothing make no slide. The deck takes keyboard focus, so the arrow keys (or `h j k l`) move between slides, `n` and Space step through the whole deck, `g` jumps to a slide number or `column/slide`, `o` or Escape shows every slide, `b` blacks the deck out and `f` toggles fullscreen. A block's `background-image::`, `background-color::`, `transition::` and `data-*` properties apply to its slide, as in Logseq.

`lsg-slides.js` loads with the first Slides card and holds the deck's styles. It builds the deck from the page's own outline and keeps only the current slide and its neighbours in the document, so images on distant slides load when the reader gets near them.

## Graph view

`graph.html` draws every page as a node and every page reference, tag and namespace parent as an edge, as Logseq's graph view does. The header's Graph link and the sidebar's Graph view item open it; they carry Logseq's classes (`cp__header` on the header, `graph-view-nav` on the item), so a stylesheet written to hide Logseq's entry points hides these too. Pages that exist only because a published page names them are nodes that open nothing. Pages that are not published never appear.

Drag to pan, scroll or pinch to zoom, click a node to open its page, and Shift-click or drag a node to select it. The panel at the top right has Logseq's sections: Nodes (the Journals, Orphan pages, Built-in pages and Excluded pages switches, and N hops from the selected nodes), Search (title filters and a list of matching nodes, which is also the keyboard path to a node), Forces (the layout sliders) and Export (as PNG). Its starting values come from `:graph/settings` and `:graph/forcesettings` in `logseq/config.edn`; a reader's changes last for the browser tab's session.

The publisher lays the graph out at build time with Logseq's forces and writes it to `_graph/default.bin`, and the journals graph to `_graph/journals.bin`, both stored gzip-compressed. Only `graph.html` loads them, through `lsg-graph.js`; `lsg-graph-worker.js` runs a force simulation only when a reader changes a force or drags a node. The Page graph card reads `_graph/p/<route hash>.json`, the page's neighbourhood; a page whose neighbourhood file would exceed 100 KB gzip has none, and its card draws the neighbourhood from the global file instead. Through the SDK, `SiteOptions::layout_cache` keeps layouts between builds in a directory of your choice.

## Appearance

Generated sites follow the reader's system appearance by default. Toggle theme in the More menu, or `t t`, switches between light and dark and saves the choice under `logseq-gardener-theme` in browser localStorage for the origin. While a choice is saved, More also offers Use system theme, which removes it. If storage is unavailable, the choice lasts for the current document and its internal navigation. Without JavaScript, shared CSS still follows the system preference.

The shared script applies the initial appearance in the document head and updates it when the system preference changes. Hosts that insert their own native Appearance selector (`<select data-lsg-theme-select>` with `system`, `light` and `dark` options inside a `data-lsg-theme-control` element) call `window.logseqGardenerInitTheme(importedNavigation)` after insertion so the selector reflects the active choice. Repeated initialization is safe and does not replace page content or media players.

The default colours are Logseq's indigo theme. Custom stylesheets can set `--lsg-text`, `--lsg-background`, `--lsg-surface`, `--lsg-sidebar`, `--lsg-header`, `--lsg-link`, `--lsg-border`, `--lsg-quote-border`, `--lsg-warning`, `--lsg-focus`, `--lsg-muted`, `--lsg-title` (page titles), `--lsg-guide` (outline guide lines and closed bullets) and `--lsg-rule` (the rule under h1 and h2 blocks). Put light overrides on `:root` and dark overrides on `:root[data-lsg-theme="dark"]`; an equivalent `@media (prefers-color-scheme: dark)` rule for `:root:not([data-lsg-theme])` supplies the no-JavaScript dark palette. A stylesheet with fixed colors needs matching dark overrides for its own components.


## Page properties

Leading page properties are available through an explicit graph opt-in:

```edn
{:publishing/page-properties? true
 :publishing/page-properties-collapsed? true
 :publishing/page-sequence? true
 :block-hidden-properties #{:private-note :operational-field}}
```

The defaults are false, true and true respectively. Enabled properties appear in a native Properties disclosure above the source body. Set the collapse option to false to start it open. `#page-properties` targets its keyboard-focusable summary; the shared fragment helper reveals the containing disclosure. Generated namespaces have no invented property section.

Rows retain authored order and the last duplicate value. Ordinary scalar text is escaped. Direct page references and comma-separated bare tags link to canonical selected source pages. Standalone public HTTPS values are clickable. Missing, private, ambiguous or generated-only reference targets omit the entire row, including mixed values. Expressions, block references, asset/file paths, absolute local paths, unsafe schemes and unsupported links are omitted. Explicit opt-in treats remaining scalar values as public metadata, so use `:block-hidden-properties` for author-specific operational fields. The setting accepts collections of keyword keys.

Internal exclusions are `id`, `custom-id`, `custom_id`, `title`, `alias`, `aliases`, `public`, `background-color`, `background_color`, `heading`, `collapsed`, `created-at`, `updated-at`, `last-modified-at`, `created_at`, `updated_at`, `last_modified_at`, `query-table`, `query-properties`, `query-sort-by`, `query-sort-desc`, `filters`, `ls-type`, `hl-type`, `hl-page`, `hl-stamp`, `hl-color`, `todo`, `doing`, `now`, `later`, `done`, `file`, `file-path` and `file-name`; prefixes `logseq.`, `logseq-proxy-` and `publishing/` are also excluded. These exclusions and configured hidden keys apply to enabled property references as well as displayed rows.

Authored `prev` and `next` retain the same generic source-page validation as sequence navigation. A valid neighbor shown in Properties is omitted from the bottom Page sequence region. Other valid neighbors remain available there. Disable `:publishing/page-sequence?` to hide that region entirely; configured hidden neighbor keys also remain hidden. Book, preset and other authored chains use the same behavior.

## Dialogue timestamps

An original outline entry such as `- **Speaker** · 00:27:30 — Spoken text` can link its timestamp to the page's recording. Plain and bold/italic speaker labels work with `M:SS` and `H:MM:SS`; the timestamp is an absolute position in the recording. The later components must be two digits below sixty. Dialogue formatting, nested entries and authored disclosures are retained. Written responses, headings, dates, code and authored link labels keep their existing text.

Automatic association requires one distinct supported audio recording rendered from the original page. The recording can follow the transcript. Repeated references to the same recording share its identity; several different recordings leave dialogue timestamps literal. Audio and dialogue copied through transclusions or backlinks do not participate. Existing audio source URLs, controls and preload settings remain in place. The renderer does not measure remote duration or infer trim offsets.

Timestamp controls are ordinary recording links with `#t=` fragments. The shared helper loads only on pages with those controls and seeks the associated original inline audio when activated. Playback begins only from activation; the latest pending position waits for metadata and is clamped to the known duration. Native media controls remain available when playback is blocked. Modified clicks retain ordinary link navigation.

A host with a persistent player can listen for the bubbling, cancelable `lsg:media-seek` event. Its `detail` contains the absolute recording `source`, integer `seconds` and originating `anchor`. Validate those values against the host's current recording, call `preventDefault()` on the custom event, and handle playback through the host player. The shared helper then prevents the original click and performs no inline playback. The helper uses one delegated capture listener per document, so replacement transcript controls work without another setup call. Hosts that do not load the helper can still handle the ordinary recording-fragment links directly.

The dialogue helper loads after the page load event, like the outline helper, and only pages with dialogue controls request it. Until it loads, or if loading fails, timestamp links keep their ordinary recording-fragment navigation. Search-enabled dialogue and journal-feed pages retain the 16 KB initial-script budget. Hosts that repackage routes must rewrite the `href` of the `link[rel="lsg-dialogue"]` helper metadata along with other local resources. The loader captures that resolved URL before navigation.
