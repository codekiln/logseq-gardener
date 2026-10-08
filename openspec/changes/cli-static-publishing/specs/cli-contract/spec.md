## MODIFIED Requirements

### Requirement: Versioned output and exits

Successful JSON SHALL be one document with format_version 1. Success SHALL return 0, invalid invocations 2 with empty stdout and a useful stderr diagnostic, and output or generation failure 1; broken pipes SHALL return 0. Redirected output SHALL contain no terminal control sequences. Publication diagnostics SHALL remain separate from successful JSON stdout.

#### Scenario: Reject invalid input

- **WHEN** a caller supplies an unknown option or nonexistent section with --format json
- **THEN** lsg returns 2, keeps stdout empty, and writes an actionable diagnostic to stderr

#### Scenario: Generation fails

- **WHEN** the SDK cannot complete publication
- **THEN** lsg returns 1, keeps stdout empty, and reports the failure on stderr
