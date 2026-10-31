## Why

Gitpa still edits `lsg`'s output after the build to add its homepage episode list and its episode player data: it swaps a placeholder in the generated HTML for evaluated query rows and writes player data into `</head>`. [Issue #195](https://github.com/codekiln/logseq-gardener/issues/195) asks `lsg` to offer a supported build step for this, and [Gitpa issue #93](https://github.com/codekiln/gitpa/issues/93) and its [`PublicationView` contract](https://github.com/codekiln/gitpa/blob/main/contracts/gardener-plugin.d.ts) describe what Gitpa needs. The runtime plugin host is in place, so the build-time half is next.

## What Changes

- The SDK gains a publication view: a read-only list of the pages and blocks a build publishes, after namespace selection, `public:: false` withholding and alias resolution. It follows Gitpa's `PublicationView` fields and carries no source paths, withheld pages or hidden properties.
- A plugin manifest may add a `build` section naming a command. `lsg publish` runs that command once per build, writes the view to its standard input as one JSON document, and reads the plugin's contributions from its standard output. Rust callers can also register an in-process build plugin through `SiteOptions`.
- A plugin can contribute two things: rows of derived content shown under a block or at the end of a page, and values added to its own per-page metadata for its browser module. Rows link to pages and blocks by their view ids; the publisher turns those into routes and anchors.
- Derived rows are presentation only. They create no backlinks, search records, graph edges, block anchors or embed content.
- The build checks every contribution before it writes a file and stops with the plugin id and the problem when a contribution is malformed or names a page or block outside the view.
- A manifest's `module` becomes optional when it has a `build` section, so a plugin can run only at build time. A build whose plugins have no module adds no browser host to its pages.
- Builds without build plugins do no extra work. A new CI job measures build plugins on the whole encode garden.

## Capabilities

### New Capabilities

- `publication-view`: the SDK's read-only view of a publication's selected pages and blocks, and what it withholds.
- `build-time-contributions`: build plugins (Rust and process), the process protocol, contribution kinds and validation, how derived rows render and stay out of indexes, and the build budgets.

### Modified Capabilities

- `published-plugin-host`: the manifest gains an optional `build` section and `module` becomes optional with it; per-page metadata includes contributed values; a build whose plugins have no module adds no host.
- `cli-static-publishing`: a failing build plugin stops `lsg publish` before output, and publishing help describes the `build` section.

## Impact

- SDK: a new `publication` module builds the view from the garden, the selection and the visibility rules the publisher uses today; the selection and withholding code moves there from `site/mod.rs`, so the publisher and other callers share it. A new `contributions` module holds the contribution types, validation and the resolution of row links to routes and anchors. `plugins.rs` reads the `build` section and runs process plugins.
- Publisher (`site/`): renders derived rows with the existing outline markup and merges contributed metadata into `data-lsg-page`. It renders only.
- CLI: no new option; `--plugin` accepts manifests with a `build` section. Help text and `docs/publishing.md` describe the section, the protocol and the contribution kinds.
- Parity harness and CI: a `[build_plugins]` table in `scripts/parity/budgets.toml`, a fixture build plugin, and a CI job that builds a pinned commit of the encode garden with and without build plugins.
- Gitpa can replace its homepage placeholder rewrite and its `</head>` player data rewrite with a build plugin ([Gitpa issue #93](https://github.com/codekiln/gitpa/issues/93)). Its trusted source preparation stays before `lsg`.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
