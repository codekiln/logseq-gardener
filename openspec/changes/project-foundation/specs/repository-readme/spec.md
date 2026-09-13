## ADDED Requirements

### Requirement: Reader entry point

The README SHALL explain the problem, audience, intended value, current maturity, maintainer, support, license, installation, first use, and links to focused guides before implementation details.

#### Scenario: First use

- **WHEN** a reader follows every README shell command from the checkout
- **THEN** the commands succeed and show the documented help or version result

### Requirement: Verified documentation

README commands and links SHALL be checked and its platform and release claims SHALL distinguish configured targets from tested releases.

#### Scenario: Review README

- **WHEN** the contributor runs documentation checks and README style and purpose assessments
- **THEN** all local links resolve, commands pass, and no unsupported maturity or installation claim remains
