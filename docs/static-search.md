# Static search contract

Enable index generation with `lsg publish … --search`, or set `SiteOptions.search = Some(SearchOptions::default())` in the SDK. Search format version 2 publishes selected stored page titles, unique aliases and original bullet inline text as a small title index, word-prefix shards and record chunks. Each page advertises its manifest through `<link rel="lsg-search" href="lsg-search-manifest-REVISION.json" data-revision="REVISION">`. A shared header button and Cmd/Ctrl+K shortcut open a native search dialog. A tiny startup module loads with the page; the dialog controller, manifest, engine and title index load only when search opens, and each query loads only the shards its words need and the chunks of the rows it draws.

The SDK's `search` module builds the records, the word index and the files, and answers the same query in Rust; the publisher only writes the files. [Issue96 — search validation](https://github.com/codekiln/logseq-gardener/issues/96) covers representative browser transfer, parsing, query latency and failure/loading states. Use the existing `scripts/parity/perf.py` and `scripts/parity/budgets.toml` for those measurements.

## Publication boundary

Search uses the same namespace selection and whole-page privacy check as HTML publishing. Page identities and alias ambiguity come from `PageNameIndex`. An ambiguous alias supplies no search key; a stored title remains a page's own searchable title even if another page claims it as an alias. Alias matches return the stored page once.

Searchable bullet text includes plain prose, inline code, emphasis, subscript/superscript, unique selected page references and explicit HTTP(S) link labels. Tags contribute text only when their names resolve to a unique selected page. Search omits property values, comments, non-bullet paragraphs, code fences, tables, custom containers, unsupported inline constructs, raw HTML, query/macro arguments, block-reference expansions and asset metadata. Page/block embeds contribute no copied target text; the selected target's original bullets participate independently. Generated namespace titles and property-content search are deferred.

A bullet with `search-exclude:: true` excludes its whole original subtree. This operational property stays out of visible property rows and search text. A host can put the control on a temporary generated-content slot while keeping surrounding authored introduction text searchable. The SDK also accepts `SearchOptions.excluded_page_bodies`, a set of canonical stored titles compared with normal page-name lookup rules; those pages retain title/alias records. These controls affect search only. Ordinary selected prose remains publication-visible content.

## Artifacts

All files live beside the generated HTML and are named by a 16-hex-digit content digest, so a rebuild after one edit changes only the manifest, the shards whose words changed and the chunk that holds the edited page. Every file declares `"format_version": 2`. The examples below come from a build of one page, `Music/Session`, with the alias `Session` and two bullets.

The manifest, `lsg-search-manifest-REVISION.json`, names every other file:

```json
{
  "format_version": 2,
  "revision": "705a38f07276efa8",
  "engine": "lsg-search-engine-17470450146bbf8f.mjs",
  "titles": "448cd43839cb21ad",
  "blocks": 2,
  "chunk_size": 256,
  "chunks": ["bfdbc9154aefca1e"],
  "shards": {"co": "e1d3cd6f73b473f3", "no": "6c944b207d31af25", "re": "0dd4f0eb1365fc18",
             "se": "8c53ae9376251ad8", "tm": "3b338af9f70ea5e3", "wi": "f10a62484c870901"}
}
```

| File | Name | Holds |
| --- | --- | --- |
| Title index | `lsg-search-titles-<titles>.json` | Per page, in title order: title, route, aliases and its number of block records. |
| Shard | `lsg-search-shard-<digest>.json` | Each normalized word under the shard's key with its block record numbers, delta-encoded. |
| Chunk | `lsg-search-chunk-<digest>.json` | `chunk_size` consecutive block records: page route, fragment, text and up to three parents, nearest last. |
| Engine | the manifest's `engine` | The pure evaluator, named by its own digest. |

```json
{"format_version": 2, "pages": [["Music/Session", "p-a33d85f823d4a0a3.html", ["Session"], 2]]}
{"format_version": 2, "words": [["tmux", [1]]]}
{"format_version": 2, "blocks": [
  ["p-a33d85f823d4a0a3.html", "source-block-16", "Session notes", []],
  ["p-a33d85f823d4a0a3.html", "source-block-32", "Recording with tmux.conf", ["Session notes"]]]}
```

Block records are ordered by page, in title-index order, then by source position, so a page's records are consecutive and the title index's counts give each block's page before its chunk loads. A block's number is its place in that order; chunk `n` holds blocks `n × chunk_size` onwards.

A word belongs to the shard keyed by its first two characters; a one-character word has its own shard. A shard larger than 64 KB (64,000 bytes) gzipped splits by the next character, and the words no longer than its key stay in a terminal shard under the key. A terminal shard (a one-character key, or a key that a longer key extends) holds only the word equal to its key.

The destination fragment prefers `block-UUID` when the authored UUID is globally unique among selected originals; otherwise it uses `source-block-POSITION`, the UTF-8 source offset. Every indexed destination is requested from the renderer and admitted only after its original anchor is emitted. Parents hold safe original parent text, each bounded to 160 Unicode characters. Records contain only published text; original source files and filesystem paths are absent.

The revision digests the manifest's other fields with a `search-v2` marker, using the publisher's non-security FNV-1a digest. It serves as a cache identity, without cryptographic integrity guarantees.

## Matching and ranking

Text is compared lowercased with locale-independent Unicode casing, in NFC, with whitespace runs collapsed. A word is a run of letters, digits or combining marks; everything else separates words.

- **Pages.** A page matches when the query's characters, ignoring spaces, brackets, slashes, underscores and parentheses, appear in order in its title or an alias. Results rank as exact title, exact alias, title or alias prefix, then by Logseq's fuzzy page score, ties in title order. A page appears once; when an alias matched best the result names it.
- **Blocks.** A block matches when every query word starts a word of its text. Results rank by how many query words equal a whole word, ties in record order. A one-character query word matches only that whole word, and a query of one-character words searches titles alone.

Each group reports its full match count and returns at most 100 results. The SDK's `SearchIndex::query` returns the same groups in the same order; `tests/search-fixture.json` runs in both engines.

## Pure consumer API

Import the manifest's engine URL on first use. It has no DOM, network, storage or navigation side effects; the caller fetches files relative to the manifest and passes their text.

```javascript
const engine = await import(engineURL);
const manifest = engine.prepareManifest(manifestJSON, expectedRevision);
const index = engine.createIndex(manifest, await text(engine.titlesFile(manifest)));
for (const key of engine.missingShards(index, input)) {
  engine.addShard(index, key, await text(engine.shardFile(manifest, key)));
}
const outcome = engine.query(index, input, { limit: 100, currentPage: pageRoute });
const blocks = outcome.blocks.results.map(result => result.block);
for (const n of engine.missingChunks(index, blocks)) {
  engine.addChunk(index, n, await text(engine.chunkFile(manifest, n)));
}
for (const result of outcome.blocks.results) {
  const row = engine.blockRow(index, result.block);
  snippetNode.textContent = engine.snippet(row.text, input);
  linkNode.href = engine.resolveResult(row);
}
```

`expectedRevision` comes from the currently loaded page's manifest link. `prepareManifest` rejects every format version other than 2, a revision mismatch and malformed fields. `createIndex`, `addShard` and `addChunk` reject text whose digest differs from the manifest's, and records of the wrong shape or page. Invalid input throws a `TypeError` with a reload/retry message. The loader must separately verify successful HTTP responses and keep loading/error states separate from an empty result.

`query` is pure. It returns `{error, pages, blocks, currentPage}`; each group is `{count, results}`. Page results have `kind: 'page'`, `page`, `title`, `href` and `alias` (null unless an alias matched). Block results have `kind: 'block'`, `block`, `page` and `title`; `blockRow` adds `href`, `text` and `parents` once the block's chunk is loaded, and returns null before. `currentPage` lists the matching blocks of the page whose route is passed. A query whose shards are not loaded returns `shards-not-loaded`. `highlight(text, input, kind)` returns `[start, end)` UTF-16 ranges to mark: word starts in block text, or the matched characters of a title. `snippet` bounds block text to 240 Unicode code points around the first match, keeping whole graphemes. Results contain plain text; render with text nodes.

Empty input returns empty groups. Non-string, ill-formed Unicode or queries over 256 code points return `invalid-query`; over 16 distinct words return `too-many-terms`. Limits must be integers 1–100, otherwise `invalid-limit`.

## Caching and host routes

Resolve artifact URLs relative to the manifest link's directory, including under a static-host subpath. Keep fetched files for the publication, keyed by revision. Keep older search files available while older HTML may still be cached; digest names let browsers cache unchanged shards and chunks across builds. A page/manifest revision or file digest mismatch requires a fresh page or a retry against that page's advertised files. Reusing old records with current source-offset anchors can navigate to unrelated content.

Hosts with readable URLs can pass a resolver to `resolveResult(result, (title, publishedHref) => readableHref)`. It preserves the original fragment and calls the resolver with the stored canonical title. The resolver returns a route without a fragment. Adapt only navigation; rewriting JSON without recomputing its digests breaks the publication contract. A Gitpa adapter must use its title-to-readable-route map and mark its temporary homepage query slot with the generic subtree exclusion before publication. The generic SDK contains no host placeholder token.

For SPA navigation, load the resolved page through the host's normal navigation hook, then call `window.logseqGardenerRevealFragment(fragment, {scroll: false})` before the host's focus/scroll step. The helper opens collapsed ancestors. Hosts retain control of the audio player, theme and page initialization. Ordinary static navigation uses the same fragment helper on page load/hash changes.

## Checks

`mise run test` runs Rust publication fixtures, the SDK query and the pure engine tests. The publication fixtures inspect every search file for private/unselected sentinels, aliases, exclusions, copied content, revision identity, exact anchor existence, repeated deterministic publication and original-source immutability. `tests/search-fixture.json` holds records, the files the SDK writes for them, and queries with expected results; `cargo test` and `node --test` both run it. After a deliberate change, regenerate it with `LSG_UPDATE_SEARCH_FIXTURE=1 cargo test -p logseq-gardener-sdk --test search_fixture` and review the diff. JavaScript tests also cover shard selection, Unicode, malformed/oversized input, snippets, highlights, digest and version checks and host route adaptation.

## Shared dialog and persistent hosts

Search-enabled pages load a content-fingerprinted `lsg-search-start-*.mjs` startup module, which imports the full `lsg-search-ui-*.mjs` controller on first open. The header trigger remains hidden until JavaScript finds valid publication metadata and native dialog support. Cmd/Ctrl+K opens the same dialog, and Escape/Close restores the original invoking control. Arrow keys move through ordinary result anchors; Enter activates a result. Native modal behavior contains Tab focus. Modified/new-tab result links retain browser behavior. Result titles, context and snippets use text nodes. The controller uses shared appearance variables on desktop/mobile and light/dark/system themes.

The dialog distinguishes loading, an empty query, no results and failed publication loading. Failures offer explicit Retry search and Reload page buttons. Loading never reloads automatically. Reopening a loaded publication reuses its prepared index; closing aborts pending requests, and changed revisions invalidate cached results. The loader checks same-origin HTTP(S) artifact paths, response status/redirect behavior, expected revision/version and the engine's digest and shape validation. Each query fetches its missing shards in parallel, then the chunks of the rows it draws; concurrent queries share requests. Serve search-enabled output over HTTP(S), including local preview; `file:` pages retain ordinary navigation with the search trigger hidden.

For a persistent host, update the current document's `link[rel="lsg-search"]` from the incoming page before reinitializing. Remove the link when the incoming page has no search publication. Then call:

```javascript
window.logseqGardenerInitSearch(document, {
  resolvePage: (canonicalTitle, publishedHref) => readableRouteFor(canonicalTitle)
});
```

The optional resolver remains configured on subsequent calls; pass `resolvePage: null` to restore manifest-relative default navigation. Default links resolve beside the manifest, including from nested current pages. The initializer is idempotent, operates on current document metadata, and preserves one body-level dialog outside `main`. A changed/missing publication closes the dialog, invalidates pending/cached data and updates trigger visibility. Hosts can replace `main` normally.

An ordinary result activation closes the dialog before its click bubbles to the host's delegated navigator. The controller does not prevent the anchor's navigation; the host retains its existing player/theme/filter initialization and fragment reveal steps. Hosts should preserve the body-level dialog when replacing page content. The controller uses no audio or theme mutation APIs.

`mise run search:browser-test` runs a generated static publication in pinned Chromium and checks keyboard/focus, safe snippets, mobile/themes, lazy requests/cache, modified links, repeated SPA initialization, metadata invalidation, explicit retry and no-JavaScript fallback. Unit loader fixtures also cover digest mismatch, remote/version failures, one fetch per shard and stale fetch/module completion across close/revision changes. This is a functional test; measured performance remains in the existing parity harness.

The [search validation matrix](parity/search-performance.md) records completed-result latency, initial script/data cost, cold/warm loading, scale and failure evidence using the existing parity harness.

The shared dialog exposes `data-lsg-search-state` for loading/searching/ready/error/closed states and advances `data-lsg-search-complete` after a completed result render. The parity harness uses completion rather than stale-list clearing to measure each key, including no-result queries.
