## ADDED Requirements

### Requirement: List the references of a page's alias group

A page's Linked References SHALL list, besides the blocks that refer to the page, the blocks that refer to any other selected page of its alias group, as Logseq's `get-page-referenced-blocks` does. The alias group SHALL be every page one or two alias links away in either direction, with the links Logseq's file parser records. Blocks on the page itself SHALL be left out, and each block SHALL be listed once. The heading's count and the journal feed's count of referring pages SHALL include these blocks. The group of a page in the alias group SHALL show the muted `Alias` label after its title: 14px, weight 500, 50% opacity. Queries SHALL keep matching a page by its own references only, as Logseq's `page-ref` rule does.

#### Scenario: Two pages that alias each other
- **WHEN** `pages/Notes___brew.md` holds `alias:: [[Notes/Homebrew]]`, `pages/Notes___Homebrew.md` holds `- A package manager, see [[Notes/brew]]`, and `pages/Notes___A.md` holds `- Install with [[Notes/brew]]` and `- Read about [[Notes/Homebrew]]`
- **THEN** both pages read `3 Linked References`, both list the two blocks of `Notes/A`, and each lists the other page's block in a group labelled `Alias`

#### Scenario: Encode garden
- **WHEN** a reader opens `Dolt`, whose `alias::` names the page `GitHub/dolthub/dolt`
- **THEN** its Linked References read `17 Linked References` in 12 groups, including `2026-03-12 Thu`, which refers only to `GitHub/dolthub/dolt`, as in Logseq's build
