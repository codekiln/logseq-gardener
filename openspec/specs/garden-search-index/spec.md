## Purpose

Give the CLI, the language server and the publisher one search index over a garden's published pages and blocks, so every tool returns the same results for the same query.

## Requirements

### Requirement: Search records from the garden index

The SDK SHALL build one page record per published page (title, uniquely resolved aliases, route) and one block record per searchable block (plain text, page, at most the three nearest parent texts, destination). The privacy rules of the static search contract SHALL decide which text becomes a record, so the SDK and the published index hold the same records.

#### Scenario: Block nested five levels deep

- **WHEN** a searchable block has five ancestors
- **THEN** its record holds the texts of its three nearest ancestors, nearest last, and its page title

#### Scenario: Private reference in a public block

- **WHEN** a published block references a page that is not published
- **THEN** the block record holds no text from the unpublished page's name

### Requirement: Word index

The SDK SHALL index every block record under each distinct normalized word it contains, using the static search contract's normalization. A word SHALL be a run of letters, digits or marks; other characters SHALL separate words. The index SHALL list, for each word, the block records that contain it in record order.

#### Scenario: Words split at punctuation

- **WHEN** a block's text is `tmux.conf: set -g mouse on`
- **THEN** the index lists the block under `tmux`, `conf`, `set`, `g`, `mouse` and `on`

### Requirement: Rust query matches the browser engine

The SDK SHALL answer a query with the same grouped, ordered results the published browser engine returns for the same records. A shared fixture of queries and expected results SHALL run against both and fail the build when they differ.

#### Scenario: Shared fixture

- **WHEN** the fixture queries `tmux`, `tmx`, `logseq query` and `tmux.conf.local` run in Rust and in the browser engine over the same garden
- **THEN** both return identical page and block result lists

### Requirement: Incremental index update

Adding, removing or changing a block on one page SHALL change only the records of that page and the index entries of the words its old and new records contain. Adding, removing or renaming a page SHALL change only the title index, the manifest, the chunks that hold that page's records and the shards of the words its records contain, unless the page shares a slot with a page whose route sorts after it or the slot count changes. Shards and chunks whose content is unchanged SHALL keep the same file name and bytes after a rebuild, and the title index SHALL stay unchanged while no page, title or alias changes.

#### Scenario: One page edited

- **WHEN** one block of one page changes from `alpha` to `beta` and the site is rebuilt
- **THEN** of the search files, only the manifest, the shards for `al` and `be`, and the record chunk holding that block differ from the previous build

#### Scenario: One block added

- **WHEN** a block reading `An added block for a churn test.` is appended to one page of a garden and the site is rebuilt
- **THEN** besides the manifest, only the chunks holding that page's records and the shards whose keys start the words `an`, `added`, `block`, `for`, `a`, `churn` and `test` differ, and every other page's HTML is unchanged

#### Scenario: An edit moves a chunk boundary

- **WHEN** an edited block grows enough to move a chunk boundary
- **THEN** only the chunks of the edited block's segment differ, besides the manifest and the affected shards

#### Scenario: One page added

- **WHEN** a page reading `An added page about zymurgy.` is added to a garden, its slot holds no other page, and the site is rebuilt
- **THEN** every shard whose key starts none of the words `an`, `added`, `page`, `about` and `zymurgy` keeps its name and bytes

#### Scenario: A page added before another in its slot

- **WHEN** an added page shares a slot with a page whose route sorts after it
- **THEN** that page's block numbers in the slot move by the added page's block count, and the shards of that page's words change as well
