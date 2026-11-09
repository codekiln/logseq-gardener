## MODIFIED Requirements

### Requirement: Versioned deterministic publication

Manifest, title index, shards and record chunks SHALL declare format version 5. The manifest SHALL be written at the fixed address `lsg-search-manifest.json`, SHALL carry the publication revision, the page URL scheme (`digest` or `readable`) and each page's number of block records in title index order, and SHALL list the title index, every shard and every chunk by content digest. Every generated page SHALL name the manifest's fixed address, without a revision, and SHALL load no search data eagerly. The pure validator SHALL reject unsupported versions, page counts that do not add up to the block total or disagree with the title index, digest mismatch and malformed record shape. A title index entry SHALL carry its page's route only when that route differs from the route its title gives under the manifest's URL scheme; the browser SHALL compute the route otherwise. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the index.

#### Scenario: Cached corpus from another publication

- **WHEN** a title index, shard or chunk's digest disagrees with the manifest
- **THEN** validation fails and no query results are accepted

#### Scenario: Version 2 data

- **WHEN** the controller receives a manifest that declares format version 2
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Version 3 data

- **WHEN** the controller receives a manifest that declares format version 3
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Version 4 data

- **WHEN** the controller receives a manifest that declares format version 4
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Pages name the fixed manifest

- **WHEN** a garden is published with search
- **THEN** every page links `lsg-search-manifest.json` with no revision in its markup, and the manifest's `revision` field identifies the publication

#### Scenario: Computable routes

- **WHEN** a garden is published with the default page URLs and search
- **THEN** the entry for `Deno Deploy` carries no route, search opens `p-<digest of "Deno Deploy">.html` for it, and an entry whose page was published at another route carries that route

### Requirement: Split search index

The publisher SHALL write a title index holding every page record, content shards that map normalized words to the blocks that contain them, and record chunks that hold block records. A shard SHALL name each block by its page's slot and its number among that slot's blocks, as the slot requirement defines, and SHALL write each word's blocks as one string of base-32 numbers. A shard SHALL hold the words that start with the same two characters, and SHALL split by the next character while it exceeds the shard budget. A one-character word SHALL be stored in the shard of its single character. A block whose page route and fragment digest is a multiple of 64 SHALL start a new segment of records; within a segment, a record chunk SHALL hold at most 256 consecutive block records, SHALL close at each multiple of 256 records from the segment's start and before the record that would take its gzip size past the chunk budget. The manifest SHALL list each chunk's first block number.

#### Scenario: Large shard splits

- **WHEN** the words starting with `th` exceed the shard budget
- **THEN** the manifest lists shards `thi`, `tho` and the others in place of `th`, and each stays within the budget

#### Scenario: Long blocks split a chunk

- **WHEN** 256 consecutive block records of one segment exceed the chunk budget gzipped
- **THEN** the publisher writes them as several chunks, each within the budget unless it holds a single record, and the browser finds each block's chunk from the listed first block numbers

#### Scenario: Shards name blocks by page

- **WHEN** a word appears in the second and fifth records of the only page in slot 3 and in the first record of the only page in slot 40
- **THEN** its shard entry is `"3103x50"`, the numbers 3, 1, 0, 3, 37 and 0, and the engine reads it as the same three blocks

## ADDED Requirements

### Requirement: Page slots

Each page with at least one block record SHALL have a slot: its route's 64-bit FNV-1a digest modulo the slot count, which SHALL be the smallest power of two at least four times the number of such pages. A slot's blocks SHALL be its pages' block records, page by page in route byte order and in record order within a page. The engine SHALL compute the same slots from the title index and the manifest's block counts.

#### Scenario: Slot count

- **WHEN** a garden has 9,186 pages with block records
- **THEN** there are 65,536 slots, and they stay 65,536 until such pages number more than 16,384

#### Scenario: Two pages share a slot

- **WHEN** pages `p-a.html` with 3 blocks and `p-b.html` with 2 blocks have the same slot
- **THEN** the slot's blocks 0 to 2 are `p-a.html`'s and its blocks 3 and 4 are `p-b.html`'s
