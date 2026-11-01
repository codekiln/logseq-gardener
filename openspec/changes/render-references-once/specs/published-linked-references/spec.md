## ADDED Requirements

### Requirement: Render each listed block once per build

The publisher SHALL render a referring block's copy and its breadcrumb at most once per build at a given open depth, and SHALL reuse them wherever the block is listed: in each target's Linked References, in Unlinked References and in query results. Each listing SHALL add only what depends on it: the row number of each referring bullet in that page's reference data and the link each copied bullet opens. The diagnostics a copy produces SHALL be reported for every page that lists it, as before. The published files SHALL be byte for byte identical to rendering the copy separately for each listing.

#### Scenario: Block that links to several pages
- **WHEN** one block links to `tmux`, `mise` and `nvim`
- **THEN** its copy renders once, and the Linked References of each of the three pages show it with that page's row number and bullet link

#### Scenario: Whole-garden output
- **WHEN** the encode garden is published with `--journals` and `--search` before and after this change
- **THEN** both builds write the same files with the same bytes
