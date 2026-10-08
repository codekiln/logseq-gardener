## ADDED Requirements

### Requirement: Explicit namespace inclusion

The SDK SHALL select a logical page identity only when it equals an included namespace root or is its slash-separated descendant. Matching SHALL be exact and case-sensitive. An empty inclusion list SHALL select nothing.

#### Scenario: Namespace boundary

- **WHEN** the included root is `My/AI`
- **THEN** `My/AI` and `My/AI/Rule` are selected and `My/AIM` is not

#### Scenario: Empty inclusion

- **WHEN** the policy has no included roots
- **THEN** every page is unselected

#### Scenario: Logical Unicode identity

- **WHEN** the included root is `研究/庭` and the candidate is `研究/庭/花`
- **THEN** the page is selected without treating its name as a filesystem path

#### Scenario: Case distinction

- **WHEN** the included root is `My/AI` and the candidate is `my/ai`
- **THEN** the candidate is unselected

### Requirement: Exclusion precedence

The SDK SHALL exclude a namespace root and all descendants even if an inclusion would select them. Reordering or duplicating selectors SHALL NOT change the result.

#### Scenario: Nested exclusion

- **WHEN** `My` and `My/Private/Public` are included and `My/Private` is excluded
- **THEN** `My/Private`, `My/Private/Note`, and `My/Private/Public` are unselected while `My/AI` remains selected

### Requirement: Validated selection inputs

Construction SHALL reject empty selectors, empty slash-separated segments, surrounding segment whitespace, and control characters. Matching SHALL return false for malformed page identities. Constructor errors SHALL identify the include or exclude list and zero-based selector position without including selector text. The policy SHALL leave inputs and garden files unchanged.

#### Scenario: Invalid configuration

- **WHEN** either selector list contains an empty string, `/My`, `My/`, `My//AI`, ` My`, or a control character
- **THEN** construction returns an error identifying its list and position

#### Scenario: Invalid candidate

- **WHEN** a candidate has empty segments or surrounding segment whitespace
- **THEN** the candidate is unselected
