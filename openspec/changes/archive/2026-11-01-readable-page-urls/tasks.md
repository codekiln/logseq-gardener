## 1. SDK layout and checks

- [x] 1.1 Add the `page_urls` module: `PageUrls`, `readable_url(title)`, the readable file path and stem of a route, and unit tests for encoding and each unsafe title
- [x] 1.2 Add `publication::select_with_urls` and the page-to-page collision checks (folded paths, folder spellings, a page folder that is another page's file), with `SelectError::PageUrl` and `SiteError::PageUrl` naming the pages
- [x] 1.3 Add `SiteOptions::page_urls` and pass it through `publish_site_with_options`

## 2. Publisher output

- [x] 2.1 Keep reference, hierarchy, query and graph neighbourhood file names flat and derived from the title digest in readable builds
- [x] 2.2 Include readable page routes wherever the publisher selects HTML documents for head additions
- [x] 2.3 Add `<base>` and the `data-lsg-readable` script to readable documents, and draw the left sidebar's graph icon without `<use>`
- [x] 2.4 Check page first segments against the generated root names before creating the output directory
- [x] 2.5 Create page folders and write each readable page to its folder's `index.html`

## 3. Browser scripts

- [x] 3.1 Add the readable variants of the outline, graph, search engine and plugin host scripts, with a test that each replaced line occurs once
- [x] 3.2 Add Node tests for the `data-lsg-readable` script and the readable graph routes

## 4. CLI and docs

- [x] 4.1 Add `--readable-urls` to `lsg publish` and its help text, refresh the help assessment and pass `mise run clilint:check`
- [x] 4.2 Describe the option in `docs/publishing.md` and `docs/local-site.md`

## 5. Tests and evidence

- [x] 5.1 SDK tests: nested navigation links, every `href` and `src` resolving to an output file, search routes, plugin metadata, `lsg-site.json` URLs, and collision and unsafe-title errors before output exists
- [x] 5.2 CLI tests in `tests/publish.rs` for `--readable-urls` success and a collision exit
- [x] 5.3 Compare default output of this branch with the previous release on fixture gardens, byte for byte
- [x] 5.4 Check a readable build in Chromium under a path prefix and from disk, including search, the right sidebar and plugin host navigation, with `mise run readable:browser-test`
- [x] 5.5 Run `mise run ci` and report a readable build's timings on the encode garden
