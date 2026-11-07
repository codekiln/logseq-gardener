## Why

`lsg publish --readable-urls` stops the encode garden build when two titles spell one namespace in different letter case ([Issue #364 — Give namespaces that differ only in letter case one readable folder](https://github.com/codekiln/logseq-gardener/issues/364)). The garden has the page `CLAUDE` and the pages `Claude/Artifact/...`, and 20 more such pairs, such as `tmux/Session` and `tmux/session`. Logseq treats each pair as one page, so `Claude/Artifact` is a child of `CLAUDE` there. The requirement "Safe and distinct page paths" makes any such pair fail the build. This blocks [codekiln/logseq-encode-garden#295 — Serve pages at their Logseq namespace URL](https://github.com/codekiln/logseq-encode-garden/issues/295).

## What Changes

- With `--readable-urls`, each folder that titles spell in more than one letter case gets one spelling. The folder takes the title of the page it holds, and otherwise the spelling that sorts first. Every page inside the folder uses that spelling in its route, file path and `lsg-site.json` `url`; titles keep their own spelling.
- The readable graph script carries the folder spellings, so a graph node opens the folder the build wrote.
- Two titles whose full paths differ only in letter case or Unicode normalization still stop the build.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readable-page-urls`: "Safe and distinct page paths" drops the failure for folders spelled in different letter case, and a new requirement "One spelling per folder" describes the spelling those folders get.

## Impact

- `crates/logseq-gardener-sdk/src/page_urls.rs`: `folder_spellings` and `respelled` choose and apply the spelling; `check_collisions` no longer compares folder spellings.
- `crates/logseq-gardener-sdk/src/publication.rs`: `select_with_urls` moves routes under such folders to the chosen spelling and records the spellings in `Publication::folders`.
- `crates/logseq-gardener-sdk/src/site/readable-graph.js`, `readable.rs`, `graph.rs`: the readable graph script receives the spellings.
- Tests: `page_urls` unit tests and `crates/logseq-gardener-sdk/tests/site_readable.rs`.
- Docs and help: the Readable URLs section of lsg/Ref/Publish Options, the `lsg publish help` behavior text and the recorded help reviews.
- Rust callers: `Publication` has a new public field, `folders`.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
