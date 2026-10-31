## Context

`lsg publish` selects pages in `site/mod.rs`: it applies namespace selection, withholds `public:: false` pages and, since [PR #273](https://github.com/codekiln/logseq-gardener/pull/273), pages not marked `public:: true` in a garden that leaves `:publishing/all-pages-public?` off, assigns routes, collects aliases, then builds the reference index, query results and search records from the selected source documents before it renders each page. The [runtime plugin host](../archive/2026-10-31-site-plugin-host/design.md) added plugin manifests, `lsg-site.json` and per-page metadata, and listed build-time contributions as the next change.

Gitpa still changes `lsg`'s output in two places that a build step can replace ([`scripts/build_site.py`](https://github.com/codekiln/gitpa/blob/main/scripts/build_site.py)):

- `install_homepage_query` finds a placeholder bullet in `index.html` and the homepage, and swaps it for evaluated episode rows linked through routes read from `pages.html`. The placeholder carries `search-exclude:: true` so the rows stay out of search.
- `install_episode_player` writes each episode's compiled player data into `</head>`.

Gitpa's [plugin architecture](https://github.com/codekiln/gitpa/blob/main/docs/plugin-architecture.md) asks for a read-only projection of selected originals after alias and privacy resolution, structured contributions addressed by public page and block ids, links resolved through `lsg`'s route table, derived rows that create no backlinks or search records, and a Rust boundary first with a versioned process protocol for TypeScript tools. Its [`PublicationView`](https://github.com/codekiln/gitpa/blob/main/contracts/gardener-plugin.d.ts) type has `apiVersion`, `revision`, `pages` (`id`, `title`, `url`) and `blocks` (`id`, `pageId`, `text`, `children`).

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) requires graph answers to come from the SDK, the publisher to render only, and every proposal to state its budgets with committed benchmark rows.

## Goals / Non-Goals

**Goals:**

- One SDK operation that produces the selected publication view, shared by the publisher and other callers.
- Build plugins written in Rust (in process) or any language (one process per build), receiving the same view and returning the same contribution types.
- Derived rows and contributed metadata that reach readers without changing references, search, the graph or embeds.
- Every contribution checked before any output file is written.
- No extra build work without build plugins, and measured costs with them on the whole encode garden in CI.

**Non-Goals:**

- Readable page routes supplied by a plugin, replacing Gitpa's `package_pages`.
- Contributed files: build plugins adding their own images, data files or stylesheets to the site.
- An `lsg` command that prints the publication view, for plugin authors testing outside a build.
- Incremental rebuilds that rerun build plugins only for changed pages.

## Decisions

### The view follows Gitpa's `PublicationView`, with two additions

The view handed to a plugin is:

```json
{
  "apiVersion": 1,
  "revision": "5b0c2a…",
  "pages": [
    {"id": "episode/1", "title": "Episode/1", "url": "p-0e27….html", "kind": "page", "properties": {"recording": "https://example.com/1.mp3"}}
  ],
  "blocks": [
    {"id": "1", "pageId": "episode/1", "text": "Intro", "children": ["1.1"]},
    {"id": "6a1f…-uuid", "pageId": "episode/1", "text": "Chapter one", "children": []}
  ]
}
```

`pages` holds every published page that has an id in `lsg-site.json` (`page`, `journal` and `namespace` kinds), sorted by id. `kind` and `properties` are additions to Gitpa's type. `properties` holds only the page properties the plugin names in `pageProperties` and the page shows readers, the same rule as runtime metadata. `blocks` lists each page's blocks in document order, page by page; a page's top-level blocks are its blocks that no other block lists as a child. `blocks` is empty unless the plugin asks for blocks.

### Block ids are page-scoped

A block's `id` is its `id::` UUID when the block has one that no other published block shares, otherwise its position path such as `3.1.2`. The pair `(pageId, id)` identifies a block. This matches `outline::BlockId` and the anchor rules for duplicate UUIDs.

### Block text is what the reader sees of the block's own content

`text` is the block's authored Markdown with its property lines removed and without its children. References to unpublished pages stay as written, as they do on the rendered page.

### The view's revision describes the view

`revision` is a digest of the view's content and the `lsg` version. It differs from `lsg-site.json`'s `revision`, which covers the generated files and is known only after contributions are applied.

### Two contribution kinds

```json
{
  "apiVersion": 1,
  "contributions": [
    {"kind": "rows", "pageId": "ghost in the patch", "blockId": "4",
     "rows": [{"content": [{"page": "episode/1"}, {"text": " — 42 min"}], "children": []}]},
    {"kind": "metadata", "pageId": "episode/1", "values": {"chapters": [{"start": 0, "title": "Intro"}]}}
  ]
}
```

- `rows` adds derived rows after the target block's own children, or at the end of the page outline before the reference sections when `blockId` is absent. A row's `content` is a list of inline nodes: `{"text"}` (plain text, escaped), `{"page", "label"?}`, `{"page", "block", "label"?}` and `{"url", "label"}` for `http`, `https` and `mailto` URLs. A row may have child rows.
- `metadata` adds values to the plugin's entry in a page's `data-lsg-page` metadata, beside the values of its `pageProperties`.

Plugins see the same view and none of each other's contributions. Contributions are applied in plugin id order, then in the order each plugin returned them.

### Derived rows render as outline bullets and stay out of indexes

The publisher renders rows as outline bullets in a `<ul class="lsg-derived" data-lsg-plugin="<id>">`. A row's bullet is a `span.lsg-dot` styled like the outline's bullet by a small stylesheet that only pages with rows carry, so other pages keep their bytes and the outline's zoom ignores derived rows. Page and URL links are plain links. Rows carry no block anchor or `id`. On a namespace page, which has no outline, page-end rows follow the page's notice. Page and block links resolve through the publisher's route table to the target page's URL; a block link adds the block's anchor (`#block-<uuid>`, or the source anchor the publisher adds for a path id, as it does for referenced blocks).

The reference index, search records, the graph, unlinked references, All pages counts and embeds are built from source documents before rows exist, so derived rows never reach them. Tests compare those outputs with and without a plugin that contributes rows.

### The SDK holds the view, validation and links; the publisher renders

- `publication` (SDK): selects pages, withholds private pages and applies the all-pages-public rule from `publishing::PublicPages`, assigns routes (including the generated namespace pages) and collects aliases, moved from `site/mod.rs`; builds the view; filters page properties per plugin. The publisher passes each page's visible properties, the rows its property renderer shows readers, into the view.
- `contributions` (SDK): contribution types, the `BuildPlugin` trait, validation against the view, and resolution of row links to routes and anchors.
- `plugins` (SDK): reads a manifest's `build` section and runs a process plugin as a `BuildPlugin`.
- `site/` (publisher): renders resolved rows and writes merged metadata.

An editor integration or another tool can build the same view and check the same contributions without the publisher.

### One manifest describes both halves of a plugin

The manifest gains an optional `build` object:

```json
{
  "id": "episode-list",
  "apiVersion": 1,
  "version": "1.0.0",
  "pageProperties": ["recording"],
  "build": {"command": ["node", "build.mjs"], "blocks": false, "timeoutSeconds": 60}
}
```

`module` becomes optional when `build` is present, and `activation` is present exactly when `module` is; a manifest with neither `module` nor `build` is an error. Metadata contributions require a `module`, since only a browser module reads them. A build in which no plugin has a `module` adds no host to its pages, and `lsg-site.json` lists only plugins with a `module`.

### A process plugin runs once per build

`lsg` starts `command` once per build, in the manifest's folder, with no shell. A first element containing `/` resolves against the manifest's folder (a leading `./` is allowed); otherwise it is looked up on `PATH`. `lsg` streams `{"apiVersion": 1, "plugin": {"id", "version"}, "view": …}` to standard input, closes it, and reads one JSON document from standard output. Standard error passes through to `lsg`'s standard error. The build fails when the process exits non-zero, writes malformed JSON, answers with another `apiVersion`, writes more than 64 MB, or runs longer than `timeoutSeconds` (default 60, at most 600), and the process is killed on timeout.

### The Rust hook is a trait

```rust
pub trait BuildPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn page_properties(&self) -> &[String] { &[] }
    fn wants_blocks(&self) -> bool { false }
    fn contribute(&self, view: &PluginView<'_>) -> Result<Vec<Contribution>, String>;
}
```

`SiteOptions::build_plugins` holds in-process plugins. Their ids share one namespace with manifest ids. A process plugin is an SDK implementation of the same trait, so the publisher handles both alike.

### Plugins run while indexes build

The view is built right after selection. Each plugin runs on its own thread while the reference index, graph layout, queries and search records are built, and the publisher joins them before rendering the first page. All contributions are validated before any file is written; on a failure the build stops with the plugin id, the contribution's position and the problem.

### Without build plugins nothing runs

When no plugin has a `build` section and `SiteOptions::build_plugins` is empty, the publisher builds no view, starts no thread or process and writes the same files as before this change.

## Risks / Trade-offs

- [Moving selection out of `site/mod.rs` could change output] → the move lands first, on its own, and a whole-encode-garden build must give the same `lsg-site.json` revision as `main`.
- [A view with blocks is large on the whole encode garden (about 125,000 bullets in 43 MB of Markdown)] → blocks are opt-in; the view borrows from the loaded garden; process plugins receive it as a stream; the budgets below cap time and memory.
- [Path ids change when a page is edited] → they are stable within one build, which is the only time a plugin uses them; UUIDs stay stable across builds.
- [Build plugins are trusted code with the publisher's file access] → `docs/publishing.md` states that the view limits what `lsg` hands a plugin and does not isolate the plugin's process. Gitpa's source preparation remains its earlier trust boundary.
- [Gitpa's `PublicationView` type has no `kind`, `properties` or contribution types yet] → the PR lists the additions for Gitpa issue #93 to adopt; the extra fields are optional for a TypeScript reader of the current type.

## Performance budgets

Measured on a pinned commit of the encode garden by a new CI job, against a build of the same binary without plugins. "Per page" divides the added time by the published page count.

| Measure | Budget |
| --- | --- |
| Build work without build plugins | none: no view, thread or process; output identical to `main` |
| Added build time, in-process no-op plugin with blocks | ≤ 0.05 ms per page |
| Added build time, process no-op plugin, pages only | ≤ 0.1 ms per page |
| Added build time, process no-op plugin with blocks | ≤ 0.3 ms per page |
| Peak memory of the `lsg` process with the process plugin and blocks | ≤ 15% above the build without plugins |
| Output of a build whose plugin contributes nothing | identical to the build without plugins |
| First paint of pages without contributions | unchanged |
| Derived rows on one page | ≤ 30 KB gzip, and the page within `[desktop]` and `[phone]` |

The no-op plugins are fixtures: a Rust example that registers an in-process plugin, and a Node script for the process protocol. `parity:budgets` in `mise run ci` checks the fixture-garden rows (derived row size, first paint, unchanged search and reference files).

## Resolved Questions

### 1 - Which fields does the view have, and how does it relate to Gitpa's `PublicationView`?

Decided by the agent without codekiln: Gitpa's fields (`apiVersion`, `revision`, `pages` with `id`, `title`, `url`, and `blocks` with `id`, `pageId`, `text`, `children`), plus `kind` and `properties` on pages. Following Gitpa's type avoids a translation layer; `kind` tells journals from pages, and `properties` lets a plugin find episodes without seeing every property.

### 2 - What identifies a block?

Decided by the agent without codekiln: the pair of page id and block id, where the block id is the block's UUID when it is unique among published blocks and its position path otherwise. Gitpa's type does not say whether ids are unique across pages; this rule reuses `outline::BlockId` and the existing anchor rules for duplicate UUIDs.

### 3 - What text does a block carry?

Decided by the agent without codekiln: the block's authored Markdown without property lines or children. Removing every property line keeps `id::`, hidden properties and private values out of the view with one rule; visible block properties can be added later under a declared list, like page properties.

### 4 - Which page properties does a plugin see?

Decided by the agent without codekiln: the ones it names in `pageProperties` that the page shows readers, the rule runtime metadata already uses. A plugin then sees no more at build time than in the browser.

### 5 - What is the view's `revision`?

Decided by the agent without codekiln: a digest of the view and the `lsg` version. The site revision depends on files that contributions change, so it cannot be known when the view is handed out. This differs from what Gitpa's type comment implies, and the PR notes it.

### 6 - What can a plugin contribute in API version 1?

Decided by the agent without codekiln: derived rows and per-page metadata. These replace Gitpa's two remaining output rewrites (the homepage placeholder and the `</head>` player data); routes and files are listed as non-goals.

### 7 - How do contributions address targets and links?

Decided by the agent without codekiln: by view ids only, with no alias lookup. The view already resolved aliases, and ids from the view are the only names a plugin can be sure are published.

### 8 - How do derived rows stay out of backlinks, search and the graph?

Decided by the agent without codekiln: they are added at render time, after every index is built from source documents, and carry no block anchor. Gitpa's current workaround (a `search-exclude::` placeholder) is no longer needed.

### 9 - Is row content Markdown?

Decided by the agent without codekiln: no, a small list of inline nodes with plain text. Markdown would send derived text through the parser that extracts references, and plain nodes leave no route for injected HTML.

### 10 - Does a build plugin need its own manifest or option?

Decided by the agent without codekiln: no, an optional `build` section in the existing manifest, passed with `--plugin`. Metadata contributions belong to the same plugin id as the browser module that reads them, and one manifest keeps that pairing.

### 11 - What transport does the process protocol use?

Decided by the agent without codekiln: one process per plugin per build, the view streamed as one JSON document on standard input and the contributions as one JSON document on standard output. This meets issue #195's requirement of no per-page round trip, needs no library in the plugin's language, and matches Gitpa's `PublicationView` as a single value.

### 12 - Do plugins receive blocks by default?

Decided by the agent without codekiln: no; `build.blocks` (manifest) or `wants_blocks` (Rust) opts in. Gitpa's homepage list needs only pages and properties, and the blocks of the encode garden are most of the view's size.

### 13 - When do plugins run?

Decided by the agent without codekiln: on their own threads as soon as the view exists, while indexes build, joined before the first page renders. Plugin time then overlaps index work that the build does anyway.

### 14 - What happens when a plugin fails or returns a bad contribution?

Decided by the agent without codekiln: the build stops before writing any file, naming the plugin and the problem. Plugin manifests already fail this way, and a partial site with missing rows would be published silently.

### 15 - How large may contributed metadata be?

Decided by the agent without codekiln: at most 64 KB of JSON per page per plugin, and a key may not repeat one of the plugin's `pageProperties`. The cap protects first paint; the page budgets still apply beneath it.

### 16 - How are the budgets measured on the whole encode garden in CI?

Decided by the agent without codekiln: a new CI job checks out a pinned commit of the public encode garden, builds it without plugins and with each no-op fixture plugin, and checks the `[build_plugins]` rows. Today's whole-garden numbers come from local `parity:perf` runs; issue #106 asks for committed rows in CI.

### 17 - How does the moved selection keep PR #273's all-pages-public rule?

Decided by the agent without codekiln: `publication::select` contains the selection loop from `site/mod.rs` as PR #273 left it, including `PublicPages` and its withheld-page diagnostic, so the view and the site follow the same rule. The generated namespace pages moved with it, because their routes are part of route assignment and the view lists them. The proposal predates PR #273; a scenario in the `publication-view` spec now covers a garden that publishes only marked pages.

### 18 - Where does the rule for which page properties readers see live?

Decided by the agent without codekiln: in the publisher's property renderer, as before; `PublicationView::build` takes each page's visible properties as an input. Whether a value is shown depends on how it renders (a local asset path is not shown, a URL is), and moving that into the SDK would move property rendering with it. The view and the runtime metadata still read the same list.

### 19 - Can a manifest with `build` and `module` leave out `activation`?

Decided by the agent without codekiln: no; `activation` is present exactly when `module` is. A default activation only for plugins with a build section would make the same manifest field mean different things.

### 20 - How do derived rows look and behave next to authored bullets?

Decided by the agent without codekiln: they use a separate bullet element (`span.lsg-dot`) styled like the outline's bullet, with the stylesheet added only to pages that show rows. The outline's bullet click zooms into a block by its position; derived rows are not blocks, and changing the outline script or shared styles would change every page of every build. Page links in rows are plain links without the page-reference brackets, since a derived row is not a reference; a block link without a label shows the target page's title, because the block's text is Markdown that rows never parse.

### 21 - What may an in-process Rust plugin contribute?

Decided by the agent without codekiln: rows only. Metadata needs a browser module, and only a manifest names one; a Rust plugin's id may not repeat a manifest's id.

### 22 - How large are "64 KB" and "64 MB"?

Decided by the agent without codekiln: 64,000 and 64,000,000 bytes, following `budgets.toml`, where KB is 1,000 and MB is 1,000,000.

### 23 - What does a timeout stop?

Decided by the agent without codekiln: the program `lsg` started. A program that starts its own processes must stop them itself; `lsg` does not track process groups, which differ between platforms.

### 24 - How does the whole-garden CI job get the garden?

Decided by the agent without codekiln: a shallow checkout of the public encode garden at commit `5095bdf6`. Without history, Created and Updated dates in All pages fall back as they do for any garden outside git; the job compares builds of the same binary, so that affects every variant alike. Each variant is built three times, interleaved, and the fastest build counts, to keep runner noise below the budgets.

## Open Questions

None.
