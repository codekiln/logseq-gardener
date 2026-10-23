## ADDED Requirements

### Requirement: Publish email autolinks

The publisher SHALL render parsed email autolinks as mailto anchors displaying the source address. It SHALL escape HTML text and attributes and encode address characters that would otherwise introduce URI queries, fragments, or escapes. It SHALL preserve email-looking content in literal code. Email rendering SHALL perform no network or asset requests.

#### Scenario: Address navigation
- **WHEN** selected source contains `<reader@example.org>`
- **THEN** generated HTML displays `reader@example.org` as a link to `mailto:reader@example.org` without an unsupported-email diagnostic

#### Scenario: Address escaping
- **WHEN** a parsed email contains HTML-sensitive characters or URI delimiters
- **THEN** its visible address is HTML-escaped and its mailto destination preserves those characters as address data

#### Scenario: Literal address
- **WHEN** an address occurs inside inline or fenced code
- **THEN** it remains literal code without an email anchor

### Requirement: Preserve safe email labels and fallbacks

The publisher SHALL render an email inside another link label as escaped text without creating a nested anchor. Missing, non-string, empty, or whitespace/control-containing email components SHALL retain a visible literal fallback and local diagnostic.

#### Scenario: Email in a link label
- **WHEN** an email node appears inside a rendered link label
- **THEN** the label displays the address and only the outer link creates an anchor

#### Scenario: Malformed email data
- **WHEN** an email node lacks usable local-part or domain data
- **THEN** the publisher emits a visible escaped fallback with a diagnostic and no email anchor
