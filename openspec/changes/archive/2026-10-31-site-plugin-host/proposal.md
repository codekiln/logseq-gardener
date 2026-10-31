## Why

Sites built on `lsg`, such as Ghost in the Patch, need their own browser behavior: a recording player that keeps playing across page changes. Today Gitpa adds it by rewriting generated HTML with string replacement, reading routes out of `pages.html`, and running a second router beside `lsg`'s scripts. [Issue #195](https://github.com/codekiln/logseq-gardener/issues/195) asks `lsg` to own those extension points, and [Gitpa PR #94](https://github.com/codekiln/gitpa/pull/94) proposes the consumer contract this change implements.

## What Changes

- Every build writes `lsg-site.json`, a versioned list of published pages (id, title, URL, kind) and copied assets, so callers stop reading routes from HTML.
- `lsg publish --plugin PATH` (repeatable) and `SiteOptions::plugins` take a plugin manifest: id, host API version, module, stylesheets, activation and the page properties the plugin may read. The build checks every manifest before creating output and rejects unsupported API versions, duplicate ids, missing or escaping asset paths and route collisions.
- With at least one plugin, each page carries its plugin metadata as JSON, and a small host module loads after first paint. The host owns same-document navigation, gives each page an `AbortSignal` that is aborted before the page is replaced, keeps plugin regions outside `<main>` mounted, re-runs `lsg`'s page scripts on the new page, calls plugins before fragment focus, and disposes everything once.
- `lsg`'s page scripts register their per-page setup with one shared initializer, so the same code runs on first load and after a same-document navigation.
- With no plugins, generated pages are unchanged apart from `lsg-site.json`, and no plugin or host module is requested.

## Capabilities

### New Capabilities

- `published-site-manifest`: the versioned `lsg-site.json` file listing published pages and assets.
- `published-plugin-host`: plugin manifests, build-time validation, per-page plugin metadata, the browser host, its lifecycle and same-document navigation.

### Modified Capabilities

- `cli-static-publishing`: `lsg publish` gains the repeatable `--plugin PATH` option, its errors and its help text.

## Impact

- SDK (`garden-core` side): page identity for the manifest comes from the existing page-name operations; a new plugin-manifest parser and validator lives in the SDK so the CLI, the language server and other callers share it. The selection of page properties a plugin may read reuses the published-properties visibility rules.
- Publisher (`site/`): new `site/plugins.rs` copies plugin assets under `plugins/<id>/`, writes per-page metadata and `lsg-site.json`; new `site/host.mjs` is the browser host; `shell.js`, `outline.js`, `references.js`, `media.js`, `queries.js` and the other per-page scripts move their setup into registered initializers.
- CLI: `src/cli.rs` and `src/help.rs` gain `--plugin`; `docs/publishing.md` documents the manifest and lifecycle.
- Parity harness: a `[plugins]` table in `scripts/parity/budgets.toml` and benchmark rows for a fixture plugin.
- Gitpa adopts the host in its own repository after this change merges ([Gitpa issue #93](https://github.com/codekiln/gitpa/issues/93)); its legacy player entry from [Gitpa PR #96](https://github.com/codekiln/gitpa/pull/96) is replaced by a plugin module.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
