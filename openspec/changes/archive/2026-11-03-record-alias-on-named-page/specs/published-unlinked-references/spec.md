## ADDED Requirements

### Requirement: Look for every name of a page's alias group

A page's Unlinked References SHALL look for the page's title and every name its `alias` row lists, which includes the other names of its alias group, as Logseq's `get-page-unlinked-references` looks for `get-page-alias-names`.

#### Scenario: A page another page lists as its alias
- **WHEN** `pages/brew.md` holds `alias:: [[Homebrew]]` and a block elsewhere says `install it with brew`
- **THEN** the block appears in the Unlinked References of `Homebrew` as well as `brew`
