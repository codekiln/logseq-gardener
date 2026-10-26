## MODIFIED Requirements

### Requirement: Index authored references from published sources

The publisher SHALL take linked references from the SDK reference index, which derives them only from selected visible original documents. Page identities SHALL resolve through existing selected-title and alias rules. Each referring block SHALL count once per target, and self-page references SHALL be omitted. Parsed page links, tags, embed page targets, and authored page-reference property values SHALL supply references, and a reference to a namespace child SHALL also count for each of its namespace ancestors; property names, literal code, comments, external links, generated navigation, and pure block UUID references SHALL not manufacture page backlinks. Missing or ambiguous identities SHALL supply no result or filter option. Safe generated namespace destinations SHALL receive the references of their namespace children and MAY receive authored references without entering source alias, embed, or homepage lookup.

#### Scenario: Repeated references and aliases
- **WHEN** one source block links twice to a selected target and once to its unique alias
- **THEN** it contributes one referring block for that target

#### Scenario: Namespace child
- **WHEN** a block links to `[[mise/Config]]`
- **THEN** the block appears in the linked references of `mise` and counts toward its total

#### Scenario: Tags and authored properties
- **WHEN** published source refers to a selected target through a tag or a page-reference property value
- **THEN** the appropriate source block supplies a backlink, and leading properties appear as the page's property block

#### Scenario: Literal and generated content
- **WHEN** a target appears only in literal code, comments, generated hierarchy links, a property name, or a pure block UUID reference
- **THEN** those occurrences supply no page backlink

#### Scenario: Withheld source and identities
- **WHEN** private or excluded sources contain unique text, references, or related-page names
- **THEN** none of their text, names, counts, filter options, or metadata enters backlink output, fragments or scripts

### Requirement: Show grouped context and explicit counts

A page with at least one linked reference SHALL have a Linked References section whose heading reads `N Linked References`, or `1 Linked Reference`, where N counts referring blocks; with any filter active it SHALL read `F of N Linked References`. The heading SHALL be 16px at weight 500 and remain readable while collapsed. A page with no linked references SHALL have no Linked References section.

Groups SHALL be one per referring page: journals first, newest first, then other pages by lower-cased title. Within a group, blocks whose parent is the page SHALL come first, then document order. Each group SHALL be a foldable card, open by default, with the surface background, 4px radius, 16px padding and `8px 0` margin, headed by a plain link to the page at 16px and weight 400 with no brackets, followed by a muted `Alias` label when the page is an alias of the target. Each referring block SHALL render with bullets and its children, as in the page outline. Each source block SHALL appear once across overlapping referring outlines within its group. Copies SHALL not duplicate original anchor IDs, and labels and source-relative media SHALL render normally.

A referring block below the top level SHALL have a breadcrumb of its nearest three ancestors, with `⋯` replacing older ones, excluding the page. Each crumb SHALL be the ancestor's first line through the inline renderer and SHALL link to the ancestor's anchor on its page. Crumbs SHALL be separated by a chevron icon at 50% opacity, at 16px body text with `8px 0 8px 24px` margin. A referring property block SHALL show its safe displayed rows that refer to the target.

#### Scenario: Count text
- **WHEN** a reader opens `tmux`
- **THEN** the heading reads `795 Linked References`

#### Scenario: Zero references
- **WHEN** a reader opens `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter`
- **THEN** the page has no Linked References section

#### Scenario: Journal order
- **WHEN** a reader opens Linked References on `mise`
- **THEN** the first groups are `2026-10-19 Mon`, `2026-10-04 Sun` and `2026-09-10 Thu`, and journal groups precede every other group

#### Scenario: Shortened breadcrumb
- **WHEN** a referring block on `mise` has eight ancestors
- **THEN** its breadcrumb shows `⋯` followed by its three nearest ancestors with their inline markup

#### Scenario: Referring block navigation
- **WHEN** a reader follows a crumb or a referring block's destination
- **THEN** the browser opens that block on its original page, and fragment navigation opens only the containing disclosures

#### Scenario: Repeated context and media
- **WHEN** a source block is copied into linked-reference fragments
- **THEN** its labels and source-relative media retain normal rendering while original anchor IDs remain unique

### Requirement: Filter by related published pages

The Linked References header SHALL carry a `Filter` control with a funnel icon at 60% opacity with no filter, in the success colour when every filter is an include, the error colour when every filter is an exclude, and the warning colour when mixed. Activating it SHALL open a dialog with the title `Filter`, the hint `Click to include and shift-click to exclude. Click again to remove.`, `Includes:` and `Excludes:` rows of active filters, a search input with the placeholder `Search in linked pages`, and a chip for every related page with a superscript count, most frequent first. Click or Enter SHALL include a chip, shift-click or Shift+Enter SHALL exclude it, and activating an active chip SHALL remove it; each chip's accessible name SHALL state its state. Typing SHALL narrow the chips by fuzzy match. Chips and counts SHALL come from the blocks the current filter keeps.

A block SHALL match when its path references include every included page and none of the excluded pages; matching blocks keep their ancestors for context, and empty groups SHALL hide. A page's `filters::` property SHALL set the starting filter, already reflected in the first-paint heading and icon. Filter changes SHALL last for the visit through `sessionStorage` scoped to the site and page, and SHALL last for the page view when storage is unavailable. The dialog SHALL work with the keyboard and on narrow screens. Without scripts, the unfiltered fragments SHALL remain readable.

#### Scenario: Starting filter
- **WHEN** a reader opens `mise`
- **THEN** the heading reads `4 of 589 Linked References` and the funnel is green

#### Scenario: Options follow the filter
- **WHEN** a reader opens the filter dialog on `mise` with `Diataxis/How To` included
- **THEN** the chips start with `Diataxis` 4 and `AWS` 2, and after removing the include they start with `mise/Config` 106

#### Scenario: Combined inclusion and exclusion
- **WHEN** two related pages are included and a third is excluded
- **THEN** only references containing both includes and no exclude remain visible

#### Scenario: Keyboard exclude
- **WHEN** a keyboard user focuses a chip and presses Shift+Enter
- **THEN** the chip moves to the Excludes row and the funnel turns to the error colour

#### Scenario: No filtered results
- **WHEN** no reference matches the active filters
- **THEN** the heading reads `0 of N Linked References` and the section shows a readable no-results status

#### Scenario: Empty filters property
- **WHEN** a reader opens `McKinsey`, which has `filters:: {}`
- **THEN** the linked references start unfiltered

### Requirement: Honor graph reference disclosure settings

The publisher SHALL read integer `:ref/linked-references-collapsed-threshold` and `:ref/default-open-blocks-level` from the graph's top-level `logseq/config.edn` map, defaulting to 100 and 2. A Linked References section SHALL start collapsed when its referring-block count reaches the threshold; zero SHALL collapse every section while preserving its heading. Referring-block parents at or beyond the configured open depth SHALL start closed, and authored collapse SHALL also remain effective. Config reading SHALL be bounded, read-only, and ignore commented, discarded, string, or nested-map lookalikes. Missing or unusable settings SHALL retain defaults with useful diagnostics for unusable input.

#### Scenario: Threshold zero
- **WHEN** the graph sets the collapse threshold to zero
- **THEN** every Linked References section starts closed and shows its count

#### Scenario: Open depth and authored collapse
- **WHEN** open depth is two and a reference contains nested parents plus an authored collapsed parent
- **THEN** parents at depth two or deeper and the authored collapsed parent start closed while manual disclosure and fragment reveal still work

#### Scenario: Defaults and misleading config text
- **WHEN** config is absent or contains setting names only in comments, strings, discarded forms, or nested maps
- **THEN** absent top-level integer settings retain their defaults
