## MODIFIED Requirements

### Requirement: Render each listed block once per build

The publisher SHALL render a referring block's copy and its breadcrumb at most once per build at a given open depth, and SHALL reuse them wherever the block is listed: in each target's Linked References, in Unlinked References and in query results. Each listing SHALL add only what depends on it: the row number of each referring bullet in that page's reference data and the link each copied bullet opens. A diagnostic a copy produces SHALL be reported once, under the file the copied block comes from, and SHALL be left out when that file's own page reports the same diagnostic. The published files SHALL be byte for byte identical to rendering the copy separately for each listing.

#### Scenario: Block that links to several pages
- **WHEN** one block links to `tmux`, `mise` and `nvim`
- **THEN** its copy renders once, and the Linked References of each of the three pages show it with that page's row number and bullet link

#### Scenario: Whole-garden output
- **WHEN** the encode garden is published with `--journals` and `--search` before and after this change
- **THEN** both builds write the same files with the same bytes

#### Scenario: Listed block with a missing link
- **WHEN** a block in `pages/Notes___Other.md` links to `Notes/Start` and to a missing page `Absent`
- **THEN** the report has one diagnostic for `Absent`, with source `pages/Notes___Other.md`, and none under `pages/Notes___Start.md`
