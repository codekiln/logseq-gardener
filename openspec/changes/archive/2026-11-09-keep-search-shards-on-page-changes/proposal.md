## Why

Since search format version 3, editing a block rewrites only its page and the search files that hold it, but adding, removing or renaming a page still rewrites most search shards: 1,094 of 1,234 shards (7.9 MB) on the encode garden. Shards name each block by its page's place in title order, and a new page moves every later page's number ([#539](https://github.com/codekiln/logseq-gardener/issues/539)). The garden adds pages often, and each such publish uploads those shards again.

## What Changes

- Shards name each block by a slot, which depends only on its page's route and the number of slots, and the block's number among that slot's blocks. A page added, removed or renamed changes only the shards of its own words, except in the two cases below.
- A page added to a slot whose other page's route sorts after it also changes that page's shards. On the encode garden this happens to about one added page in fifteen.
- When the number of pages with blocks crosses a power of two, every shard changes once.
- Each word's block list is written as one compact string of base-32 numbers instead of a JSON array, which keeps total shard size within 6% of version 4.
- `lsg publish --search` writes search format version 5. **BREAKING** for hosts that read the search files; a page published before the change asks the reader to reload when it meets version 5 data.
- A new `[search]` budget caps the encode garden's total shard size, and the publication speed check in CI enforces it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `static-search-contract`: Format version 5 and how shards name blocks.
- `garden-search-index`: An added, removed or renamed page changes only the shards that hold its words.

## Impact

The SDK's search files, the browser engine and search controller, the shared search fixture, the parity size checks and the Static Search Data reference page change. No dependency is added.

## Citations

- [Keep search shards unchanged when a page is added, removed or renamed](https://github.com/codekiln/logseq-gardener/issues/539)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
