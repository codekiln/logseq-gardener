## MODIFIED Requirements

### Requirement: Versioned deterministic publication

Manifest, title index, shards and record chunks SHALL declare format version 3. The manifest SHALL be written at the fixed address `lsg-search-manifest.json`, SHALL carry the publication revision and each page's number of block records in title index order, and SHALL list the title index, every shard and every chunk by content digest. Every generated page SHALL name the manifest's fixed address, without a revision, and SHALL load no search data eagerly. The pure validator SHALL reject unsupported versions, page counts that do not add up to the block total or disagree with the title index, digest mismatch and malformed record shape. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the index.

#### Scenario: Cached corpus from another publication

- **WHEN** a title index, shard or chunk's digest disagrees with the manifest
- **THEN** validation fails and no query results are accepted

#### Scenario: Version 2 data

- **WHEN** the controller receives a manifest that declares format version 2
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Pages name the fixed manifest

- **WHEN** a garden is published with search
- **THEN** every page links `lsg-search-manifest.json` with no revision in its markup, and the manifest's `revision` field identifies the publication

### Requirement: Split search index

The publisher SHALL write a title index holding every page record, content shards that map normalized words to the blocks that contain them, and record chunks that hold block records. A shard SHALL name each block by its page number in title index order and its number among that page's records. A shard SHALL hold the words that start with the same two characters, and SHALL split by the next character while it exceeds the shard budget. A one-character word SHALL be stored in the shard of its single character. A block whose page route and fragment digest is a multiple of 64 SHALL start a new segment of records; within a segment, a record chunk SHALL hold at most 256 consecutive block records, SHALL close at each multiple of 256 records from the segment's start and before the record that would take its gzip size past the chunk budget. The manifest SHALL list each chunk's first block number.

#### Scenario: Large shard splits

- **WHEN** the words starting with `th` exceed the shard budget
- **THEN** the manifest lists shards `thi`, `tho` and the others in place of `th`, and each stays within the budget

#### Scenario: Long blocks split a chunk

- **WHEN** 256 consecutive block records of one segment exceed the chunk budget gzipped
- **THEN** the publisher writes them as several chunks, each within the budget unless it holds a single record, and the browser finds each block's chunk from the listed first block numbers

#### Scenario: Shards name blocks by page

- **WHEN** a word appears in the second and fifth records of the fourth page and in the first record of the ninth page
- **THEN** its shard entry is `[3, 1, 0, 3, 5, 0]`, and the engine reads it as the same three blocks
