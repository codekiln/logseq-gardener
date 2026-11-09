## Why

Six older publishing requirements contradict newer requirements or the code, so a reader of the main specs cannot tell which behaviour is intended. [Issue #371 Reconcile specs and code where they disagree](https://github.com/codekiln/logseq-gardener/issues/371) lists them, from the review of the publishing pages in [PR #365 correct publishing pages that contradict the specs](https://github.com/codekiln/logseq-gardener/pull/365) and the archive of `include-all-pages` in [PR #570 sync and archive include-all-pages](https://github.com/codekiln/logseq-gardener/pull/570). Two of the code disagreements are bugs small enough to fix with the spec changes.

## What Changes

- `local-static-publishing` "Render supported syntax and report unsupported references" says the publisher runs queries at build time, labels an unlabelled block reference with the target's display content, and renders `{{renderer …}}` as nothing, as the newer published-queries, published-transclusion and published-inline-markup requirements say.
- `published-graph-view` "Load graph data only on the graph page" lets an open Page graph card fetch the graph files, as published-page-graph "Load neighbourhoods per page" says.
- `static-search-contract` "Selected-source search corpus" lets a `{{youtube-timestamp}}` add its displayed time to search, as published-media-embeds "Show timestamps as text outside the page body" says.
- `publishing-namespace-selection` "Explicit namespace inclusion" says an empty include list selects nothing unless the policy selects every page.
- `cli-static-publishing` "Explicit SDK-backed publishing" lists `--include-all` among the publish flags.
- `authored-page-sequences` "Present accessible sequence navigation" leaves a neighbour out of the bottom Page sequence region when its key is in `:block-hidden-properties`, as the code and its tests do.
- Bug fix: a page reference in a hidden page property no longer adds the page to the Linked References of the referenced page's namespace parent.
- Bug fix: the `revision` in `lsg-site.json` and in each page's plugin metadata changes when only a build plugin's metadata values change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-static-publishing`: queries, block reference labels and `{{renderer …}}` follow the newer requirements.
- `published-graph-view`: an open Page graph card may fetch the graph files.
- `static-search-contract`: a YouTube timestamp contributes its displayed time.
- `publishing-namespace-selection`: an empty include list selects every page when the policy selects every page.
- `cli-static-publishing`: `lsg publish` lists `--include-all`.
- `authored-page-sequences`: a hidden `prev` or `next` key also hides that neighbour's bottom link.

## Impact

- `crates/logseq-gardener-sdk/src/references.rs`: leading property blocks name only the shown rows' pages and those pages' namespace parents.
- `crates/logseq-gardener-sdk/src/site/mod.rs` and `site/plugins.rs`: the revision digest also reads each page's plugin values and the plugin host script.
- Published sites: a namespace parent loses Linked References that came only from hidden properties; sites with browser plugins get a new revision when contributed metadata changes.
- Remaining items of #371 stay open: the parity harness waiting for `data-lsg-search-drawn`, and the page initializers, which [Issue #576 Check page behaviour after a plugin navigation and align the page initializers spec](https://github.com/codekiln/logseq-gardener/issues/576) tracks.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
