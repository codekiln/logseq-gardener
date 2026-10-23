# Generate a local HTML garden

Generate standalone HTML pages from selected Logseq namespaces with `lsg publish`, using the shared Rust SDK. The output includes an index, relative page links, and supported referenced assets. Open `index.html` in a browser to read the garden.

Each page ends with an expanded **Hierarchy** section. Its flat list starts with the current complete namespace path, then shows every published descendant path at all levels. Each available slash-separated segment links to its cumulative namespace destination. A leaf still includes its own full path, so its ancestors remain reachable from the bottom of the page. The native Hierarchy disclosure collapses or expands the whole list and supports keyboard operation.

For example, `A/B/C` lists `A/B/C`, `A/B/C/D`, `A/B/C/E`, and `A/B/C/E/F` when those destinations are available. Every row displays the full path, with separate links for `A`, `A/B`, and the remaining prefixes. Missing intermediate namespaces receive their own navigation pages, so a selected `Instruments/Synth/Preset` can be reached through `Instruments` and `Instruments/Synth` without creating those Markdown files.

The complete page index includes these namespace destinations with their full logical names. The reported `pages` count continues to count selected source documents. Generated namespace pages contain navigation only: they contribute no source aliases, blocks, embeds, or homepage targets.

For a complete checkout, build, and browser walkthrough, use the [weekend trial](weekend-trial.md).

## Try the public garden

From this repository's publishing worktree, run:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
mise exec -- cargo run --bin lsg -- publish \
  --graph "$garden_root" --output /private/tmp/garden-site-preview \
  --filename-format triple-lowbar \
  --include My/AI --exclude My/AI/Agent
```

Open `/private/tmp/garden-site-preview/index.html`. The command selects `My/AI` and its descendants and excludes `My/AI/Agent` and its descendants. Repeat `--include` and `--exclude` for additional roots. An empty include list produces an empty index. Selection is case-sensitive, and exclusions always win.

Choose a fresh output directory with an existing parent outside the source garden on each run. Existing output is refused, which also prevents excluded files from a previous run remaining in the site. The publisher leaves source files unchanged. A filesystem write failure may leave partial output in the newly created directory; use a fresh destination after resolving the error.

Use `triple-lowbar` when `logseq/config.edn` sets `:file/name-format :triple-lowbar`; use `legacy` for the default filename format. The command reports publication counts and the index path on standard output and unsupported constructs on standard error. Add `--format json` for a versioned result with `pages`, `assets`, `withheld_pages`, `skipped_journals`, `diagnostic_count`, `output_directory`, and `index_path`. Completed generation with diagnostics exits zero; fatal generation failure exits one with empty stdout, and invalid arguments or selectors exit two. Run `lsg publish help` for offline command documentation. To keep the diagnostic report, append `2> /private/tmp/garden-site-report.txt`.

## Visibility and references

A page without a `public` property is published when its title matches the namespace policy. The publisher withholds an entire page containing any parsed `public:: false` property, including a private child in an outline. `public:: true` preserves the namespace policy. Journals are counted and skipped pending configured date naming. Deeply nested content beyond the renderer's visibility limit also withholds its page.

Hierarchy navigation uses only selected visible original titles. An ordinary unselected ancestor may have a navigation-only page, but its body, aliases, properties, and assets stay outside the publication. A stored private or visibility-withheld ancestor receives no generated destination, even when it is outside the include roots. Its segment remains plain text in each visible full-path row while available prefixes retain their links. A generated prefix that conflicts with a selected alias is also left unlinked. Aliases retain their existing source lookup behavior and do not create namespace branches.

Page references can use a selected page's title or leading aliases. When matching names, the publisher ignores letter case, surrounding whitespace, and a slash at either end. It also normalizes equivalent Unicode spellings of accented letters using Normalization Form C (NFC). Duplicate selected titles fail generation before output is created. Alias ambiguity keeps a visible fallback and local diagnostic. Missing and excluded destinations retain labels from the selected source. Markdown links to page filenames require later lookup work.

Competing visible claims, including alias/title collisions, keep the source label and receive a local ambiguity diagnostic. Claims from withheld pages never enter lookup. Only the last leading `alias` property is used; comma-separated plain names and direct page references are supported. Quoted or wholly literal values retain their literal spelling; nested properties and `aliases` metadata do not supply page aliases. Routes and namespace selection retain the original logical title.

References to unique explicit UUID IDs on published Markdown outline items link to the target note. The default link label is a short plain-text excerpt of the selected target; an explicit source label takes precedence. Ordinary block references provide navigation. Page and block embed macros insert selected visible content, including block descendants, using the embedded document’s own asset paths. Expansion stops at cycles, an active expansion chain of 16, 128 expansions, or 10,000 embedded AST nodes per output document; repeated copies omit original IDs so block links continue to open the original page. Missing, excluded, private, duplicate, invalid, and unsupported targets remain literal with diagnostics. IDs in root page properties are not outline targets. Unavailable or ambiguous embeds and unsupported queries retain their literal fallbacks.

## Rendering and assets

The publisher renders outlines, headings, paragraphs, emphasis, literal code, quotes, and tables. Raw HTML and Hiccup appear as escaped literal content. Unsupported syntax receives a visible fallback and a local diagnostic. Ordinary HTTP, HTTPS, and mailto anchors remain links. Email autolinks such as `<reader@example.org>` become mailto links; addresses in literal code stay literal. Email addresses inside another link’s label remain plain label text. Supported HTTPS raster images render directly. The browser loads remote media when the site is viewed; generation performs no remote fetch. HTTP media and unsupported remote types receive a fallback.

Local asset URLs resolve relative to their source document and must point beneath the graph's `assets/` directory. The publisher copies only assets requested by selected rendered content. It rejects symlinks, traversal outside assets, absolute URLs, query/fragment suffixes, and unsupported file types. Supported types are PNG, JPEG, GIF, WebP, AVIF, ICO, PDF, MP3, MP4, OGG, WAV, and WebM. Raster image syntax renders an image. MP3, WAV, and OGG image syntax renders labelled audio controls and a link to the recording. Local recording links request a download. Hosted recording links may open the recording in the browser; download behavior depends on the source host and browser. Browser playback depends on the recording codec. PDF and video image syntax renders a link. Ordinary Markdown links remain links. HTTPS media uses a supported path extension before any query or fragment; URL credentials, controls, backslashes, and missing authorities receive fallbacks.

Pages have deterministic fixed-length routes derived from their titles; the publisher checks for route collisions. Each page contains its own styling and uses relative routes, so it can be read from disk or served by an ordinary static file server.

## Try a podcast selection

Publish retained GitP sessions directly from the public garden:

```sh
mise exec -- cargo run --bin lsg -- publish \
  --graph "$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)" \
  --output /private/tmp/gitp-session-preview \
  --filename-format triple-lowbar \
  --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu
```

Open the generated index, then `GitP/A/Session/24/11/19-Tue`. Its artwork and recording reference HTTPS media; viewing artwork and listening require network access. The exclude selector removes the September 24 session and its asset namespace. Retained production notes remain visible unless their page is withheld; finer private-outline selection and homepage query evaluation remain future work.

## Evidence and next work

[The CLI demonstration record](../openspec/changes/archive/2026-10-08-cli-static-publishing/verification.md) records browser inspection, source preservation, and the fixture checks against excluded page, block, and asset content entering output. [Issue #20 — Publish through lsg](https://github.com/codekiln/logseq-gardener/issues/20) tracks the command under [Issue #11 — Selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11).

The next product work is to improve reference resolution and precise private-block handling. Use the local diagnostic report to choose the syntax and reference gaps that matter for your garden.

## Customize appearance

Supply a local CSS file to style the index and every selected page:

```sh
lsg publish --graph /path/to/garden --output /private/tmp/styled-site --filename-format triple-lowbar --include GitP/A/Session --stylesheet /path/to/theme.css
```

The publisher copies the file unchanged to `custom.css` beside the HTML files and loads it after the built-in styles. Standard CSS specificity and cascade rules apply. Omitting `--stylesheet` keeps the built-in presentation. The stylesheet does not contribute to the reported graph asset count.

The path must name a readable regular file with a `.css` extension (case-insensitive) and have no symlink components. Use the physical path if a parent directory is a symlink, such as `/private/tmp` on macOS. Invalid stylesheet input fails before the output directory is created. Choose CSS you trust: the browser can request resources referenced by `url()` and `@import`. Relative resource URLs resolve from the copied stylesheet in the output root. Only the specified CSS file is copied; arrange its other resources separately.

SDK callers can use `site::publish_site_with_options` with `SiteOptions { stylesheet: Some(path) }`. The existing `site::publish_site` function continues to use built-in presentation. The `publish_site` SDK example accepts the same optional `--stylesheet PATH` flag.

## Choose a homepage

Add `--homepage "Ghost in the Patch"` to render a uniquely selected visible page or alias at `index.html`. The original page route remains available and the complete page list moves to `pages.html`. Every page links to Home and All pages. An unavailable or ambiguous homepage fails before output is created. Omitting `--homepage` retains the generated page index.

## Collapsed outlines

An outline parent with an attached `collapsed:: true` property starts closed.
Readers can open its summary with a click or the keyboard to reveal the complete
outline. Nested closed parents open independently. False or absent flags keep the
outline expanded, and a leaf stays visible without an empty disclosure.

Formatting and links remain in the summary. When a parent contains a page or block
embed, its text labels the summary and the complete rich content appears inside.
Media, transclusions and publication visibility keep their existing behavior.

Links to blocks inside closed outlines reveal the containing disclosures, including
on the initial page load. Sites that replace document content during navigation can
call `window.logseqGardenerRevealFragment(hash, {scroll: false})` before their own
fragment focus and scrolling. The helper accepts an encoded fragment beginning with
`#`, opens only the target's containing disclosures, and returns the target element
or `null` when unavailable.

## Linked references

Each source and namespace page has a bottom **Linked references** section showing
how many published pages and referring contexts link to it. The counts remain
visible when the section is collapsed, including when both are zero. Several
links to the same destination in one context count once; several referring blocks
on one page count as one linking page and multiple references. Leading property
references have a minimal **Page properties** context. Pure block UUID references
retain block navigation and do not create page backlinks.

Related-page controls offer **Any**, **Include**, and **Exclude**. A result must
match every included page and none of the excluded pages in its reference path.
Parent references remain visible when a matching child supplies the related page;
unrelated child branches hide. **Find related pages** searches the available
filter options. **Reset filters** clears the selections and restores every result.
Active filters show the visible versus total page and reference counts.

Only selected visible source content supplies references and filter identities.
Aliases resolve through the same published-name lookup as ordinary links.
Generated hierarchy and navigation links do not add backlinks. Authored links can
refer to an already-safe generated namespace destination. Reference copies retain
source-relative assets and link back to the originating page or outline block.

The publisher reads these top-level integer settings from `logseq/config.edn`:

- `:ref/linked-references-collapsed-threshold` defaults to `100`. A section starts
  closed when its reference-context count reaches this threshold. Set it to `0`
  to start every section closed while keeping its counts visible.
- `:ref/default-open-blocks-level` defaults to `2`. Reference-context parents at
  this depth or deeper start closed. Authored collapsed parents also stay closed.

Missing settings use defaults. Unusable config receives a local diagnostic;
reading is bounded and never evaluates EDN or follows config-path symlinks.
Native disclosures and all unfiltered context remain readable without JavaScript.
Filter controls appear when the trusted script initializes them.

A host that replaces page content can call
`window.logseqGardenerInitBacklinks(root)` after inserting the new content. The
call is idempotent and separate from `logseqGardenerRevealFragment`; it installs
section-local filter controls and does not change navigation or media playback.
