## MODIFIED Requirements

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
