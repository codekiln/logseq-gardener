## Why

Adding one block to one page of the encode garden and publishing again rewrites 16,056 files (228 MB): every page, every search shard and every record chunk. Every page names the search manifest by its revision, so any edit changes every page, and shards and chunks number blocks across the whole garden, so one added block renumbers all the blocks after it. The garden publishes several times an hour to its `gh-pages` branch, and each publish uploads the whole site again ([#349](https://github.com/codekiln/logseq-gardener/issues/349)).

## What Changes

- Every page names the search manifest at the fixed address `lsg-search-manifest.json`; the revision moves inside the manifest only.
- The search loader revalidates the manifest with the server on every load, and starts again from the current manifest when a file it names has gone because the site was published again.
- Shards name each block by its page and its place within that page, so a shard changes only when one of its words is added to or removed from a block.
- Each page's number of blocks moves from the title index to the manifest, so the title index changes only when a page, title or alias changes.
- Record chunks start at blocks chosen by their own digest, so an edit changes only the chunks that hold the edited page's records.
- Search format version 3 replaces version 2. **BREAKING** for hosts that read the search files or the `data-revision` attribute; a page published before the change asks the reader to reload when it meets version 3 data.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `static-search-contract`: The fixed manifest address, format version 3 and the chunk layout.
- `garden-search-index`: An added or removed block changes only its own page's chunks and the shards of its words.
- `accessible-static-search`: How the loader keeps the manifest current and recovers after a later publication.

## Impact

The SDK's search files and site wiring, the browser engine and search controller, the shared search fixture, the parity size checks and browser proof, and the Static Search Data and Host Integration API reference pages change. No dependency is added.

## Citations

- [Rewrite only changed pages when one block changes](https://github.com/codekiln/logseq-gardener/issues/349)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
