## Why

GitHub Pages serves a site of at most 1 GB, and publishing the encode garden with `lsg` writes 1.22 GB ([Issue #348 · Keep the encode garden's site under GitHub Pages' 1 GB limit](https://github.com/codekiln/logseq-gardener/issues/348)), so the garden cannot be published there without the post-processing step its build added. Two things account for most of the size: every page repeats the same 24 KB of styles and 4.6 KB of inline scripts, which adds up to about 400 MB, and the Linked and Unlinked References fragment files hold 443 MB of HTML that compresses to 80 MB.

## What Changes

- The styles that `lsg` writes into every page move to one shared stylesheet, `lsg-site.css`, which each page links. The other inline style blocks, for search, the flashcard widget and build plugins' derived rows, each become a shared file linked from the same place in the head, so the order in which styles apply stays the same.
- The scripts each page carries inline shrink to the two that must run before the page paints: restoring the saved theme and layout, and opening the block a link's fragment points to. The theme controller and the search start-up move to shared files that pages load with `defer`.
- Linked and Unlinked References fragment files are stored gzip-compressed, as `_refs/<page>-linked-<n>.html.gz` and `_refs/<page>-unlinked-<n>.html.gz`, and `lsg-references.js` decompresses them in the browser. Hierarchy fragments stay readable HTML.
- **BREAKING** for readers without scripts: a page's reference sections show their headers, and the link that opened the first fragment as a readable page is removed. Pages, the Hierarchy section and All pages keep their script-free fallbacks.
- Every file that all pages name has a fixed name and content that depends only on the `lsg` version, so a garden edit never changes it.
- The publication benchmark records the output size of each part of the site, and its CI check fails when the encode garden's site exceeds 550 MB. On the encode garden, the site is projected to shrink from 1,217 MB to about 475 MB.

## Capabilities

### New Capabilities

- `published-site-size`: the shared stylesheets and scripts, the limit on what each page carries inline, stable names for files every page references, and the site-size budgets that CI checks on the encode garden.

### Modified Capabilities

- `published-reference-sections`: fragment files for Linked and Unlinked References are stored compressed and are no longer readable pages without scripts.
- `published-app-shell`: the shell's styles leave each page for the shared stylesheet, so the per-page shell budget covers only the header and sidebar markup.
- `publication-speed`: benchmark rows record the output size of each part of the site.

## Impact

- `crates/logseq-gardener-sdk/src/site/`: `mod.rs` writes the shared stylesheets and scripts and drops the inline copies; `references.rs` and `references.js` write and read compressed fragments; `theme.js`, `fragments.js` and the search start-up split into an inline part and a shared file.
- The SDK's reference index, page selection and search index are unchanged. The publisher only renders and packages what they return.
- `scripts/parity/`: `budgets.toml`, `budgets.py` and `publish_bench.py` gain the size checks; `docs/parity/publication-speed.json` gains rows with the new fields.
- Tests in `crates/logseq-gardener-sdk/tests/` that look for inline styles and scripts change to look for the shared files.
- `flate2`, which the graph view already uses for its data files, writes the compressed fragments. No new dependency.
- After a release with this change, the garden's `mise-tasks/site/build` can drop its `share_repeated_styles` step.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
