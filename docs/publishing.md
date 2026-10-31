# Publishing

Generate a static HTML site from selected Logseq Markdown namespaces with `lsg publish`. The command uses the Rust SDK to load the garden, resolve visible references and embeds, render pages, and copy supported referenced assets. The source garden remains unchanged.

## Try a garden

From this repository, publish part of the public encode garden into a fresh output directory:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
preview_root="$(mktemp -d "${TMPDIR:-/tmp}/lsg-publishing.XXXXXX")"
mise exec -- cargo run --bin lsg -- publish \
  --graph "$garden_root" --output "$preview_root/site" \
  --filename-format triple-lowbar \
  --include Logseq/Frontmatter --exclude Logseq/Frontmatter/tags
python3 -m http.server 8000 --bind 127.0.0.1 --directory "$preview_root/site"
```

Open <http://127.0.0.1:8000/> and follow the frontmatter page's selected embeds. The excluded tags explanation stays unavailable. Stop the server with Ctrl-C. Choose another port if needed. The output directory must be absent and outside the source garden; each run creates a fresh destination.

The [local-site guide](local-site.md) describes filename formats, visibility, rendering, assets, optional search and journal publication. The [garden trial](weekend-trial.md) adds workshop navigation and podcast listening checkpoints. Rust callers can use `site::publish_site` or `site::publish_site_with_options` from `logseq-gardener-sdk`; [the SDK example](../crates/logseq-gardener-sdk/examples/publish_site.rs) shows the call.

## Namespace selection

The SDK's `publishing::NamespaceSelection` selects logical page names using included and excluded namespace roots. A root includes itself and descendants separated by `/`: `My/AI` selects `My/AI/Rule`, while `My/AIM` stays outside the selection. Exclusions take precedence, and an empty inclusion list selects nothing.

Matching is exact and case-sensitive. Selectors use logical page titles derived from the configured filename format and leading title properties. Aliases help resolve references; they do not select a page into a namespace. Empty selectors, empty namespace segments, surrounding segment whitespace, and control characters produce constructor errors. Malformed candidate names are unselected. Names are not filesystem paths.

## Publication boundaries

The publisher resolves page titles, aliases, explicit outline references, and page/block embeds against selected visible content. A parsed `public:: false` withholds its entire source page. Unless `logseq/config.edn` sets `:publishing/all-pages-public? true` or `:all-pages-public? true`, a selected page must also be marked `public:: true`, as in Logseq; `publishing::PublicPages` applies this rule, and [Visibility and references](local-site.md#visibility-and-references) describes it. Namespace exclusions remove target pages and their contributions; they do not redact text already written on a retained page. Ambiguous or unavailable targets retain visible fallbacks and local diagnostics.

Visibility also applies to embedded content, referenced assets, reference indexes and optional search data. Mixed-visibility fixtures in [publisher tests](../crates/logseq-gardener-sdk/tests/site.rs) and [embed tests](../crates/logseq-gardener-sdk/tests/site_embeds.rs) check generated files for excluded content. Local diagnostics stay outside the site. Review the report and retained source content before publishing a sensitive garden.

## Site manifest

Every build writes `lsg-site.json` at the output root, so build tools can read routes without parsing HTML. Generated pages never request it.

```json
{
  "apiVersion": 1,
  "revision": "102ef744a7e1b199",
  "pages": [
    {"id": "show/episode 1", "title": "Show/Episode 1", "url": "p-0e2769dd320a1844.html", "kind": "page"},
    {"id": null, "title": "Journals", "url": "index.html", "kind": "home"}
  ],
  "assets": ["assets/a-ffc572edff75a892.mp4"]
}
```

- `id` is the page's lower-cased name, as Logseq identifies pages. Generated lists (All pages, the journals feed) have `null`. A homepage at `index.html` has its page's id.
- `kind` is `page`, `journal`, `namespace` (a namespace page with no file), `home` (`index.html`) or `list`. The graph view is not listed.
- `revision` changes when any generated file changes, except the graph view's layout data.
- `pages` is sorted by `url` and `assets` by path, so identical input produces identical bytes. Withheld and excluded pages, and assets only they use, are absent.

A build with plugins that have a browser module adds `plugins`: each such plugin's `id`, `version`, `module`, `styles` and `activation`, with paths relative to the output root. A plugin that runs only at build time is not listed.

## Plugins

A plugin adds browser behavior to a published site, such as a recording player that keeps playing while the reader moves between pages, and can add content to pages at build time (see [Build plugins](#build-plugins)). Pass its manifest with `--plugin PATH`; repeat the option for more plugins. Rust callers set `SiteOptions::plugins`.

```json
{
  "id": "episode-player",
  "apiVersion": 1,
  "version": "1.0.0",
  "module": "player.mjs",
  "styles": ["player.css"],
  "activation": "after-load",
  "pageProperties": ["recording"]
}
```

| Field | Meaning |
| --- | --- |
| `id` | Lower-case letters, digits and single hyphens; unique in the build. Files go to `plugins/<id>/`. |
| `apiVersion` | The host API version. This build supports `1`. |
| `version` | The plugin's own version, copied to `lsg-site.json`. |
| `module` | An ES module (`.js` or `.mjs`), relative to the manifest's folder. Optional with `build`. |
| `styles` | Optional stylesheets (`.css`), relative to the manifest's folder. The host adds each once. Needs `module`. |
| `activation` | `after-load` imports the module on every page; `on-demand` imports it on the first page whose metadata has an entry for the plugin. Present exactly when `module` is. |
| `pageProperties` | Optional page property names the plugin reads. |
| `build` | Optional command that runs once per build; see [Build plugins](#build-plugins). |

The build checks every manifest before it creates the output directory and stops with the manifest's path and the problem when a field is missing, unknown or malformed, `apiVersion` is unsupported, two manifests share an `id`, a manifest has neither `module` nor `build`, or a module, stylesheet or build program path is absolute, contains `..`, passes through a symlink, is not a regular file or collides with another file of the plugin. Plugins are ordered by `id`, so argument order does not change the output. The SDK's `plugins::read_manifests` runs the same checks for other callers. [`tests/fixtures/plugins/recorder`](../tests/fixtures/plugins/recorder) is a complete example.

### What a page carries

With at least one plugin that has a `module`, every page except the graph view gets two tags before `</head>`:

- `<script type="application/json" data-lsg-page>` with the page's `id`, `title`, `url`, the site `revision`, and `plugins`: for each plugin, the values of its `pageProperties` that the page shows readers, as authored text, and the values its build command contributed for the page. A property hidden by `:block-hidden-properties`, a page without page properties enabled (`:publishing/page-properties?`) and a withheld page give the plugin nothing; a page with none of a plugin's properties or contributions has no entry for it.
- A module script that imports `lsg-host-<digest>.mjs` after first paint.

Without plugins, or with build-only plugins, pages carry neither tag and no host or plugin code is requested.

### Host API

The module's default export is `{ id, apiVersion: 1, setup(host) }`, with `id` matching the manifest. The host checks the export, then calls `setup` once per document; `setup` may return a cleanup function. A mismatched export or a failing import is logged to the console and skipped; the page keeps working.

`host` provides:

- `apiVersion`: `1`.
- `document` and `baseURL` (the site root).
- `onPageAdopted(listener)`: calls `listener` now for the current page and after every later navigation, with `{ page: { id, title, url }, main, revision, metadata, signal }`. `metadata` is this plugin's entry from the page metadata, or `{}`. `signal` is aborted before the page is replaced; pass it to `addEventListener` for page-scoped listeners. Returns a function that removes the listener.
- `persistentRegion(localId)`: returns `{ element, dispose }`, where `element` is a `<div data-lsg-plugin="<id>" data-lsg-region="<localId>">` at the end of `<body>`, outside `<main>`, kept across navigations.

If `setup` throws, the host removes that plugin's regions and listeners, logs the error, and the other plugins keep working. When the page unloads, the host aborts the page signal, calls every cleanup once and removes the plugins' regions.

### Navigation

With plugins, the host owns navigation. A plain left click on a link to another page of the site, and Back and Forward between such pages, adopt the next page in the same document:

1. Fetch and parse the page. A response overtaken by a newer navigation is dropped.
2. Compare its `revision` with the current page's. When they differ, the site was rebuilt: the current page stays, and a notice offers a link that loads the new version normally, so a playing recording is not cut off.
3. Abort the current page's signal.
4. Replace `<main>`, the title and the page metadata, update the left sidebar and the header's page-specific buttons, add any styles and scripts the new page needs, and push a history entry.
5. Run lsg's page initializers (outline arrows and zoom, references, All pages, journals, queries, theme controls) on the new `<main>`. Other scripts register more with `window.logseqGardenerPage(init)`; `init(main, signal)` runs now and after every navigation.
6. Notify plugins.
7. Reveal the fragment, as a full page load would, and move focus to it.

A fetch failure, or a page without metadata such as the graph view, loads normally. Modified clicks, links with a `target` or `download`, and links within the same page keep their usual behavior.

### Build plugins

A build plugin adds content to pages while `lsg` builds the site, such as a list of episodes under a heading of the homepage, or the chapter data an episode player reads. It replaces editing the generated HTML after the build.

Add a `build` section to the manifest:

```json
{
  "id": "episode-list",
  "apiVersion": 1,
  "version": "1.0.0",
  "pageProperties": ["recording"],
  "build": {"command": ["node", "build.mjs"], "blocks": false, "timeoutSeconds": 60}
}
```

| Field | Meaning |
| --- | --- |
| `command` | The program and its arguments. `lsg` runs it once per build, without a shell, in the manifest's folder. A program name containing `/` is a file in that folder; otherwise it is looked up on `PATH`. |
| `blocks` | `true` to receive every block of every page. Default `false`: pages only. |
| `timeoutSeconds` | How long the program may run, 1 to 600. Default 60. |

A manifest with `build` may leave out `module` and `activation`; it then runs only at build time and adds nothing to pages beyond its contributions.

#### The view

`lsg` writes one JSON document to the program's standard input and closes it:

```json
{
  "apiVersion": 1,
  "plugin": {"id": "episode-list", "version": "1.0.0"},
  "view": {
    "apiVersion": 1,
    "revision": "5b0c2a4f1e9d8c7b",
    "pages": [
      {"id": "show/episode 1", "title": "Show/Episode 1", "url": "p-0e2769dd320a1844.html", "kind": "page", "properties": {"recording": "https://example.com/1.mp3"}}
    ],
    "blocks": [
      {"id": "1", "pageId": "show/episode 1", "text": "Intro", "children": ["1.1"]},
      {"id": "1.1", "pageId": "show/episode 1", "text": "Detail", "children": []}
    ]
  }
}
```

- `pages` holds every published page, journal and namespace page, sorted by `id`, after namespace selection, `public::` rules and alias resolution. Withheld and excluded pages are absent, and no field holds a source path.
- `properties` holds the values of the plugin's `pageProperties` that the page shows readers, the same values the browser module receives.
- `blocks` lists each page's blocks in document order, page by page, and is empty unless the plugin asked for blocks. A block's `id` is its `id::` UUID when no other published block has the same UUID, and its position path such as `3.1.2` otherwise; the pair of `pageId` and `id` names a block. `text` is the block's Markdown without its property lines and without its children.
- `revision` is a digest of the view and the `lsg` version. It differs from the `lsg-site.json` revision, which covers the generated files.

#### Contributions

The program writes one JSON document to standard output; standard error passes through to `lsg`'s:

```json
{
  "apiVersion": 1,
  "contributions": [
    {"kind": "rows", "pageId": "home", "blockId": "4",
     "rows": [{"content": [{"page": "show/episode 1"}, {"text": " — 42 min"}], "children": []}]},
    {"kind": "metadata", "pageId": "show/episode 1", "values": {"chapters": [{"start": 0, "title": "Intro"}]}}
  ]
}
```

- `rows` adds bullets after the children of block `blockId`, or at the end of the page's outline when `blockId` is absent. A row's `content` is a list of `{"text"}`, `{"page", "label"?}`, `{"page", "block", "label"?}` and `{"url", "label"}` (`http`, `https` or `mailto`). A page or block link shows the page's title unless it has a label. Text is shown as written: it is never read as Markdown or HTML.
- `metadata` adds values to the plugin's entry in a page's metadata, beside its page property values, for its browser module's `onPageAdopted` listener. It needs a `module`; a key may not repeat one of the plugin's `pageProperties`, and a page's entry is at most 64 KB of JSON.

Rows render inside `<ul class="lsg-derived" data-lsg-plugin="<id>">`. They are added after linked references, search records, the graph, All pages counts and embeds are built, so they create no backlink, search result or graph edge, and an embed of the target block shows only its authored children. Rows carry no block anchor of their own. Contributions to the same place appear in plugin id order, then in the order each plugin returned them.

The build stops before creating the output directory, naming the plugin and the problem, when the program cannot start, exits non-zero, runs past its timeout (it is then stopped), writes more than 64 MB or something other than that JSON document, or answers with another `apiVersion`; and, naming the contribution's position, when a contribution names a page or block outside the plugin's view, links to one, uses another URL scheme, has no rows, or breaks a metadata rule.

Rust callers can add in-process plugins to `SiteOptions::build_plugins`: any type that implements `contributions::BuildPlugin` receives the same view as a `publication::PluginView` and returns the same contributions. Their ids share one namespace with manifest ids. In-process plugins have no browser module, so they contribute rows only. [`examples/build_plugin.rs`](../crates/logseq-gardener-sdk/examples/build_plugin.rs) registers one. `publication::select` and `contributions::resolve` give other tools the same page selection and contribution checks the publisher uses.

Without a `build` section and without in-process plugins, the build creates no view and runs no plugin code. [`tests/fixtures/plugins/episode-list`](../tests/fixtures/plugins/episode-list) is a complete build plugin.

### Trust

Plugins run in the reader's browser with the page's full access; the publisher does not sandbox them. Use only plugins you would add to the site yourself. The publisher limits what data reaches a plugin to the page properties readers can already see.

A build command runs on the publishing machine with your file access. The view limits what `lsg` hands the program; it does not isolate the program, which can read anything you can. Run only build plugins you trust as much as the build itself.

## Current work

[The initial publishing outcome](https://github.com/codekiln/logseq-gardener/issues/11) and [Logseq site parity work](https://github.com/codekiln/logseq-gardener/issues/106) record delivered behavior and validation. Gitpa now [builds its podcast site with a pinned Gardener revision](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml); its [preparation code](https://github.com/codekiln/gitpa/blob/main/scripts/build_site.py) supplies site-specific presentation. [Publication speed](parity/publication-speed.md) reports how long publication takes on a Gitpa-sized garden and the encode garden, and how `--timings` shows where a build spends its time.

[Garden-wide Markdown loading](https://github.com/codekiln/logseq-gardener/pull/254) is a proposal awaiting implementation. [Issue #195 — plugin ownership and consumer contract](https://github.com/codekiln/logseq-gardener/issues/195) coordinates the plugin host with Gitpa, which plans to replace its HTML rewriting with a plugin ([Gitpa issue #93](https://github.com/codekiln/gitpa/issues/93)). [Issue #3 — parser compatibility](https://github.com/codekiln/logseq-gardener/issues/3) retains the broader adoption evidence and remaining comparison work.

## Sources

- [Project Brief](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md): describes per-page HTML through the shared garden engine.
- [Project Proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md): describes Markdown export, resolved links and embeds, referenced assets, and publication filtering.
- [Astra's publishing proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal___Astra.md): extends filtering to public diagnostics and excluded target names.
- [Gitpa publication workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml): builds the podcast site with Gardener and retains RSS validation and site-specific preparation.
