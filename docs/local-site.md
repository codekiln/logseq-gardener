# Generate a local HTML garden

Generate standalone HTML pages from selected Logseq namespaces with the Rust SDK. The output includes an index, relative page links, and supported referenced assets. Open `index.html` in a browser to read the garden.

## Try the public garden

From this repository's publishing worktree, run:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
mise exec -- cargo run -p logseq-gardener-sdk --example publish_site -- \
  "$garden_root" /private/tmp/garden-site-preview triple-lowbar \
  --include My/AI --exclude My/AI/Agent
```

Open `/private/tmp/garden-site-preview/index.html`. The command selects `My/AI` and its descendants and excludes `My/AI/Agent` and its descendants. Repeat `--include` and `--exclude` for additional roots. An empty include list produces an empty index. Selection is case-sensitive, and exclusions always win.

Choose a fresh output directory with an existing parent outside the source garden on each run. Existing output is refused, which also prevents excluded files from a previous run remaining in the site. The publisher leaves source files unchanged. A filesystem write failure may leave partial output in the newly created directory; use a fresh destination after resolving the error.

Use `triple-lowbar` when `logseq/config.edn` sets `:file/name-format :triple-lowbar`; use `legacy` for the default filename format. The command reports generated files on standard output and unsupported constructs on standard error. To keep the diagnostic report, append `2> /private/tmp/garden-site-report.txt`.

## Visibility and references

A page without a `public` property is published when its title matches the namespace policy. It withholds an entire page containing any parsed `public:: false` property, including a private child in an outline. `public:: true` preserves the namespace policy. Journals are counted and skipped pending configured date naming. Deeply nested content beyond the renderer's visibility limit also withholds its page.

Page references resolve by selected display title using Unicode lowercase comparison. Ambiguous selected names fail generation before output is created. Aliases, Logseq's additional name normalization, and Markdown links to page filenames require later lookup work. Missing and excluded destinations retain labels from the selected source with a local diagnostic.

Block references, embeds, and queries remain visible as fallbacks with diagnostics. The publisher does not expand referenced content. This makes the initial exclusion boundary testable while [Issue #4 — SDK page lookup](https://github.com/codekiln/logseq-gardener/issues/4) continues toward complete graph resolution.

## Rendering and assets

The publisher renders outlines, headings, paragraphs, emphasis, literal code, quotes, and tables. Raw HTML and Hiccup appear as escaped literal content. Unsupported syntax receives a visible fallback and a local diagnostic. Ordinary HTTP, HTTPS, and mailto anchors remain links; remote images receive a fallback.

Local asset URLs resolve relative to their source document and must point beneath the graph's `assets/` directory. The publisher copies only assets requested by selected rendered content. It rejects symlinks, traversal outside assets, absolute URLs, query/fragment suffixes, and unsupported file types. Supported types are PNG, JPEG, GIF, WebP, AVIF, ICO, PDF, MP3, MP4, OGG, WAV, and WebM. Raster image syntax renders an image; other assets render links, including non-image files requested with image syntax.

Pages have deterministic fixed-length routes derived from their titles; the publisher checks for route collisions. Each page contains its own styling and uses relative routes, so it can be read from disk or served by an ordinary static file server.

## Evidence and next work

[The local demonstration record](../openspec/changes/local-static-site/verification.md) records browser inspection, source preservation, and the fixture checks against excluded page, block, and asset content entering output. [Issue #17 — Local HTML site](https://github.com/codekiln/logseq-gardener/issues/17) tracks the initial publisher under [Issue #11 — Selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11).

The next product work is to improve reference resolution and precise private-block handling, then make the demonstrated workflow available through `lsg`. Use the local diagnostic report to choose the syntax and reference gaps that matter for your garden.
