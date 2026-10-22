## Purpose

Define authored outline disclosure state, preserved publication safety, and accessible descendant navigation.

## Requirements

### Requirement: Honor authored outline collapse state

The publisher SHALL render an outline parent with an attached parsed `collapsed:: true` property and descendants as an initially closed native disclosure. The parent label SHALL remain visible in its summary and its complete descendant subtree SHALL remain in the generated document. False or absent collapse flags SHALL retain expanded rendering. Nested authored states SHALL be independent. Following siblings SHALL remain outside the disclosure. Leaf items SHALL remain visible without empty disclosures. Page-level properties and property-like code or literal text SHALL not control an unrelated outline's collapse state.

#### Scenario: True parent with following sibling
- **WHEN** Dialogue has an attached true collapse property, a transcript child, and a following sibling
- **THEN** Dialogue renders a closed disclosure containing its transcript, and the following sibling remains visible outside it

#### Scenario: False or absent property
- **WHEN** outline parents have false or absent collapse flags
- **THEN** their descendants retain expanded rendering

#### Scenario: Independent nested collapse
- **WHEN** a closed authored parent contains another closed authored parent
- **THEN** opening the outer disclosure reveals the nested summary while its descendants remain closed until opened separately

#### Scenario: Leaf and literal metadata
- **WHEN** a leaf has a true flag, or property-like text appears at page level or in a code example
- **THEN** the leaf remains visible without an empty disclosure and the unrelated outline state is unchanged

### Requirement: Preserve content and publication safety

Collapsed outlines SHALL retain existing source order, escaping, supported rich content, media, selected transclusions, valid unique IDs, and original block-reference destinations. Summary markup SHALL remain valid when parent content includes flow constructs. Source visibility and whole-page private-property checks SHALL apply before rendering regardless of authored collapse state. Collapsed selected transclusions SHALL preserve their own source context and existing expansion limits.

#### Scenario: Media and selected transclusion below a closed parent
- **WHEN** a selected closed parent contains supported media and a selected page or block embed
- **THEN** opening it reveals the supported rendered content with correct asset sources and valid IDs

#### Scenario: Rich parent content
- **WHEN** the parent label contains supported formatting, links, or an embed
- **THEN** the disclosure retains its content safely with valid summary and body markup

#### Scenario: Private descendant
- **WHEN** a selected page contains a private property below an authored closed parent
- **THEN** the existing visibility policy withholds the whole page and its assets

### Requirement: Keep disclosure and descendant navigation accessible

Disclosure controls SHALL support native keyboard expansion and collapse, visible focus, and readable wrapping on narrow screens. Navigating to a valid descendant fragment inside closed disclosures SHALL reveal the target by opening its containing disclosures, both on initial page load and later fragment navigation. Unrelated authored closed branches SHALL keep their state. Existing source content and recording controls SHALL remain usable through disclosure interaction.

#### Scenario: Keyboard and mobile interaction
- **WHEN** a reader focuses a closed summary at a narrow mobile width and activates it by keyboard
- **THEN** the disclosure opens with visible focus and readable wrapped content, and can be closed again

#### Scenario: Direct fragment and later fragment change
- **WHEN** a valid block anchor lies inside nested closed disclosures and is reached by initial URL or later fragment navigation
- **THEN** its containing disclosures open and the target becomes reachable while unrelated closed branches stay closed
