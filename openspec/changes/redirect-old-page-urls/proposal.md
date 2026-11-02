## Why

A garden that switches to `lsg publish --readable-urls` loses its old page addresses: the `p-<digest>.html` files of earlier lsg builds, and the `#/page/<name>` links of Logseq's own published site. Links to those addresses sit in feeds, notes and other sites. Gitpa and logseq-encode-garden each write their own redirects after `lsg publish` to keep them working. [Issue #469 Redirect old page URLs to readable URLs](https://github.com/codekiln/logseq-gardener/issues/469) moves those redirects into `lsg`, so both sites can drop their copies.

## What Changes

- `lsg publish --redirect-old-urls`, and `SiteOptions::old_url_redirects` set to `true`, add redirects from old page addresses to the pages the build publishes.
- With `--readable-urls`, each page, journal and namespace page gets a small file at its old `p-<digest>.html` name that opens the page's readable URL and keeps the `#fragment`.
- `index.html` sends `#/page/<name>` to the page that the name, a journal title or an alias names, ignoring letter case. `#/all-pages` opens All pages and `#/graph` opens the graph view.
- A redirect file that would share a name with another generated file stops the build before the output directory exists.
- Without the option, the output is unchanged.

## Capabilities

### New Capabilities

- `old-page-url-redirects`: which old addresses redirect, where each one leads, and the files the option writes.

### Modified Capabilities

- `cli-static-publishing`: `lsg publish` gains `--redirect-old-urls` and its help text.

## Impact

- SDK: `SiteOptions::old_url_redirects` and a new `site/old_urls.rs` that writes the redirect files, the name map and the `index.html` script.
- CLI: `src/cli.rs`, `src/help.rs` and the help assessment in `tests/assessments/help.json`.
- Docs: `lsg/Ref/Publish Options` in `docs/pages/`.
- Gitpa and logseq-encode-garden can pass the flag and remove their own redirect code once they build with a release that has it. That work belongs to those repositories.

## Citations

- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don't%20Repeat%20Yourself%20DRY.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
