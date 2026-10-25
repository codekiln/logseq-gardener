## MODIFIED Requirements

### Requirement: Keep disclosure and descendant navigation accessible

Disclosure controls SHALL support native keyboard expansion and collapse, visible focus, and readable wrapping on narrow screens. An authored closed disclosure SHALL show the block's bullet on Logseq's filled disc in place of the browser's disclosure triangle, and SHALL open without JavaScript. Navigating to a valid descendant fragment inside closed disclosures SHALL reveal the target by opening its containing disclosures, both on initial page load and later fragment navigation. Unrelated authored closed branches SHALL keep their state. Existing source content and recording controls SHALL remain usable through disclosure interaction.

#### Scenario: Keyboard and mobile interaction
- **WHEN** a reader focuses a closed summary at a narrow mobile width and activates it by keyboard
- **THEN** the disclosure opens with visible focus and readable wrapped content, and can be closed again

#### Scenario: Direct fragment and later fragment change
- **WHEN** a valid block anchor lies inside nested closed disclosures and is reached by initial URL or later fragment navigation
- **THEN** its containing disclosures open and the target becomes reachable while unrelated closed branches stay closed

#### Scenario: Closed bullet in place of the triangle
- **WHEN** a reader opens `rw-cli`, which has four authored collapsed blocks
- **THEN** each shows its bullet on a filled disc, no native disclosure triangle is visible, and each opens by click or keyboard with JavaScript turned off
