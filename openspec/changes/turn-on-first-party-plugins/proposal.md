# Proposal

## Why

The GitP/A episode pages read differently on the two sites that publish them. Gitpa's site has a player that keeps playing across pages, chapter headings that seek the recording and previous and next links in the header; it gets them from about 52 KB of its own scripts and from Python that rewrites `lsg`'s HTML. The logseq-encode-garden site shows plain pages ([Issue #486 — Propose first-party plugins that a garden turns on in config.edn](https://github.com/codekiln/logseq-gardener/issues/486), part of [#484 — Render the GitP/A namespace the same on both sites](https://github.com/codekiln/logseq-gardener/issues/484); the [analysis](../../../docs/parity/gitp-a-shared-rendering.md) compares the two sites).

If that behaviour ships inside `lsg` as plugins that a garden turns on in its own `logseq/config.edn`, both sites run the same code at the same `lsg` version, gitpa deletes its copies, and a garden that turns nothing on keeps matching Logseq's published site. The plugins have to keep the per-page loading that [Issue #106 — Match Logseq's look and behaviour, beat its performance](https://github.com/codekiln/logseq-gardener/issues/106) asks of every feature.

## What Changes

- `lsg` ships first-party plugins inside its binary. Each one uses the existing manifest and host contract, and its browser module and stylesheet are compiled into `lsg` and written to `plugins/<id>/` when a garden turns the plugin on.
- A garden turns plugins on with one entry in `logseq/config.edn`: `:lsg/plugins [:podcast-episode :sequence-header]`. Logseq ignores the key. An unknown or malformed entry stops the build before it writes output, and the message lists the plugins this `lsg` ships.
- A first-party plugin can pair an in-process build step with a browser module, so its build step can hand page metadata to its module. `--plugin PATH` keeps working, and its ids share one namespace with the first-party ids.
- A plugin chooses its pages by page properties and by the `logseq-entity::` type, such as `Logseq/Entity/Podcast/Episode`. Its module loads only on pages that carry its metadata, or on pages reached after one.
- Two plugins:
  - `podcast-episode`: a player that keeps playing across pages, chapter headings and timestamp-led bullets that seek the recording, and dialogue lines that seek the same player.
  - `sequence-header`: the page's previous and next links in the header.
- Each first-party plugin gets its own `[plugins.<id>]` table in `scripts/parity/budgets.toml`. `parity:budgets`, `parity:build-bench` and `parity:perf --first-party` measure each plugin against those tables, on the fixture garden in CI and on the whole encode garden against Logseq's build.
- `lsg` with no `:lsg/plugins` key, or an empty one, writes the same files as before this change.

## Capabilities

### New Capabilities

- `first-party-plugins`: the plugins `lsg` ships, the `:lsg/plugins` key, how a plugin's files reach the site, how a plugin chooses its pages, and the budget rows every first-party plugin meets.
- `podcast-episode-plugin`: the recording, chapters and timestamp-led bullets of a podcast episode page, and the player that keeps playing across pages.
- `sequence-header-plugin`: previous and next links in the header of a page with an authored page sequence.

### Modified Capabilities

- `published-plugin-host`: a site has no plugin cost unless `--plugin` or `:lsg/plugins` turns on a plugin with a module.
- `build-time-contributions`: a first-party plugin's in-process build step is a build plugin, shares the id namespace and may contribute metadata for its own module.
- `cli-static-publishing`: `lsg publish` reads `:lsg/plugins`, stops on an unknown or malformed entry, and its help lists the first-party plugins.

## Impact

- SDK: a first-party plugin registry beside `plugins.rs`; reading `:lsg/plugins` in `garden_settings.rs`; an entity-type operation over page properties; episode recording and chapter extraction; the authored page-sequence resolution moves from `site/sequence.rs` into the SDK so the bottom sequence region and `sequence-header` share it. The CLI and the language server can call each operation.
- Publisher: no new rendering. It writes first-party plugin files through the same path as manifest plugins.
- Plugin sources: `crates/logseq-gardener-sdk/plugins/podcast-episode/` and `crates/logseq-gardener-sdk/plugins/sequence-header/`, each with `lsg-plugin.json`, a module and a stylesheet; Node tests in `tests/` and Chromium checks in `mise run plugins:browser-test`.
- Parity harness: `[plugins.podcast-episode]` and `[plugins.sequence-header]` tables and two `[plugins]` rows in `scripts/parity/budgets.toml`, fixture pages under `Tools/Plugins` in `tests/fixtures/parity-garden`, a first-party variant in `parity:build-bench`, and `parity:perf --first-party` rows in the Performance page.
- Documentation: a new `lsg/Ref/First-Party Plugins` page, and links to it from `lsg/How To/Add a Site Plugin`, `lsg/Ref/Plugin Host API` and the publishing help.
- Consumers, after a release: logseq-encode-garden adds the key to its `logseq/config.edn` ([logseq-encode-garden #341](https://github.com/codekiln/logseq-encode-garden/issues/341)); gitpa adds it to `gitp-garden/logseq/config.edn`, moves its `lsg` pin and deletes its player, navigation, header and chapter-rewriting code ([gitpa #159](https://github.com/codekiln/gitpa/issues/159), [gitpa #160](https://github.com/codekiln/gitpa/issues/160)).

## Budgets and placement

The design holds the full table. In short:

- First paint: every page stays within the `[desktop]` and `[phone]` budgets (100 KB of data before the title). With any browser plugin on, a page without a plugin entry grows by at most 0.5 KB gzip; the largest fixture episode grows by at most 4 KB gzip. Neither figure grows when the garden gains 2,000 pages.
- Script: none before first paint. After it, the host module stays within 4 KB gzip; `podcast-episode` adds at most 8 KB gzip of script and styles, and `sequence-header` at most 1.5 KB, each only on pages that use it.
- Per-keystroke search latency: unchanged, at most 50 ms on desktop and 100 ms on the phone profile. Plugins add no search records, and the search files match the build without plugins.
- Peak phone memory: at most 10 MB of JavaScript heap per page, with at most 1 MB added by `podcast-episode` after playing across 20 navigations and 0.2 MB by `sequence-header`.
- Build time: each plugin's build step adds at most 0.05 ms per published page on the whole encode garden.

The SDK holds every relationship: which pages a plugin applies to, the episode's recording and chapters, and the page sequence. The publisher writes files and metadata, and the browser modules draw what the metadata says.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify/Create Uniform Interfaces](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Create%20Uniform%20Interfaces.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
