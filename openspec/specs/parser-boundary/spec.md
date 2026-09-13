## Purpose

Keep Markdown authoritative and establish prerequisites for parser adoption.

## Requirements

### Requirement: Markdown and comparison prerequisites

Logseq Markdown SHALL remain authoritative. The architecture record SHALL require a named comparison experiment before parser adoption or substantive graph semantics, with lsdoc as the Rust candidate, official mldoc as syntax reference, and Logseq OG graph-parser and graph-validator as graph compatibility references.

#### Scenario: Review next parser change

- **WHEN** a contributor opens the architecture guide
- **THEN** it identifies comparison corpora, separate syntax, graph and byte-preservation checks, and the change-local named experiments directory for results
