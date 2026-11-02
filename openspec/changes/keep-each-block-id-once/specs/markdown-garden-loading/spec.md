## ADDED Requirements

### Requirement: Keep each block id on the block Logseq reads first

When more than one block of a loaded garden gives the same block UUID through `id::`, `custom-id::` or `custom_id::`, the block that Logseq's file parser reads first SHALL keep its id properties, and every other block with that UUID SHALL lose its `id`, `custom-id` and `custom_id` properties and keep the rest. The SDK SHALL read the files in Logseq's order: files whose path holds `journals/`, from the last path to the first, then files whose path holds `contents.`, then every other file by path; and within a file, its blocks in source order. A UUID SHALL match in any letter case. An id that is not a UUID SHALL stay on every block that gives it.

#### Scenario: Two pages give a block the same id

- **WHEN** `pages/json___5.md` and `pages/json___jsonc.md` each hold a block with `id:: 67c822b0-3271-41da-a14f-4f023a1f4556`
- **THEN** the block in `pages/json___5.md` keeps the id, the block in `pages/json___jsonc.md` has no `id` property, and a block embed of the id shows the block in `pages/json___5.md`

#### Scenario: A journal and a page give a block the same id

- **WHEN** `journals/2025_03_05.md`, `journals/2025_03_06.md` and `pages/a.md` each hold a block with the same id
- **THEN** the block in `journals/2025_03_06.md` keeps the id, and the other two blocks have no `id` property

#### Scenario: Two blocks of one page give the same id

- **WHEN** a page holds a block with `id::` and a later block with the same UUID in `custom_id::` and the property `kind:: note`
- **THEN** the first block keeps the id, and the later block keeps only the property `kind`
