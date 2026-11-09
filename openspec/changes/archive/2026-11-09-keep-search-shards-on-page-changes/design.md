## Context

Search format version 4 names each block in a shard by two numbers: its page's place in title order and its place among that page's records. A word's blocks are pairs: the first pair is absolute, and each later pair steps to the next page. A page added in title order moves the number of every page after it, so every word whose first page sorts later changes, and so does every word whose pages straddle the new one. On the encode garden at commit `66b81e9`, adding the page `Logseq/Churn Added Page` rewrote 1,094 of 1,234 shards.

The shards must stay inside the `[search]` budgets: 64 KB gzip for one shard, 200 KB for one keystroke's download, 1,000 ms for a cold keystroke on the throttled phone profile, and 30 MB of heap growth. The browser engine is limited to 0.5 KB of growth over its size before format version 4.

The SDK builds the records, the slots and the files; the publisher only writes them. The engine must compute the same slots in the browser from the title index and the manifest.

## Goals / Non-Goals

**Goals:**

- Adding, removing or renaming a page changes only the shards that hold that page's words, apart from two rare cases that the design names.
- Total shard size and every `[search]` budget stay within their limits on the whole encode garden.
- The engine finds a block's number from a shard entry without new files on the first open.

**Non-Goals:**

- Keep the All pages listing files unchanged when a page is added. Each static All pages page lists a fixed range of titles, so on the encode garden an added page rewrites 459 of them, 7.8 MB; that is a separate change to the page list.

## Decisions

### Name each block by a slot derived from its page's route

A page with block records has a slot: the 64-bit FNV-1a digest of its route modulo the slot count. A slot's blocks are its pages' blocks, page by page in route order. A shard names each block as a slot and the block's number among the slot's blocks, ordered by slot, with the same pair stepping as version 4.

A page's slot depends only on its route and the slot count. An added page therefore changes another page's block numbers only when both share a slot and the added page's route sorts first. On the encode garden 8,560 of the 65,536 slots hold a page, so an added page shares a slot 13% of the time and moves another page's numbers about 6.5% of the time.

Alternatives, measured with the experiment in `experiments/shard-numbering/`:

- Title order (version 4) is the smallest, but moves every later page.
- Assigning numbers from the previous publication's manifest would keep them dense and stable, but the output would then depend on what the output folder held before, and a fresh checkout would number pages differently from an incremental build.
- Ordering pages by the date git first saw them would make added pages sort last, but a removed page would still move every later page, and a shallow clone or a copy without `.git` would give other numbers.
- Placing colliding pages at the next free slot (open addressing) keeps one page per slot, but an added page can then push a page out of its slot just as often, and one move can push another.

### Size the slots at four times the pages with blocks

The slot count is the smallest power of two at least four times the number of pages with block records: 65,536 for the encode garden's 9,186. Pages without blocks, such as reference-only pages, need no slot, so linking a new name adds no slot. The count changes only when that number crosses a power of two, at 16,384 pages on the encode garden; that publication rewrites every shard once.

A sparser table makes a shared slot rarer but makes each step between slots longer. Measured on the encode garden, with the string encoding below, shards totalled 2.68 MB gzip with 32,768 slots, 2.83 MB with 65,536 and an estimated 2.97 MB with 131,072, against 2.68 MB for version 4. Four times the pages halves the chance of a shared slot from twice the pages for 5% more shard bytes.

### Write each word's numbers as one base-32 string

Slots scatter a word's pages that title order kept together, which makes the version 4 JSON arrays 21% larger with 65,536 slots. Writing the numbers as one string, each number in base 32 with every digit but the last from the second half of a 64-character alphabet, removes the separators and makes most numbers one character. It reduces version 4's own shards by 14%, and the slot-named shards end up 6% larger than version 4 in total instead of 21%.

A per-page grouping (page step, block count, then block steps) was larger in every combination measured, because most words appear once per page.

### Compute the slots in the engine when the first shard loads

The slot count is at most 2^32, so a slot needs only the low 32 bits of the 64-bit FNV-1a digest, and those depend only on the low 32 bits of the offset basis and the prime. The engine computes them with 32-bit integer multiplication over each route's ASCII characters. It does so when it parses its first shard, not when it reads the title index: in Node on the encode garden, the slots take about 6 ms, and parsing every shard takes 115 ms against 111 ms for version 4. The decoder reads each string in one pass with a table of digit values; a first version that split the strings with a regular expression took 320 ms.

### Format version 5

Shard entries change shape, so every file declares format version 5, and the controller asks the reader to reload when it meets version 4 data.

### Budget the shards' total size in CI

`[search] shards_total_gzip_mb = 3` caps the encode garden's shards, which total 2.84 MB with this change. `parity:publish-bench --check`, which CI runs on a pinned commit of the encode garden, records the shard count, total and largest shard in each row and fails over the budget.

## Risks / Trade-offs

- [Shards are 6% larger in total] → The largest shard stays at 55.6 KB, under the 64 KB budget, and the keystroke downloads and latencies in `verification.md` stay within their budgets.
- [About one added page in fifteen also changes another page's shards] → That page's shards change once; it is the same cost version 4 paid for every added page, limited to one page's words.
- [The slot count doubles once per doubling of the garden] → That publication rewrites every shard once, as every added page did before.
- [The engine grows by about 0.86 KB] → The engine loads on first use only. The `[reference_only]` engine growth budget rises from 0.5 to 1.4 KB, with the reason in `budgets.toml`.
- [Keystrokes on the throttled phone are a little slower] → The 95th percentile of keystrokes that fetch no shard rose from 66 to 80 ms, under the 100 ms budget, and search's heap growth rose by 0.5 MB, to 6.3 MB of a 30 MB budget. The slots and the shorter-lived strings account for the difference; `verification.md` has the runs.

## Migration Plan

Publishing with the new `lsg` writes version 5 files. A reader with a page from an earlier publication open sees the reload message, as with every format change.

## Resolved Questions

### 1 - How should shards name a block so another page's arrival leaves them unchanged?

Decided by the agent without codekiln: by a slot computed from the page's route and the block's number among the slot's blocks. It is the only measured scheme that depends on the page alone and keeps the publisher's output a function of the garden; the alternatives above need the previous output or git history, or move pages just as often.

### 2 - How many slots?

Decided by the agent without codekiln: the smallest power of two at least four times the pages with block records. It keeps an unrelated page's shards unchanged for about 93.5% of added pages, against about 88% at twice the pages, for 5% more shard bytes; eight times would cost another 5% for about 3 more points.

### 3 - Should pages without block records get slots?

Decided by the agent without codekiln: no. They have no blocks to name, and counting them would let every new link to a missing page move the garden toward a slot count change.

### 4 - How are colliding pages ordered within a slot?

Decided by the agent without codekiln: by route, in byte order. Any order that depends on the pages alone puts an added page first half of the time; route order is simple to state and needs no other data in the browser. An order by block count would let an edit to one page move another page's numbers, which version 3 removed.

### 5 - How is the extra shard size paid for?

Decided by the agent without codekiln: by writing each word's numbers as a base-32 string. Without it, slot-named shards are 21% larger than version 4; with it, 6%.

### 6 - When does the engine compute the slots?

Decided by the agent without codekiln: when it parses its first shard. Computing them with the title index would put the hashing and sorting of 9,186 routes before the first page results on a phone.

### 7 - Which budget covers total shard size?

Decided by the agent without codekiln: a new `[search] shards_total_gzip_mb` of 3 MB, checked by `parity:publish-bench --check` in CI on the pinned encode garden commit. No budget covered the total before; 3 MB leaves about 6% over this change's 2.84 MB on the current garden.

### 8 - May the engine exceed its 0.5 KB growth budget?

Decided by the agent without codekiln: yes, up to 1.4 KB. The slot computation and the one-pass string decoding need about 0.86 KB; the engine loads only when a reader opens search, and the per-keystroke and first-open measurements stay within budget.

## Open Questions

None.
