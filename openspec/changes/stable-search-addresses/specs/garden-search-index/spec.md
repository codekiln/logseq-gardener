## MODIFIED Requirements

### Requirement: Incremental index update

Adding, removing or changing a block on one page SHALL change only the records of that page and the index entries of the words its old and new records contain. Shards and chunks whose content is unchanged SHALL keep the same file name and bytes after a rebuild, and the title index SHALL stay unchanged while no page, title or alias changes.

#### Scenario: One page edited

- **WHEN** one block of one page changes from `alpha` to `beta` and the site is rebuilt
- **THEN** of the search files, only the manifest, the shards for `al` and `be`, and the record chunk holding that block differ from the previous build

#### Scenario: One block added

- **WHEN** a block reading `An added block for a churn test.` is appended to one page of a garden and the site is rebuilt
- **THEN** besides the manifest, only the chunks holding that page's records and the shards whose keys start the words `an`, `added`, `block`, `for`, `a`, `churn` and `test` differ, and every other page's HTML is unchanged

#### Scenario: An edit moves a chunk boundary

- **WHEN** an edited block grows enough to move a chunk boundary
- **THEN** only the chunks of the edited block's segment differ, besides the manifest and the affected shards
