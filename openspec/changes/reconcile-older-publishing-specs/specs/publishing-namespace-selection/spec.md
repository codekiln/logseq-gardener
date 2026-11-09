## MODIFIED Requirements

### Requirement: Explicit namespace inclusion

The SDK SHALL select a logical page identity only when it equals an included namespace root or is its slash-separated descendant. Matching SHALL be exact and case-sensitive. An empty inclusion list SHALL select nothing, unless the policy selects every page as the requirement "Select every page" defines.

#### Scenario: Namespace boundary

- **WHEN** the included root is `My/AI`
- **THEN** `My/AI` and `My/AI/Rule` are selected and `My/AIM` is not

#### Scenario: Empty inclusion

- **WHEN** the policy has no included roots and does not select every page
- **THEN** every page is unselected

#### Scenario: Logical Unicode identity

- **WHEN** the included root is `研究/庭` and the candidate is `研究/庭/花`
- **THEN** the page is selected without treating its name as a filesystem path

#### Scenario: Case distinction

- **WHEN** the included root is `My/AI` and the candidate is `my/ai`
- **THEN** the candidate is unselected

