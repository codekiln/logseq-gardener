## Purpose

Define opt-in selected-source search artifacts and deterministic query evaluation.

## Requirements

### Requirement: Selected-source search corpus

The publisher SHALL provide opt-in deterministic search artifacts built from selected visible original source. Stored titles, reference-only page titles and uniquely resolved aliases SHALL participate; generated namespace titles and property-content search SHALL be outside version1. Unresolved or ambiguous reference targets, hidden or excluded pages, macros, query programs, raw HTML, asset metadata and copied/transcluded text MUST contribute no terms or snippets. Canonical-title body exclusions SHALL retain title results and remove body records.

#### Scenario: Public source refers to private content

- **WHEN** a selected public page references an excluded or private page and embeds a selected target
- **THEN** the index contains only safe original referring text and the target's own original content, without unavailable reference names or copied embed content

#### Scenario: Host excludes a generated-content slot page

- **WHEN** a host excludes a selected page body
- **THEN** its title remains searchable and its content records are absent

#### Scenario: Reference-only title

- **WHEN** a published block links `[[Deno Deploy]]`, which no file holds
- **THEN** the title index holds `Deno Deploy` with no block records, and the referring block's record holds the words `Deno Deploy`

### Requirement: Stable original destinations

Every search result SHALL identify its selected source page and an output-relative route. Content records SHALL use an actually emitted original anchor, preferring globally unique UUIDs and otherwise source offsets. Existing fragment reveal behavior SHALL open collapsed ancestors.

#### Scenario: Searchable child under collapsed parent

- **WHEN** a matching original child is nested inside collapsed source
- **THEN** its result points to an emitted anchor that the fragment reveal helper can expose

### Requirement: Versioned deterministic publication

Manifest, title index, shards and record chunks SHALL declare format version 4. The manifest SHALL be written at the fixed address `lsg-search-manifest.json`, SHALL carry the publication revision, the page URL scheme (`digest` or `readable`) and each page's number of block records in title index order, and SHALL list the title index, every shard and every chunk by content digest. Every generated page SHALL name the manifest's fixed address, without a revision, and SHALL load no search data eagerly. The pure validator SHALL reject unsupported versions, page counts that do not add up to the block total or disagree with the title index, digest mismatch and malformed record shape. A title index entry SHALL carry its page's route only when that route differs from the route its title gives under the manifest's URL scheme; the browser SHALL compute the route otherwise. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the index.

#### Scenario: Cached corpus from another publication

- **WHEN** a title index, shard or chunk's digest disagrees with the manifest
- **THEN** validation fails and no query results are accepted

#### Scenario: Version 2 data

- **WHEN** the controller receives a manifest that declares format version 2
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Version 3 data

- **WHEN** the controller receives a manifest that declares format version 3
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Pages name the fixed manifest

- **WHEN** a garden is published with search
- **THEN** every page links `lsg-search-manifest.json` with no revision in its markup, and the manifest's `revision` field identifies the publication

#### Scenario: Computable routes

- **WHEN** a garden is published with the default page URLs and search
- **THEN** the entry for `Deno Deploy` carries no route, search opens `p-<digest of "Deno Deploy">.html` for it, and an entry whose page was published at another route carries that route

### Requirement: Bounded pure query evaluation

The standalone evaluator SHALL normalize lowercase NFC text and collapse whitespace, bound query length, term count, results per group and plain-text snippet length, and return deterministic order. Empty queries SHALL return no page or block results; malformed and oversized queries SHALL report an explicit bounded error.

#### Scenario: Repeated equivalent Unicode queries

- **WHEN** equivalent composed or decomposed case variants are evaluated against the same index
- **THEN** the entire ordered result list is identical

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
