# Generate a local HTML garden

Generate standalone HTML pages from selected Logseq namespaces with `lsg publish`, using the shared Rust SDK. The output includes an index, relative page links, and supported referenced assets. Open `index.html` in a browser to read the garden.

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

Page references resolve by selected display title using Unicode lowercase comparison. Ambiguous selected names fail generation before output is created. Aliases, Logseq's additional name normalization, and Markdown links to page filenames require later lookup work. Missing and excluded destinations retain labels from the selected source with a local diagnostic.

Block references, embeds, and queries remain visible as fallbacks with diagnostics. The publisher does not expand referenced content. This makes the initial exclusion boundary testable while [Issue #4 — SDK page lookup](https://github.com/codekiln/logseq-gardener/issues/4) continues toward complete graph resolution.

## Rendering and assets

The publisher renders outlines, headings, paragraphs, emphasis, literal code, quotes, and tables. Raw HTML and Hiccup appear as escaped literal content. Unsupported syntax receives a visible fallback and a local diagnostic. Ordinary HTTP, HTTPS, and mailto anchors remain links; supported HTTPS raster images render directly. The browser loads remote media when the site is viewed; generation performs no remote fetch. HTTP media and unsupported remote types receive a fallback.

Local asset URLs resolve relative to their source document and must point beneath the graph's `assets/` directory. The publisher copies only assets requested by selected rendered content. It rejects symlinks, traversal outside assets, absolute URLs, query/fragment suffixes, and unsupported file types. Supported types are PNG, JPEG, GIF, WebP, AVIF, ICO, PDF, MP3, MP4, OGG, WAV, and WebM. Raster image syntax renders an image. MP3, WAV, and OGG image syntax renders labelled audio controls and a download link for both local and HTTPS recordings. Browser playback depends on the recording codec. PDF and video image syntax renders a link. Ordinary Markdown links remain links. HTTPS media uses a supported path extension before any query or fragment; URL credentials, controls, backslashes, and missing authorities receive fallbacks.

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

[The CLI demonstration record](../openspec/changes/cli-static-publishing/verification.md) records browser inspection, source preservation, and the fixture checks against excluded page, block, and asset content entering output. [Issue #20 — Publish through lsg](https://github.com/codekiln/logseq-gardener/issues/20) tracks the command under [Issue #11 — Selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11).

The next product work is to improve reference resolution and precise private-block handling. Use the local diagnostic report to choose the syntax and reference gaps that matter for your garden.
