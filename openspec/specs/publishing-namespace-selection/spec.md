## Purpose

Select logical page namespaces with explicit inclusion and exclusion policies.

## Requirements

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

### Requirement: Exclusion precedence

The SDK SHALL exclude a namespace root and all descendants even if an inclusion would select them. Reordering or duplicating selectors SHALL NOT change the result.

#### Scenario: Nested exclusion

- **WHEN** `My` and `My/Private/Public` are included and `My/Private` is excluded
- **THEN** `My/Private`, `My/Private/Note`, and `My/Private/Public` are unselected while `My/AI` remains selected

### Requirement: Validated selection inputs

Construction SHALL reject empty selectors, empty slash-separated segments, surrounding segment whitespace, and control characters. Matching SHALL accept a page identity whose segments begin or end with whitespace, as Logseq keeps such segments, and SHALL return false for a page identity with an empty or whitespace-only segment or a control character. Constructor errors SHALL identify the include or exclude list and zero-based selector position without including selector text. The policy SHALL leave inputs and garden files unchanged.

#### Scenario: Invalid configuration

- **WHEN** either selector list contains an empty string, `/My`, `My/`, `My//AI`, ` My`, or a control character
- **THEN** construction returns an error identifying its list and position

#### Scenario: Invalid candidate

- **WHEN** a candidate has an empty or whitespace-only segment or a control character
- **THEN** the candidate is unselected

#### Scenario: Candidate segment with surrounding spaces

- **WHEN** the included root is `Obsidian` and the candidate is `Obsidian/Q/Info / Warning / Danger`
- **THEN** the candidate is selected under that exact name

### Requirement: Select every page

The SDK SHALL let a caller turn any valid policy into one that selects every page. Such a policy SHALL select every valid logical page name that no excluded root matches, whatever its inclusion list holds. Exclusion SHALL still win, and a page identity with an empty or whitespace-only segment or a control character SHALL stay unselected.

#### Scenario: Every page with an exclusion

- **WHEN** a policy with no included roots and the excluded root `My/Private` selects every page
- **THEN** `My`, `Other`, `SolveSpace` and `研究/庭/花` are selected while `My/Private` and `My/Private/Note` are not

#### Scenario: Included roots change nothing

- **WHEN** a policy with the included root `My` and the excluded root `My/Private` selects every page
- **THEN** each candidate is selected exactly when it is selected by the policy with no included roots, the same excluded root and every page selected

#### Scenario: Invalid candidate under every page

- **WHEN** a policy selects every page and the candidate is `My//AI` or contains a control character
- **THEN** the candidate is unselected
