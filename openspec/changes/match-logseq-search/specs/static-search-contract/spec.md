## MODIFIED Requirements

### Requirement: Versioned deterministic publication

Manifest, title index, shards and record chunks SHALL declare format version 2. The manifest SHALL carry the publication revision and list every shard and chunk file by content digest. Every generated page SHALL advertise its expected manifest and revision without eagerly loading search data. The pure validator SHALL reject unsupported versions, revision or digest mismatch and malformed record shape. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the index.

#### Scenario: Cached corpus from another publication

- **WHEN** manifest, title index, shard or page revisions or digests disagree
- **THEN** validation fails and no query results are accepted

#### Scenario: Version 1 data

- **WHEN** the controller receives a manifest that declares format version 1
- **THEN** validation fails with the reload message and no results are shown

### Requirement: Bounded pure query evaluation

The standalone evaluator SHALL normalize lowercase NFC text and collapse whitespace, bound query length, term count, results per group and plain-text snippet length, and return deterministic order. Empty queries SHALL return no page or block results; malformed and oversized queries SHALL report an explicit bounded error.

#### Scenario: Repeated equivalent Unicode queries

- **WHEN** equivalent composed or decomposed case variants are evaluated against the same index
- **THEN** the entire ordered result list is identical

## ADDED Requirements

### Requirement: Split search index

The publisher SHALL write a title index holding every page record, content shards that map normalized words to block record numbers, and record chunks that hold block records. A shard SHALL hold the words that start with the same two characters, and SHALL split by the next character while it exceeds the shard budget. A one-character word SHALL be stored in the shard of its single character. A record chunk SHALL hold at most 256 consecutive block records, SHALL close at each multiple of 256 records and before the record that would take its gzip size past the chunk budget, and the manifest SHALL list each chunk's first block number.

#### Scenario: Large shard splits

- **WHEN** the words starting with `th` exceed the shard budget
- **THEN** the manifest lists shards `thi`, `tho` and the others in place of `th`, and each stays within the budget

#### Scenario: Long blocks split a chunk

- **WHEN** 256 consecutive block records exceed the chunk budget gzipped
- **THEN** the publisher writes them as several chunks, each within the budget unless it holds a single record, and the browser finds each block's chunk from the listed first block numbers

### Requirement: Fuzzy page title matching

A page or alias SHALL match when the query's characters, ignoring spaces, brackets, slashes, underscores and parentheses, appear in order in its normalized title. Matches SHALL rank as exact title, exact alias, title or alias prefix, then by Logseq's fuzzy page score, ties by normalized title and route. An alias match SHALL return one result for its page, marked with the alias.

#### Scenario: Typo still finds the page

- **WHEN** a reader searches `tmx`
- **THEN** `tmux` is the first page result

#### Scenario: Alias result

- **WHEN** a reader searches `tmux.conf.local`
- **THEN** the page `tmux/Config/local` appears once, marked as matched through its alias `tmux.conf.local`

### Requirement: Block word-prefix matching

A block SHALL match when every query word is the start of some word in the block's text. Matches SHALL rank by the number of query words that equal a whole word in the block, then by page title, then by position. The evaluator SHALL report the full match count of each group even when it returns at most 100 results.

#### Scenario: Two-word block query

- **WHEN** a reader searches `logseq query`
- **THEN** blocks containing words starting with `logseq` and with `query` match, and blocks containing only one of them do not

#### Scenario: Count beyond the result limit

- **WHEN** 2,000 blocks match `tmux`
- **THEN** the Blocks group reports a count of 2,000 and returns 100 results
