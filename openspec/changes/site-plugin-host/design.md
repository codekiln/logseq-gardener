## Context

`lsg` publishes flat HTML files (`index.html`, `p-<digest>.html`, journals) and each page's scripts set themselves up once on load; moving between pages is a full page load. Gitpa needs a recording player that keeps playing while the reader moves between episodes, so it rewrites `</head>` in generated HTML, rebuilds routes from `pages.html`, and runs its own router. [Gitpa PR #94](https://github.com/codekiln/gitpa/pull/94) proposes the TypeScript contract (`RuntimeManifest`, `RuntimeHost`, `PageAdoption`, `PersistentRegion`, `RuntimePlugin`) and the navigation order; it leaves file names, JSON layout and the build-time transport to this repository. This change implements that contract's first slice.

## Goals / Non-Goals

**Goals:**

- A route and asset manifest that callers can read instead of parsing HTML.
- Declared plugin modules and stylesheets, checked at build time, with per-page metadata drawn from visible page properties.
- A browser host that owns same-document navigation, keeps plugin regions mounted, and calls plugins in a fixed order with an abort signal per page.
- Unchanged pages and zero extra requests when no plugin is configured.

**Non-Goals:**

- Build-time presentation contributions: a read-only publication view of pages and blocks that plugins transform into structured content after indexing (Gitpa's `PublicationView`). This is the next change once the runtime host is in use.
- A process protocol so TypeScript build tools can run as build plugins.

## Decisions

### Two JSON files with fixed names

`lsg-site.json` is written by every build: Gitpa reads routes without plugins too, and the file costs readers nothing because no page requests it. The plugin manifest is an input file the caller names with `--plugin`; its fields follow Gitpa's `RuntimeManifest` (`id`, `apiVersion`, `module`, `styles`, `activation`) plus `version` and `pageProperties`. Plugin entries are added to `lsg-site.json` with output-relative URLs, so the host reads a single list.

### Page identity is Logseq's lower-cased page name

Logseq identifies a page by its lower-cased name (`:block/name`), and `lsg`'s page-name operations already compute it. Hashed file names stay a publisher detail; `url` gives the route.

### Metadata comes from named, visible page properties

A plugin lists the page properties it reads. The publisher copies their values only when the page shows them to readers, so a plugin can never see more than a reader can. Values are the raw property text; a plugin validates its own payload.

### The SDK validates manifests; the publisher renders

Manifest parsing and validation, page identity and property visibility are SDK operations the CLI and the language server can call. The publisher copies files, writes metadata and `lsg-site.json`, and ships the host.

### The host loads only with plugins, after first paint

Pages of a site with plugins get one `<script type="module">` that imports `lsg-host-<digest>.mjs` after first paint, matching the search loader. `after-load` plugins load on every page; `on-demand` plugins load on the first page whose metadata names them.

### One navigation owner, one initializer list

With plugins, the host intercepts plain left clicks on same-site page links and history traversal, fetches the page, and swaps `<main>`, the title and the metadata. `lsg`'s page scripts already install their document listeners once and expose a setup function for new content (`logseqGardenerInitOutline`, `logseqGardenerInitReferences` and others); the host keeps the initializer list that calls them with `<main>` and the page signal, and `window.logseqGardenerPage(init)` adds to it. Keeping the list in the host, rather than in `shell.js`, leaves every script of a site without plugins unchanged apart from a setup function added to `lsg-queries.js`, and keeps `lsg-shell.js` and `lsg-outline.js` within their budgets. Without plugins nothing intercepts clicks, and the scripts set up once, as today.

### Order of a navigation

Prepare (fetch, parse, check revision; drop superseded responses), then abort the old signal, then swap, then run core initializers, then notify plugins synchronously, then reveal the fragment and focus. This is the order Gitpa's contract specifies, and it lets a plugin mark up the page before `lsg` scrolls to a block.

### A new build shows a notice

A revision mismatch keeps the current page and offers a link to load the new version, so a playing recording is never cut off by a silent reload.

## Risks / Trade-offs

- [Same-document navigation can leave stale state from `lsg`'s scripts] → each initializer removes its listeners on abort; a test navigates 20 times and checks the listener count and every feature's behavior.
- [Plugins run in the page with full access] → plugins are trusted code chosen by the site author; the publisher limits what data reaches them (visible properties only) but does not sandbox them. `docs/publishing.md` says so.
- [The contract may change once Gitpa adopts it] → `apiVersion` gates changes; this change ships version 1 and a fixture plugin in the repository that exercises every host function.

## Performance budgets

| Measure | Budget |
| --- | --- |
| Added bytes per page, no plugins | 0 |
| Host module | ≤ 4 KB gzip |
| First-paint bytes with one plugin | within `[desktop]` and `[phone]` |
| Navigation after the response arrives, phone profile | ≤ 50 ms |
| Document listeners after 20 navigations | equal to after 1 |

The harness measures these on the whole encode garden with a fixture plugin, as `[plugins]` rows in `scripts/parity/budgets.toml`.

## Resolved Questions

### 1 - What are the manifest files called and shaped like?

Decided by the agent without codekiln: the output file is `lsg-site.json` (`apiVersion`, `revision`, `pages`, `assets`, and `plugins` when present); the input is a plugin manifest file passed with `--plugin`, whose fields follow Gitpa's `RuntimeManifest` plus `version` and `pageProperties`. Fixed names keep the contract discoverable, and reusing Gitpa's field names avoids a translation layer.

### 2 - Is `lsg-site.json` written without plugins?

Decided by the agent without codekiln: yes, always. Gitpa reads routes even before it adopts plugins, and no page requests the file, so readers pay nothing.

### 3 - What identifies a page?

Decided by the agent without codekiln: the lower-cased page name, which is how Logseq identifies pages. File names are content hashes and change with titles.

### 4 - Where does per-page metadata come from?

Decided by the agent without codekiln: from the page properties a plugin names in `pageProperties`, and only those the page shows readers. This lets Gitpa read `recording::` without a general property-publication policy, which belongs with the build-time contribution change.

### 5 - Does `lsg` gain same-document navigation?

Decided by the agent without codekiln: only when a plugin is configured. A playing recording needs the document to stay; Logseq's own published site keeps one document too. Sites without plugins keep full page loads and their current budgets.

### 6 - What happens when a new build is deployed during a visit?

Decided by the agent without codekiln: the current page stays and a notice links to the new version. Gitpa's contract asks for recovery without silently reloading a playing recording.

### 7 - Which build-time transport comes first?

Decided by the agent without codekiln: none in this change. The runtime host is what Gitpa needs to remove its HTML rewriting; the Rust SDK boundary for build-time contributions follows in its own change, as Gitpa's architecture document suggests.
