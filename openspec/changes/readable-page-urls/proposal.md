## Why

`lsg publish` names every page file after a digest of its title, such as `p-0e2769dd320a1844.html`, so a reader cannot tell from a URL which page it opens, and nobody can write a page's URL by hand. Gitpa rewrites its output into readable namespace URLs such as `GitP/A/Session/26/10/20-Tue/` after every build, in [gitpa scripts/site_routes.py](https://github.com/codekiln/gitpa/blob/main/scripts/site_routes.py) and [gitpa scripts/build_site.py](https://github.com/codekiln/gitpa/blob/main/scripts/build_site.py). The documentation site planned in [Issue #322 Publish the documentation as a Logseq graph on GitHub Pages](https://github.com/codekiln/logseq-gardener/issues/322) needs the same URLs, so that `lsg help` and the README can link to its pages. [Issue #324 Publish pages at readable namespace URLs](https://github.com/codekiln/logseq-gardener/issues/324) moves the capability into `lsg`.

## What Changes

- `lsg publish --readable-urls`, and `SiteOptions::page_urls` set to `PageUrls::Readable`, write each page, journal and namespace page to `<namespace path>/index.html` and link to it as `<namespace path>/`, percent-encoding each segment as Gitpa does. `GitP/A/Session` is written to `GitP/A/Session/index.html` and linked as `GitP/A/Session/`.
- Home, All pages, the journals feed and the graph view stay at the output root under their current names.
- Every link, asset, stylesheet, script, search result, right sidebar card, page graph, plugin host navigation and `lsg-site.json` `url` works from nested folders, when the site is opened from disk, and when a static server serves it under a path prefix such as `/logseq-gardener/`.
- A title that cannot form a safe path, and titles whose paths would collide on a case-insensitive file system, stop the build before the output directory exists, with a message naming the pages.
- Without the option, the output is unchanged byte for byte, so Gitpa's pinned release keeps working until it opts in.
- The SDK gives the readable URL of any title, so the CLI and other tools can link to a page.

## Capabilities

### New Capabilities

- `readable-page-urls`: the readable URL layout, links from nested folders, the safety and collision checks, and how search, the right sidebar, the page graph and the plugin host find pages in that layout.

### Modified Capabilities

- `cli-static-publishing`: `lsg publish` gains `--readable-urls` and its help text.
- `published-site-manifest`: a page's `url` can name the folder that holds its `index.html`.

## Impact

- SDK: a new `page_urls` module with the `PageUrls` layout, the readable URL of a title, and the safety and collision checks; `publication::select_with_urls`; `SiteOptions::page_urls`; new `PageUrl` variants on `SelectError` and `SiteError`.
- Publisher (`site/`): writes page folders, adds a `<base>` element and a short link script to readable pages, keeps reference, hierarchy, query and graph files flat under their current names, and serves readable variants of the outline, shell, sidebar, graph, search engine and plugin host scripts.
- CLI: `src/cli.rs`, `src/help.rs` and the help assessment in `tests/assessments/help.json`.
- Docs: `docs/publishing.md` and `docs/local-site.md`.
- Gitpa can pass `--readable-urls` and drop its `package_pages` step after it upgrades; that work belongs to the Gitpa repository.

## Citations

- [My/Principle/Make it Obvious](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20it%20Obvious.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don't%20Repeat%20Yourself%20DRY.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
