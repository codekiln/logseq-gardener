# Static search contract

Enable index generation with `lsg publish … --search`, or set `SiteOptions.search = Some(SearchOptions::default())` in the SDK. Version1 emits selected stored page titles, unique aliases and original bullet inline text. Each page advertises its manifest through `<link rel="lsg-search" href="lsg-search-manifest-REVISION.json" data-revision="REVISION">`. Generation adds no search UI, script loading or corpus fetch to the ordinary page path.

The accessible dialog follows in [issue95 — search dialog](https://github.com/codekiln/logseq-gardener/issues/95). [Issue96 — search validation](https://github.com/codekiln/logseq-gardener/issues/96) covers representative browser transfer, parsing, query latency and failure/loading states. Its measurements will choose chunking or worker execution. Use the existing `scripts/parity/perf.py` and `scripts/parity/budgets.toml` for those measurements.

## Publication boundary

Search uses the same namespace selection and whole-page privacy check as HTML publishing. Page identities and alias ambiguity come from `PageNameIndex`. An ambiguous alias supplies no search key; a stored title remains a page's own searchable title even if another page claims it as an alias. Alias matches return the stored page once.

Searchable bullet text includes plain prose, inline code, emphasis, subscript/superscript, unique selected page references and explicit HTTP(S) link labels. Tags contribute text only when their names resolve to a unique selected page. Search omits property values, comments, non-bullet paragraphs, code fences, tables, custom containers, unsupported inline constructs, raw HTML, query/macro arguments, block-reference expansions and asset metadata. Page/block embeds contribute no copied target text; the selected target's original bullets participate independently. Generated namespace titles and property-content search are deferred.

A bullet with `search-exclude:: true` excludes its whole original subtree. This operational property stays out of visible property rows and search text. A host can put the control on a temporary generated-content slot while keeping surrounding authored introduction text searchable. The SDK also accepts `SearchOptions.excluded_page_bodies`, a set of canonical stored titles compared with normal page-name lookup rules; those pages retain title/alias records. These controls affect search only. Ordinary selected prose remains publication-visible content.

## Artifacts

All files live beside the generated HTML. The manifest is externally linked and fingerprinted. Its JSON fields are:

```json
{
  "format_version": 1,
  "revision": "0123456789abcdef",
  "corpus": {
    "href": "lsg-search-0123456789abcdef.json",
    "bytes": 1234,
    "records": 12
  },
  "engine": "lsg-search-engine-0123456789abcdef.mjs"
}
```

`bytes` counts the serialized UTF-8 corpus length; `records` counts page plus content records. These contain only published records. The single corpus has `format_version`, `revision`, `pages` and `content`:

```json
{
  "format_version": 1,
  "revision": "0123456789abcdef",
  "pages": [{
    "id": "p-0123456789abcdef.html",
    "title": "Music/Session",
    "href": "p-0123456789abcdef.html",
    "aliases": ["Session"]
  }],
  "content": [{
    "id": "p-0123456789abcdef.html#source-block-42",
    "page": "p-0123456789abcdef.html",
    "href": "p-0123456789abcdef.html#source-block-42",
    "text": "Authored session notes",
    "ancestors": ["Recording"],
    "position": 42
  }]
}
```

Page IDs are the publisher's output-relative routes. Content IDs combine that route and original source offset. `position` is the UTF-8 source offset used for stable ordering within the publication. The destination prefers `#block-UUID` when the authored UUID is globally unique among selected originals; otherwise it uses `#source-block-POSITION`. Every indexed destination is requested from the renderer and admitted only after its original anchor is actually emitted. Breadcrumbs contain safe original parent text, each bounded to160 Unicode characters. Content records contain full searchable inline text; query results return bounded snippets.

The revision fingerprints deterministic serialized records and evaluator source with the publisher's existing non-security digest and a `search-v1` policy marker. It serves as a cache identity, without cryptographic integrity guarantees. Revision changes follow search-visible content, titles, aliases, routes and offsets. Original source files and filesystem paths are absent from the artifacts.

## Pure consumer API

Import the manifest's revision-specific engine URL on first use. It has no DOM, network, storage or navigation side effects.

```javascript
const { prepareIndex, query, resolveResult } = await import(engineURL);
const index = prepareIndex(manifest, corpus, expectedRevision);
const outcome = query(index, userInput, { limit: 20 });
for (const result of outcome.results) {
  titleNode.textContent = result.title;
  snippetNode.textContent = result.text;
  const destination = resolveResult(result);
}
```

`expectedRevision` comes from the currently loaded page's manifest link. `prepareIndex` copies and freezes records, validates shape/version/revision/counts/routes and rejects duplicate IDs or destinations. Invalid input throws a `TypeError` with a reload/retry message. The loader must separately verify successful HTTP responses, decoded JSON and the manifest's serialized transfer length, preserve the page's expected revision, and keep loading/error states separate from an empty result. Fetches and retry/cancellation policy belong to the consumer.

`query` is pure and preserves its inputs. It returns `{error, results}`. Each result has `kind` (`page` or `block`), `page`, `title`, `href`, `text` and `ancestors`. Page results have empty `text` and `ancestors`. Results contain plain text; render with text nodes. Snippets show context around a matched term, bounded to240 Unicode code points including ellipses. Display windows preserve whole graphemes; snippets are produced only for the final returned results. The evaluator supplies no HTML or normalized highlight offsets.

Matching lowercases with locale-independent JavaScript Unicode casing, normalizes to NFC, and collapses whitespace. Accents remain significant; composed/decomposed forms match. Terms are unique whitespace-separated literal substrings; punctuation and CJK remain literal. Terms must all match a single title, a single alias, or a single bullet. Ranking buckets are exact title, exact alias, title/alias prefix, all-term title/alias, then all-term bullet content. Ties use normalized title, output route, then numeric source position. String comparison uses deterministic JavaScript code-unit ordering, independent of locale.

Empty input returns an empty successful result. Non-string, ill-formed Unicode or queries over256 code points return `invalid-query`; over16 distinct terms return `too-many-terms`. Limits must be integer1–100, otherwise `invalid-limit`. The default is20 results. Limits cap returned records; all published records participate in matching. Page and original-content matches can both appear for one source page.

## Caching and host routes

Resolve artifact URLs relative to the manifest link's directory, including under a static-host subpath. Cache a prepared corpus by format version and revision. Keep immutable corpus/manifest files available while older HTML may still be cached. A page/manifest/corpus revision mismatch requires a fresh page or a retry against that page's advertised files. Reusing an old corpus with current source-offset anchors can navigate to unrelated content.

Hosts with readable URLs can pass a resolver to `resolveResult(result, (title, publishedHref) => readableHref)`. It preserves the original fragment and calls the resolver with the stored canonical title. The resolver returns a route without a fragment. Adapt only navigation; rewriting JSON without recomputing its revision breaks the publication contract. A Gitpa adapter must use its title-to-readable-route map and mark its temporary homepage query slot with the generic subtree exclusion before publication. The generic SDK contains no host placeholder token.

For SPA navigation, load the resolved page through the host's normal navigation hook, then call `window.logseqGardenerRevealFragment(fragment, {scroll: false})` before the host's focus/scroll step. The helper opens collapsed ancestors. Hosts retain control of the audio player, theme and page initialization. Ordinary static navigation uses the same fragment helper on page load/hash changes.

## Checks

`mise run test` runs Rust publication fixtures and the pure engine tests. The search fixtures inspect the serialized artifacts for private/unselected sentinels, aliases, exclusions, copied content, revision identity, exact anchor existence, repeated deterministic publication and original-source immutability. JavaScript fixtures assert whole result order, Unicode, malformed/oversized input, safe text snippets, incompatible data and host route adaptation. Search adds no separate performance harness.
