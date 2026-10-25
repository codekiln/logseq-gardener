## ADDED Requirements

### Requirement: Show pages tagged with a page

A published page that at least one published page tags SHALL show a foldable section, open by default, headed `Pages tagged with "<name>"` with the lower-cased page name, listing a link to each tagging page in the SDK's order. The heading SHALL be 16px, weight 700 at 50% opacity; the list SHALL use circle markers with `8px 0 0 19.2px` margin, rows with `4px 0` margins, and links in the link colour with no brackets. A page that no page tags SHALL have no section.

#### Scenario: Term
- **WHEN** a reader opens `Term`
- **THEN** the section reads `Pages tagged with "term"` and lists 178 pages starting with `1Password/Dev/CLI/Secret Reference`

#### Scenario: Lower-cased name
- **WHEN** a reader opens `Claude/Code`
- **THEN** the heading reads `Pages tagged with "claude/code"` above 38 pages
