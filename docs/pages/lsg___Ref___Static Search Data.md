tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Search Dialog]], [[lsg/Ref/Host Integration API]], [[lsg/Ref/Logseq Compatibility/Search Performance]]

- # Static Search Data Reference
	- ## Overview
		- `lsg publish --search` writes a search index beside the generated HTML, and every page reads it through the shared search dialog. Rust callers set `SiteOptions.search = Some(SearchOptions::default())`.
		- Search format version 2 publishes the stored titles, unique aliases and original bullet inline text of the selected pages as a small title index, word-prefix shards and record chunks.
		- Each page advertises the manifest through `<link rel="lsg-search" href="lsg-search-manifest-REVISION.json" data-revision="REVISION">`.
		- A small startup module loads with the page. The dialog controller, the manifest, the engine and the title index load when a reader points at or focuses the search button, presses Cmd/Ctrl+K or opens search. Each query loads only the shards its words need and the chunks of the rows it draws. [[lsg/Ref/Search Dialog]] describes the dialog.
		- The SDK's `search` module builds the records, the word index and the files, and answers the same queries in Rust; the publisher only writes the files.
	- ## Publication boundary
		- Search uses the same namespace selection and whole-page privacy check as the HTML pages, which [[lsg/Ref/Selection and Visibility]] describes.
		- ### Pages and aliases
			- Page identities and alias ambiguity come from `PageNameIndex`.
			- An ambiguous alias supplies no search key. A stored title stays its page's own searchable title even when another page claims it as an alias.
			- An alias match returns the stored page once.
		- ### Searchable text
			- Searchable bullet text includes plain prose, inline code, emphasis, subscript and superscript, unique selected page references and the labels of explicit HTTP and HTTPS links. A `{{youtube-timestamp}}` macro contributes its displayed time, such as `10:20`.
			- A tag contributes text only when its name resolves to a unique selected page.
			- Search leaves out property values, comments, paragraphs outside bullets, code fences, tables, custom containers, unsupported inline constructs, raw HTML, query arguments, the arguments of other macros, block reference expansions and asset metadata.
			- A page or block embed contributes no copied target text. The selected target's original bullets are searchable on their own.
			- Generated namespace titles and searching property content are not supported yet.
		- ### Exclusions
			- A bullet with `search-exclude:: true` excludes its whole original subtree from search. This property stays out of visible property rows and of search text.
			- A host can put the property on a temporary block that its own content fills, and keep the authored introduction around it searchable.
			- The SDK also accepts `SearchOptions.excluded_page_bodies`, a set of stored page titles compared with the normal page-name lookup rules. Those pages keep their title and alias records.
			- These controls affect search only. Excluded prose stays visible on its published page.
	- ## Files
		- Every file lives beside the generated HTML and is named by a 16-hex-digit content digest, so a rebuild after one edit changes only the manifest, the shards whose words changed and the chunk that holds the edited block. When the edit moves a chunk boundary, it changes the chunks among the same 256 records.
		- Every file declares `"format_version": 2`. The examples below come from a build of one page, `Music/Session`, with the alias `Session` and two bullets.
		- ### Manifest
			- The manifest, `lsg-search-manifest-REVISION.json`, names every other file:
			- ```json
			  {
			    "format_version": 2,
			    "revision": "023f35471723cefd",
			    "engine": "lsg-search-engine-d2ca70f1c216ba6b.mjs",
			    "titles": "448cd43839cb21ad",
			    "blocks": 2,
			    "chunks": ["bfdbc9154aefca1e"],
			    "chunk_starts": [0],
			    "shards": {"co": "e1d3cd6f73b473f3", "no": "6c944b207d31af25", "re": "0dd4f0eb1365fc18",
			               "se": "8c53ae9376251ad8", "tm": "3b338af9f70ea5e3", "wi": "f10a62484c870901"}
			  }
			  ```
			- The revision digests the manifest's other fields with a `search-v2` marker, using the publisher's FNV-1a digest. It serves as a cache identity and gives no cryptographic integrity guarantee.
		- ### Other files
			- | File | Name | Holds |
			  | --- | --- | --- |
			  | Title index | `lsg-search-titles-<titles>.json` | For each page, in title order: its title, route, aliases and number of block records. |
			  | Shard | `lsg-search-shard-<digest>.json` | Each normalized word under the shard's key, with its block record numbers, delta-encoded. |
			  | Chunk | `lsg-search-chunk-<digest>.json` | Up to 256 consecutive block records, at most 32 KB (32,000 bytes) gzipped: page route, fragment, text and up to three parents, nearest last. |
			  | Engine | the manifest's `engine` | The pure evaluator, named by its own digest. |
			- The title index, a shard and a chunk from the example build:
			- ```json
			  {"format_version": 2, "pages": [["Music/Session", "p-a33d85f823d4a0a3.html", ["Session"], 2]]}
			  {"format_version": 2, "words": [["tmux", [1]]]}
			  {"format_version": 2, "blocks": [
			    ["p-a33d85f823d4a0a3.html", "source-block-16", "Session notes", []],
			    ["p-a33d85f823d4a0a3.html", "source-block-32", "Recording with tmux.conf", ["Session notes"]]]}
			  ```
		- ### Block records and chunks
			- Block records are ordered by page, in title index order, then by source position, so a page's records are consecutive and the title index's counts give each block's page before its chunk loads. A block's number is its place in that order.
			- `chunk_starts[n]` is the number of the first block in chunk `n`, which holds the blocks up to the next chunk's start, or up to `blocks` for the last chunk. The chunk that holds a block is the last one that starts at or before it.
			- The publisher cuts the records into groups of 256, and closes a chunk at the end of a group or before the record that would take its gzip size past 32 KB. A single record over 32 KB gets a chunk of its own. Because groups always start at multiples of 256, an edit that moves a chunk boundary changes only the chunks of its own group.
		- ### Shards
			- A word belongs to the shard keyed by its first two characters; a one-character word has its own shard.
			- A shard larger than 64 KB (64,000 bytes) gzipped splits by the next character, and the words no longer than its key stay in a terminal shard under the key.
			- A terminal shard, one with a one-character key or a key that a longer key extends, holds only the word equal to its key.
		- ### Destinations
			- A record's destination fragment is `block-UUID` when the authored UUID is unique among the selected original blocks, and otherwise `source-block-POSITION`, the block's UTF-8 offset in its source file.
			- Every indexed destination is requested from the renderer and admitted only after its original anchor is emitted.
			- Parents hold safe original parent text, each at most 160 Unicode characters.
			- Records contain only published text. Original source files and file system paths are absent.
	- ## Matching and ranking
		- Text is compared lowercased with locale-independent Unicode casing, in NFC, with runs of whitespace collapsed. A word is a run of letters, digits or combining marks; every other character separates words.
		- ### Pages
			- A page matches when the query's characters, ignoring spaces, brackets, slashes, underscores and parentheses, appear in order in its title or an alias.
			- Results rank as exact title, exact alias, title or alias prefix, then by Logseq's fuzzy page score, with ties in title order.
			- A page appears once. When an alias matched best, the result names it.
		- ### Blocks
			- A block matches when every query word starts a word of its text.
			- Results rank by how many query words equal a whole word, with ties in record order.
			- A one-character query word matches only that whole word, and a query of one-character words searches titles alone.
		- Each group reports its full match count and returns at most 100 results.
		- The SDK's `SearchIndex::query` returns the same groups in the same order. [`tests/search-fixture.json`](https://github.com/codekiln/logseq-gardener/blob/main/tests/search-fixture.json) runs in both engines.
	- ## Engine API
		- Import the manifest's engine URL on first use. The engine has no DOM, network, storage or navigation side effects. The caller fetches the files relative to the manifest, and passes the parsed manifest and the text of every other file.
		- ```javascript
		  const engine = await import(engineURL);
		  const manifest = engine.prepareManifest(JSON.parse(manifestText), expectedRevision);
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
		- ### Loading and validation
			- `expectedRevision` comes from the currently loaded page's manifest link.
			- `prepareManifest` rejects every format version other than 2, a revision mismatch and malformed fields.
			- `createIndex`, `addShard` and `addChunk` reject text whose digest differs from the manifest's, and records of the wrong shape or page.
			- Invalid input throws a `TypeError` with a reload or retry message.
			- The loader must check for successful HTTP responses itself, and keep loading and error states separate from an empty result.
		- ### query
			- `query` is pure. It returns `{error, pages, blocks, currentPage}`, and each group is `{count, results}`.
			- Page results have `kind: 'page'`, `page`, `title`, `href` and `alias`, which is null unless an alias matched.
			- Block results have `kind: 'block'`, `block`, `page` and `title`.
			- `currentPage` lists the matching blocks of the page whose route the caller passes.
			- A query whose shards are not loaded returns the error `shards-not-loaded`.
			- Empty input returns empty groups.
			- A non-string, ill-formed Unicode or a query over 256 code points returns `invalid-query`. A query with more than 16 distinct words returns `too-many-terms`. A limit must be an integer from 1 to 100; otherwise the result is `invalid-limit`.
		- ### Rows and text
			- `blockRow` adds `href`, `text` and `parents` to a block result once the block's chunk is loaded, and returns null before.
			- `highlight(text, input, kind)` returns `[start, end)` UTF-16 ranges to mark: word starts in block text, or the matched characters of a title.
			- `snippet` bounds block text to 240 Unicode code points around the first match, keeping whole graphemes.
			- Results contain plain text. Render them with text nodes.
	- ## Caching and host routes
		- Resolve file URLs relative to the manifest link's folder, also under a static host's subpath.
		- Keep fetched files for the publication, keyed by revision. Keep older search files available while older HTML may still be cached; digest names let browsers cache unchanged shards and chunks across builds.
		- A page and manifest revision mismatch, or a file digest mismatch, requires a fresh page or a retry against that page's advertised files. Reusing old records with current source-offset anchors can navigate to unrelated content.
		- ### Readable routes
			- A host with readable URLs can pass a resolver: `resolveResult(result, (title, publishedHref) => readableHref)`. `resolveResult` calls the resolver with the stored title, keeps the original fragment, and expects a route without a fragment.
			- Adapt only navigation. Rewriting the JSON without recomputing its digests breaks the files.
			- A Gitpa adapter must use its title-to-readable-route map, and mark its temporary homepage query block with `search-exclude:: true` before publication. The SDK contains no host placeholder token.
			- In a build with `--readable-urls`, the page routes in the search files are the pages' folders, such as `GitP/A/Session/`, and the engine accepts them. [[lsg/Ref/Publish Options]] describes that layout.
		- [[lsg/Ref/Host Integration API]] describes `window.logseqGardenerInitSearch` for a host that keeps the dialog across pages, and `window.logseqGardenerRevealFragment` for opening a result's collapsed ancestors after the host's own navigation. The host keeps control of its audio player, theme and page initialization.
	- ## Checks
		- `mise run test` runs the Rust publication fixtures, the SDK query tests and the pure engine tests.
		- The publication fixtures inspect every search file for private and unselected sentinel text, aliases, exclusions, copied content, revision identity, exact anchor existence, repeated deterministic publication and unchanged original sources.
		- `tests/search-fixture.json` holds records, the files the SDK writes for them, and queries with their expected results; `cargo test` and `node --test` both run it. After a deliberate change, regenerate it and review the diff:
		- ```sh
		  LSG_UPDATE_SEARCH_FIXTURE=1 cargo test -p logseq-gardener-sdk --test search_fixture
		  ```
		- The JavaScript tests also cover shard selection, Unicode, malformed and oversized input, snippets, highlights, digest and version checks, and host route adaptation.
		- [[lsg/Ref/Search Dialog]] describes the browser test of the dialog.
		- ### Performance
			- [Issue 96, Validate static search loading, performance, failure handling and privacy](https://github.com/codekiln/logseq-gardener/issues/96), covered browser transfer, parsing, query latency and the failure and loading states. `scripts/parity/perf.py` and `scripts/parity/budgets.toml` take those measurements; [[lsg/How To/Measure Performance]] shows how to run them.
			- [[lsg/Ref/Logseq Compatibility/Search Performance]] records the file sizes, first open, keystroke latency and downloads of format version 2 on the whole encode garden, next to Logseq's build and Pagefind.
