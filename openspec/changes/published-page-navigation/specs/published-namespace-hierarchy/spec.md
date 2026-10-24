## ADDED Requirements

### Requirement: Distinguish namespace destinations

The publisher SHALL visibly label generated namespace pages and their entries in the complete page index and Hierarchy as Namespace. Links to generated namespace destinations SHALL have a distinction beyond color and an accessible Namespace name. Selected stored source pages SHALL retain ordinary page labels even when empty. Classification SHALL use only selected visible source routes: missing and ordinarily unselected source ancestors SHALL have the same Namespace presentation. Withheld and alias-conflicting ancestors SHALL retain their existing unavailable behavior. Full logical anchor text in the page index SHALL remain unchanged, with the status label outside the anchor. Source-only name, alias, embed, homepage, route, and count contracts SHALL remain intact.

#### Scenario: Generated and stored destinations
- **WHEN** a selected stored page needs a generated ancestor, and another selected stored page has an empty body
- **THEN** only the generated ancestor has the Namespace page/index/hierarchy status and distinct link style

#### Scenario: Unselected ancestor privacy
- **WHEN** one generated ancestor has an ordinary unselected source file and another has no source file
- **THEN** both have identical Namespace status without source file paths, content, or existence claims

#### Scenario: Safe index labels and private gaps
- **WHEN** the index and hierarchy include generated destinations while a private or alias-conflicting prefix is unavailable
- **THEN** generated index anchors keep their exact full logical names, generated links identify Namespace status, and unavailable prefixes receive no new route
