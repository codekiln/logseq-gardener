## Purpose

Show published readers, like Logseq's Unlinked References section, the blocks that mention the current page by name without linking to it.

## Requirements

### Requirement: Show unlinked references

Every published page SHALL end with a collapsed `Unlinked References` section. Opening it SHALL load the page's unlinked mentions from the SDK index and change the heading to `N Unlinked References`, or `1 Unlinked Reference`, where N counts blocks. Results SHALL use the linked-reference group markup, order and styles, without breadcrumbs and without filters. A page with no mentions SHALL show `0 Unlinked References` and no groups once opened.

#### Scenario: Count after opening
- **WHEN** a reader opens Unlinked References on `tmux`
- **THEN** the heading reads `856 Unlinked References` and the groups start with the block that links `[[My/AI/Rule/Dev/Workflow/Git Worktree PR/Tmux]]`

#### Scenario: Collapsed by default
- **WHEN** a reader opens `mise`
- **THEN** Unlinked References shows only its header, reading `Unlinked References`
