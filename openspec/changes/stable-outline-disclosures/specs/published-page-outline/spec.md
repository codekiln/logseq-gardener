## MODIFIED Requirements

### Requirement: Collapse and expand any block

On a page with an outline, readers SHALL be able to collapse and expand any block that has children by its own stable, focusable arrow or by the strip over its children's guide line. Each arrow SHALL have a target at least 24px square directly left of and vertically centered on its owning bullet, including heading bullets. Moving continuously from a nested block's text through its gutter to its arrow SHALL retain that arrow's owner. Closed arrows SHALL remain visible without hover; keyboard focus and touch pointers SHALL expose usable controls before hover. Arrow activation SHALL toggle only its owner and update its accessible expanded state. The initializer SHALL be idempotent and support newly inserted page and journal-feed outlines; native authored disclosures SHALL remain usable without JavaScript. A collapsed block SHALL show its bullet on a filled disc. The state SHALL last only in the open tab. Navigating to a block anchor inside a reader-collapsed block SHALL expand its collapsed ancestors.

#### Scenario: Collapse an ordinary block
- **WHEN** a reader clicks the arrow on `Important limitations` on `1Password/Account/Service`, then clicks it again
- **THEN** its children are hidden and its bullet shows the filled disc, and the second click shows the children again

#### Scenario: Linked block inside a folded block
- **WHEN** a reader has collapsed a block and then follows a link to a block anchor inside it
- **THEN** the collapsed block expands and the target is visible

#### Scenario: Stable nested pointer target
- **WHEN** a reader moves continuously from a nested heading through its gutter to its arrow
- **THEN** the same block owns the arrow throughout and one click toggles only that block

#### Scenario: Keyboard and touch before hover
- **WHEN** a reader first reaches an expanded or closed parent using a keyboard or touch pointer
- **THEN** its own focusable control is available, a closed arrow is already visible, and native activation updates its expanded state

#### Scenario: Heading alignment and replacement content
- **WHEN** plain blocks and headings are initialized, or a host inserts replacement page or journal content and initializes it repeatedly
- **THEN** each expandable block has one arrow at least 24px square centered on its own bullet, leaf blocks have none, and existing zoom, fragment and collapse behavior remains available
