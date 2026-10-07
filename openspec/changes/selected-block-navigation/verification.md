# Selected workshop navigation

The public garden was published with `AI/ES/25/ws` included and `AI/ES/25/ws/3` excluded. The generated site contains workshop pages and referenced assets. Every generated UUID link was checked against the actual ID in its generated destination; all links resolved.

A browser click on the workshop summary’s Mike Christensen reference opened the MCP workshop note and placed its nested note at the top of the viewport. The link displayed an escaped plain-text excerpt of the selected note. The temporary browser tab and loopback server were closed after validation.

```sh
mise exec -- cargo run --bin lsg -- publish \
  --graph /Users/Myer/ghq/github.com/codekiln/logseq-encode-garden \
  --output /tmp/logseq-workshop-site \
  --filename-format triple-lowbar \
  --include AI/ES/25/ws --exclude AI/ES/25/ws/3 --format json
python3 -m http.server 8000 --directory /tmp/logseq-workshop-site
```

Choose a fresh output directory. From the index, open “What I Learned at the AI Engineer Summit Workshops 2025” and follow the reference below “Person/Mike Christensen.” Exclusion controls target pages and IDs; text already written inside selected pages remains selected content.

The source snapshot across pages, journals, and assets matched after generation. The existing source garden and its local changes were preserved. Fixtures additionally check excluded/private target text and assets across all generated files, duplicate IDs, UUID case, root page IDs, custom-ID precedence, malformed IDs, nested anchors, source labels, bounded labels, and source preservation.

`cargo test -p logseq-gardener-sdk` passed. `mise run ci` passed before the additional explicit-label fixture; that focused fixture also passed. A final required-check run will cover the complete handoff revision. [Selected garden media PR #36](https://github.com/codekiln/logseq-gardener/pull/36) supplies the base implementation. OpenSpec synchronization must preserve media behavior before applying this delta.

Limitations: references provide navigation and a short label. Transclusion, implicit IDs, journals, aliases, and query expansion retain the publisher’s existing limits. Only explicit IDs attached to rendered Markdown outline items supply destinations.
