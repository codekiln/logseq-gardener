## Purpose

Keep Markdown authoritative and establish prerequisites for parser adoption.

## Requirements

### Requirement: Markdown and comparison prerequisites

Saved Logseq Markdown files SHALL remain the source of the notes. The OpenSpec plan SHALL require a named comparison experiment before parser adoption or garden operations, with lsdoc as the Rust candidate, official mldoc as syntax reference, and Logseq OG graph-parser and graph-validator as graph compatibility references.

#### Scenario: Review next parser change

- **WHEN** a contributor opens the architecture guide and its OpenSpec plan
- **THEN** the guide explains the parser roles and pending choice, while the plan identifies sample gardens, compatibility checks, text-preservation checks, and the named experiment directory for results
