## 1. SDK

- [x] 1.1 Add a plugin-manifest type and a parse-and-validate operation (fields, `apiVersion`, id format, path checks, symlinks, regular files) with unit tests for each rejection
- [x] 1.2 Add `SiteOptions::plugins`, validate all manifests before output creation, sort by id, and reject duplicate ids and route collisions
- [x] 1.3 Expose the visible page-property values a plugin names, reusing the published-properties visibility rules, with a test that hidden properties and withheld pages yield nothing

## 2. Site manifest

- [x] 2.1 Write `lsg-site.json` from the publisher's route table (id, title, url, kind) and copied assets, sorted, with a revision digest over generated files
- [x] 2.2 Test that every listed URL exists, withheld pages are absent, and two builds produce identical bytes

## 3. Plugin output

- [x] 3.1 Copy plugin modules and stylesheets to `plugins/<id>/` and add plugin entries to `lsg-site.json`
- [x] 3.2 With plugins, add the `data-lsg-page` metadata element and the after-paint host loader to every page
- [x] 3.3 Test that a build without plugins produces HTML identical to the previous build

## 4. Page initializers

- [x] 4.1 Add `window.logseqGardenerPage(init)` and run registered initializers on first load (in `host.mjs`, so a site without plugins keeps `lsg-shell.js` unchanged)
- [x] 4.2 Move per-page setup in the outline, references, media, queries, slides, flashcards, graph, sidebar and fragment scripts into initializers that clean up on abort (the host's initializer list calls each script's existing setup entry point; `lsg-queries.js` gains one; media, slides, flashcards, sidebar and dialogue use document-level listeners installed once, and the graph view loads normally)
- [x] 4.3 Keep `lsg:media-seek` behavior and its existing tests unchanged

## 5. Browser host

- [x] 5.1 Write `site/host.mjs`: plugin import by activation, export checks, `setup`, `onPageAdopted`, `persistentRegion`, disposal on unload
- [x] 5.2 Add same-document navigation: link and history interception, fetch, superseded-response handling, revision check and notice, swap, initializers, plugin notification, fragment reveal and focus, fallback to a full load on error
- [x] 5.3 Add a fixture plugin under `tests/fixtures/plugins/` that uses every host function and holds a persistent `<audio>` element
- [x] 5.4 Browser tests: recording keeps playing across navigation and Back, on-demand loading, stale response dropped, fragment after adoption, setup failure cleanup, listener count after 20 navigations

## 6. CLI and docs

- [x] 6.1 Add repeatable `--plugin PATH` to `lsg publish`, its errors and help text; update clilint checks
- [x] 6.2 Document `lsg-site.json`, the plugin manifest, the host API and lifecycle, and the trust model in `docs/publishing.md`

## 7. Budgets and evidence

- [x] 7.1 Add `[plugins]` budgets and `parity:budgets` / `parity:perf` rows for the fixture plugin on the whole encode garden
- [x] 7.2 Record evidence in the PR: no-plugin byte comparison, host size, navigation timing, and screenshots of the fixture recording playing across navigation
- [x] 7.3 Comment on [issue #195](https://github.com/codekiln/logseq-gardener/issues/195) with the shipped API and its differences from [Gitpa PR #94](https://github.com/codekiln/gitpa/pull/94)
