## Purpose

Show published readers, like Logseq's Unlinked References section, the blocks that mention the current page by name without linking to it.
## Requirements
### Requirement: Show unlinked references

Every published page SHALL end with a collapsed `Unlinked References` section. Opening it SHALL load the page's unlinked mentions from the SDK index and change the heading to `N Unlinked References`, or `1 Unlinked Reference`, where N counts blocks. Results SHALL use the linked-reference group markup, order and styles, without breadcrumbs and without filters. A page with no mentions SHALL show `0 Unlinked References` and no groups once opened. A reference-only page SHALL list at most the groups of one fragment file, about 50 pages; when it has more, the heading SHALL still count every block and the list SHALL end with `Showing the first N of M pages`.

#### Scenario: Count after opening
- **WHEN** a reader opens Unlinked References on `tmux`
- **THEN** the heading reads `856 Unlinked References` and the groups start with the block that links `[[My/AI/Rule/Dev/Workflow/Git Worktree PR/Tmux]]`

#### Scenario: Collapsed by default
- **WHEN** a reader opens `mise`
- **THEN** Unlinked References shows only its header, reading `Unlinked References`

#### Scenario: Reference-only page with many mentions

- **WHEN** a reader opens Unlinked References on a reference-only page whose mentions span 812 pages and 1,904 blocks
- **THEN** the heading reads `1904 Unlinked References`, the groups of the first fragment file show, and the list ends with `Showing the first 50 of 812 pages`, where 50 is the number of groups that file holds

### Requirement: Look for every name of a page's alias group

A page's Unlinked References SHALL look for the page's title and every name its `alias` row lists, which includes the other names of its alias group, as Logseq's `get-page-unlinked-references` looks for `get-page-alias-names`.

#### Scenario: A page another page lists as its alias
- **WHEN** `pages/brew.md` holds `alias:: [[Homebrew]]` and a block elsewhere says `install it with brew`
- **THEN** the block appears in the Unlinked References of `Homebrew` as well as `brew`
