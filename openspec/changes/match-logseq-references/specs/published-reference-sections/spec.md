## ADDED Requirements

### Requirement: Order the sections below the outline

Every published page SHALL show, after its outline, the sections that apply in this order: Pages tagged with, Linked References, Hierarchy, Unlinked References. Each section header SHALL be a foldable row with a 14px chevron and the heading. Journal pages SHALL have no Hierarchy section, as today.

#### Scenario: Order on mise
- **WHEN** a reader opens `mise`
- **THEN** the sections appear as Pages tagged with, Linked References, Hierarchy, Unlinked References

### Requirement: Load section bodies when opened

A page's HTML SHALL hold each reference section's header and counts, and no groups. The publisher SHALL write linked and unlinked groups to fragment files under the page's output, in group order with about 50 groups per file, plus a data file for filtering. Opening a section SHALL fetch its first file and fetch later files as the reader nears the end. Without scripts, each section SHALL offer a link to its first fragment, and each fragment SHALL be a readable page linking to the next. Fragments SHALL follow the same rendering, anchor and withholding rules as page content. Fragment navigation to a referring block SHALL still open the block on its source page.

#### Scenario: First paint independent of references
- **WHEN** the encode garden is built
- **THEN** `Answer/AI`, `note/my`, `mise` and `Claude/Code` each send at most 100 KB before their title, and no reference group is in their HTML

#### Scenario: Scripts off
- **WHEN** a reader with scripts disabled opens Linked References on `mise`
- **THEN** a link opens the first fragment, which shows its groups and links to the next fragment

#### Scenario: Swapped content
- **WHEN** a host replaces page content and initializes the references script again
- **THEN** the new sections load their fragments and no handler is registered twice

### Requirement: Keep reference sections within budgets

The `[references]` table in `scripts/parity/budgets.toml` SHALL set: `references.js` at most 6 KB minified, references styles at most 1 KB gzip on every page, each fragment file at most 100 KB gzip, first group visible at most 300 ms on desktop and 1,500 ms on the phone profile, peak phone heap at most 20 MB after opening both reference sections on `mise` and scrolling to the end, a filter change on `mise` at most 50 ms on desktop, and whole-garden build time at most 25% above the build before this change. `parity:budgets` SHALL check the size budgets in CI, and `parity:perf` SHALL mark the others.

#### Scenario: Oversized fragment
- **WHEN** a fragment file exceeds 100 KB gzip
- **THEN** `parity:budgets` fails and names the file
