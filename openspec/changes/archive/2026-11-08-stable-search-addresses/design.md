## Context

Search format version 2 names every search file by its content digest, which suits caching, but three things make one edit rewrite almost the whole site:

- Every page carries `<link rel="lsg-search" href="lsg-search-manifest-<revision>.json" data-revision="<revision>">`. The revision follows every search file, so every page changes.
- A shard lists, for each word, block numbers counted across the whole garden. One block added to `Deno` renumbers every block on the pages that sort after it, so nearly every shard changes.
- Record chunks hold 256 consecutive blocks and close at each multiple of 256. One added block shifts every later boundary, so nearly every chunk changes.

On the encode garden at commit `a7d948e`, appending one block to `pages/Deno.md` and publishing again rewrote 14,602 pages, 1,127 shards, 324 chunks, the title index, the manifest and `lsg-site.json`: 16,056 files, 228 MB.

## Goals / Non-Goals

**Goals:** After a one-block edit, only the edited page, the search manifest, `lsg-site.json`, the chunks that hold the edited page's records and the shards that hold the edited block's words change. Readers never combine files from two publications. First open, keystroke latency and the `[search]` budgets stay where they are or improve.

**Non-Goals:**

- Keep shards unchanged when a page is added, removed or renamed. Shards number pages in title order, so such an edit still changes most shards (about 5 MB on the encode garden), while pages and chunks stay as they are.

## Decisions

### Name the manifest at a fixed address

Every page links `lsg-search-manifest.json` without a revision. The manifest keeps its `revision` field, which identifies the publication. The engine, title index, shards and chunks keep digest names, and the engine checks every file it parses against the digest the manifest lists.

### Keep the manifest current by revalidating it

The loader already fetches every search file with `cache: 'no-cache'`, so the browser asks the server whether its copy is current and receives a 304 response when it is. The manifest it reads is therefore the one published when search loads. A tab that loaded the manifest before a later publication may then ask for a shard or chunk that publication removed. The loader treats any failed or mismatched file as that case: it forgets the manifest, the index and every fetched file, and the dialog shows its Retry state. Retry loads the current manifest and builds a new index from it, so old and new files never mix.

Alternatives: putting the revision in a query string (`lsg-search-manifest.json?r=…`) would still change every page. A service worker could manage freshness, but it adds a moving part for a problem that revalidation already solves.

### Name blocks by page in shards

A shard lists each word's blocks as pairs: a page number in title order and the block's number among that page's records. The first pair is absolute; each later pair steps to the next page and gives the block number there, or steps zero pages and gives the step from the previous block. The engine turns each pair into the same global block number as before, so ranking, chunk lookup and the Rust query stay unchanged.

On the encode garden this makes shards 19% larger in total gzip (2.16 MB to 2.58 MB); the largest shard fell from 56.6 KB to 48.9 KB because the 64 KB budget splits shards by gzip size. Naming pages by a number derived from their route instead would also survive added pages, but the measured cost was 50% to 75% more shard bytes, because those numbers are spread over a much larger range.

### Move block counts into the manifest

The engine needs each page's block count to turn pairs into block numbers. The counts move from the title index to the manifest, which changes with every publication anyway, so the 244 KB gzip title index changes only when a page, title or alias changes.

### Start chunks at blocks chosen by their own digest

A block starts a new segment when the FNV-1a digest of `<page route>#<fragment>` is a multiple of 64. Each segment is then cut, as before, into groups of at most 256 records and at most 32 KB gzip. Whether a block starts a segment depends only on that block, so an added, removed or edited block changes only its own segment's chunks. A segment longer than 256 records (about 2% of segments) still splits at fixed offsets, so an edit inside one changes the chunks after it within that segment.

Segments average about 64 records, so the encode garden has 1,906 chunks instead of 475. Each chunk is smaller (3.0 KB gzip on average instead of 10.8 KB), so drawing a result row downloads less, and the manifest grows from 23 KB to 50 KB gzip.

### Load the engine and the title index together

The manifest grew, so the loader now fetches the title index while it imports the engine instead of after. Both names come from the manifest, and the loader checks that the engine derives the same title index name.

### Format version 3

The shard, title index and manifest shapes change, so every file declares format version 3. The controller of a page published earlier rejects version 3 data and asks the reader to reload, which loads the new page and controller.

## Risks / Trade-offs

- [A page added, removed or renamed still changes most shards] → Pages and chunks stay unchanged in that case. The non-goal above records it as later work.
- [Shards are 19% larger] → Each shard stays within the 64 KB budget; on the encode garden the 95th percentile keystroke download fell from 76 KB to 38–40 KB because rows now fetch smaller chunks, and the largest keystroke download stayed within the 200 KB budget.
- [The manifest is larger, on the critical path of the first open] → The parallel engine and title index fetch more than pays for it: on the throttled phone profile, first page results came in about 100 ms sooner than on main.
- [A reader keeps an old manifest in memory across a publication] → The first missing file forgets it and Retry loads the current one; the browser proof covers this case.
- [The search phase of a build takes about 0.4 s longer on the encode garden] → It stays inside the 25% search build budget and the 20% publication budget.

## Migration Plan

Publishing with the new `lsg` writes version 3 files and pages that link the fixed manifest. Old `lsg-search-manifest-<revision>.json` files disappear with the rest of the old output. Hosts that read `data-revision` read the manifest's `revision` field instead.

## Resolved Questions

### 1 - Where does the publication revision live once pages name a fixed manifest?

Decided by the agent without codekiln: only inside the manifest. Any revision in the page's markup, even in a query string, would change every page on every publication, which is the problem this change removes.

### 2 - How does a reader avoid a stale manifest at a fixed address?

Decided by the agent without codekiln: the loader keeps fetching with `cache: 'no-cache'`, which revalidates the manifest on every load, and it forgets the manifest and every file it named as soon as a named file is missing or fails its digest check. Revalidation costs one conditional request per load, the digest names already keep every other file correct in any cache, and the recovery path reuses the dialog's existing Retry state instead of adding a new one.

### 3 - How should shards name blocks so another page's edit leaves them unchanged?

Decided by the agent without codekiln: as a page number in title order and a block number within that page. It meets the issue's goal for block edits at the smallest size cost measured (19% more shard gzip, against 50% to 75% for numbers that would also survive added pages).

### 4 - Should shards also stay unchanged when a page is added, removed or renamed?

Decided by the agent without codekiln: not in this change. Every stable page number measured made shards at least 50% larger, which would slow each keystroke's download; pages and chunks already stay unchanged in that case, so the remaining cost is about 5 MB of shards per such publication on the encode garden.

### 5 - How are record chunks grouped so an edit stays inside its own chunks?

Decided by the agent without codekiln: by segments that start at blocks whose digest is a multiple of 64. Grouping by whole pages would cascade within a group when a page grows; cut points chosen by each block's own digest move only when that block changes. An average of 64 records keeps most segments under the 256-record limit and makes each row's chunk smaller.

### 6 - Where do the per-page block counts go?

Decided by the agent without codekiln: into the manifest. The manifest changes with every publication already, and moving the counts keeps the title index unchanged across block edits.

### 7 - Does the larger manifest slow the first open?

Decided by the agent without codekiln: it did by about 70 ms on the throttled phone profile until the loader fetched the title index in parallel with the engine import; with that change first page results arrive about 100 ms sooner than on main, so the larger manifest is kept and the parallel fetch is part of this change.

## Open Questions

None.
