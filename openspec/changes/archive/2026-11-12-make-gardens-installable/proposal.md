# Proposal

## Why

A reader who likes a published garden can already add it to the Home Screen or Dock, but the result is plain: with no icon, iOS 16.4 and later draws a monogram from the site's first letter. The person publishing the garden has no way to choose the app's name or icon ([Issue #604 — Propose installable published gardens](https://github.com/codekiln/logseq-gardener/issues/604), part of [#603 — Make published gardens easy to install as web apps](https://github.com/codekiln/logseq-gardener/issues/603)).

A web app manifest, an icon and a few head tags give an installed garden its own name, icon and window. Installation needs no service worker: Chrome has offered menu installation without one since version 108 on Android and 112 on desktop, and Safari adds any page to the Home Screen or Dock. Offline reading is a separate decision ([Issue #607](https://github.com/codekiln/logseq-gardener/issues/607)) and stays out of this change.

## What Changes

- A garden opts in with one key in its own `logseq/config.edn`: `:lsg/install {:name "Ghost in the Patch" :short-name "GitP" :icon "assets/app-icon.png"}`. Logseq ignores the key. A garden without the key publishes byte-for-byte the site it publishes today.
- With the key, `lsg publish` writes `lsg.webmanifest` and `lsg-icon.png` at the site root. Every full page links the manifest, links the icon as `apple-touch-icon` and sets `theme-color` for light and dark appearance. All three use relative paths, so the site works under a GitHub Pages path such as `/gitpa/`, with readable URLs, and opened from disk.
- The manifest has no `id`. Its identity is its `start_url`, `./`, which resolves to the garden's own folder wherever it is hosted, so gardens that share a GitHub Pages origin install as separate apps.
- Chromium browsers get an Install app item in the More menu. It stays hidden until the browser offers installation and uses `beforeinstallprompt`. Safari and Firefox get no in-page control, and the documentation tells their readers to use the Share menu.
- `lsg` adds no service worker, no cache and no browser storage for installation.
- Documentation: a how-to for readers (`lsg/How To/Install a Published Garden as an App`), a reference for the key (`lsg/Ref/Install Options`), and a pointer from `lsg/How To/Publish a Garden to GitHub Pages`.
- A checklist of real-device checks for iOS, iPadOS, macOS and Android, and a set of automated checks that run in CI.

## Capabilities

### New Capabilities

- `installable-gardens`: the `:lsg/install` key and its validation, the manifest and icon, the head tags, the scope and identity rules under a path prefix, the Install app item, the size budgets and the documentation.

### Modified Capabilities

- `published-site-size`: the manifest and icon join the files that every page of an installable site references, with fixed names that depend only on the `lsg` version and the `:lsg/install` value.

## Impact

- SDK: reading and checking `:lsg/install` in `garden_settings.rs` into an `InstallSettings` value, a new `install.rs` that builds the manifest and checks the icon, and the head tags and menu item in the publisher's page writer. A garden without the key takes the existing code path.
- Publisher: writes `lsg.webmanifest`, `lsg-icon.png` and `lsg-install.js`, reserves the three names under `--readable-urls`, and adds the head tags and the hidden menu item to every full page of an installable site.
- Scripts: one new file, `crates/logseq-gardener-sdk/src/site/install.js`.
- Parity harness: an `[install]` table in `scripts/parity/budgets.toml`, an `install` variant in `parity:budgets` and `parity:build-bench`, and a Chromium check in the plugin browser tests.
- Documentation: the two pages above, one pointer, and a new row in `lsg/Ref/Logseq Compatibility/Performance`.
- Consumers, after a release: the documentation site in this repository, logseq-encode-garden and gitpa can each add the key and an icon. Each is its own issue in the consumer's repository.

## Budgets and placement

The design holds the full table. In short:

- Sites without the key: identical output, so no budget changes.
- Sites with the key: each page grows by at most 0.5 KB uncompressed and 0.3 KB gzip; `lsg-install.js` is at most 1.5 KB uncompressed and 1 KB gzip; the manifest is at most 1 KB; the icon is a PNG of at most 256 KB; first paint stays within the existing 100 KB data and 16 KB script budgets; no service worker, cache or storage; search files are identical; the build adds at most 50 ms to the encode garden.
- The SDK holds every rule: reading and checking the key, the manifest's contents, the icon checks and which pages get the tags. The publisher writes files and tags. The browser script only reacts to `beforeinstallprompt`.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
