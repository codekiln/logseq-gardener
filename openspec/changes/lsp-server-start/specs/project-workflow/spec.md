## ADDED Requirements

### Requirement: Separate language server package

The workspace SHALL provide a `logseq-gardener-lsp` library package that holds the Language Server Protocol code and depends on the SDK. The CLI package SHALL depend on it for `lsg lsp`. The SDK SHALL build without it and without any protocol dependency. The package SHALL be unpublished and SHALL produce no release artifact of its own; releases SHALL give it the same version as the CLI and the SDK.

#### Scenario: Use the SDK without the server

- **WHEN** a Rust application depends on logseq-gardener-sdk alone
- **THEN** its dependency tree holds no `lsp-server`, `lsp-types` or `notify` package

#### Scenario: Plan a release

- **WHEN** a contributor runs `mise run release:check`
- **THEN** the release plan builds `lsg` as its only application and the check passes
