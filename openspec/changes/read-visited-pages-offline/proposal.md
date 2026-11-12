# Proposal

## Why

A reader on a phone often loses the network between two pages: on a train, in a lift, on slow hotel Wi-Fi. A published garden is static HTML, so a page the reader has already opened could open again with no connection. Today it cannot ([Issue #607 — Evaluate and propose offline reading for published gardens](https://github.com/codekiln/logseq-gardener/issues/607), part of [#603 — Make published gardens easy to install as web apps](https://github.com/codekiln/logseq-gardener/issues/603)).

The assessment in `design.md` found the benefit real and the risks controllable, but only with limits. A service worker that stored the whole garden would break the project's performance rules: the encode garden publishes about 13,500 pages and 600 MB. Offline reading therefore covers pages the reader has visited, up to a fixed size, and only after the reader asks for it. The rest of the risks (copies of pages the author later unpublishes, stale pages, a worker that controls neighbouring gardens on a shared host) each get a rule below.

## What Changes

- A garden opts in with one key in its own `logseq/config.edn`: `:lsg/offline true`. Logseq ignores the key. A garden without the key publishes byte-for-byte the site it publishes today. The key is independent of `:lsg/install` from [make-gardens-installable](../make-gardens-installable/proposal.md); each works alone.
- With the key, `lsg publish` writes `lsg-offline.js` and `lsg-sw.js` at the site root and adds two items to the More menu: Save pages for offline reading and Remove offline copies. The page script loads with `defer`, registers nothing and stores nothing until the reader chooses Save.
- After the reader chooses Save, the service worker keeps a copy of each page the reader opens, and of the small files that page needs, in one Cache Storage cache named for the garden's own path. It uses network first: a reader who is online always gets the current page, and a reader who is offline gets the last copy. A page that was never opened offline shows the browser's usual offline page.
- A cached page shows a notice with the date it was saved, so a reader can tell an old copy from the current page.
- Storage has fixed bounds: copies of at most 256 KB each, at most 250 pages and 32 MB, with the least recently used copy dropped first. Audio, video, the search index and anything larger stay online only.
- Unpublished pages do not linger: copies expire after 30 days, a page that now returns 404 or 410 is deleted when the reader next opens it online, and every new publish makes the worker check each saved page and delete the ones the site no longer serves.
- The worker is registered at the garden's own folder and never at an origin root. On a garden served from an origin root, the Save item is absent.
- `:lsg/offline false` writes a worker that deletes its caches and unregisters itself, for authors who turn offline reading off after readers have saved pages.
- Documentation: a how-to for readers (`lsg/How To/Read a Published Garden Offline`) and a reference for the author (`lsg/Ref/Offline Options`).
- Size budgets in `scripts/parity/budgets.toml`, a table of browser checks that run in CI, and a short real-device checklist.

## Capabilities

### New Capabilities

- `offline-reading`: the `:lsg/offline` key and its validation, the two menu items, the visited-page cache and its limits, the update and removal rules, the isolation rules for a shared origin, the removal worker, the size budgets and the documentation.

### Modified Capabilities

None.

## Impact

- `lsg-sdk` reads the key, owns the limits and writes the worker source as a pure function of the settings and a build identifier. `lsg publish` writes the files and the menu items. Pages of a garden without the key do not change.
- No change to search, graph, queries or any first-paint rule: nothing loads before a reader opts in, and an active worker adds at most 50 ms to an online navigation.
- No new dependency.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
