## Why

People and coding agents need a dependable terminal interface to Logseq Markdown gardens. Establishing a tested CLI contract and reproducible development workflow now lets later parser comparisons and graph commands build on working infrastructure.

## What Changes

- Introduce a reusable Rust SDK package and a CLI package producing `lsg`, with version reporting and offline hierarchical help.
- Provide versioned JSON, documented exits, noninteractive behavior, and clean output streams.
- Add AGPLv3 licensing, dependency notices, pinned mise tools, shared CI tasks, OpenSpec, and RuleSync source.
- Explain what the CLI currently does and how to try it in the README and focused guides.
- Add pinned Clilint global and hierarchical-help checks, Conventional Emoji title validation, release-plz preparation, and cargo-dist distribution checks.
- Record the parser comparison prerequisites while treating the saved Markdown notes as the original data.

## Capabilities

### New Capabilities

- `project-workflow`: Reproducible tools, executable tasks, generated instructions, and separate SDK and CLI packages.
- `cli-contract`: Offline help navigation, version reporting, output formats, and exit behavior.
- `repository-readme`: Purpose, maturity, installation, support, and verified first use.
- `licensing`: AGPLv3 distribution and dependency notice maintenance.
- `semantic-releases`: Validated titles, version preparation, and checksummed native artifacts.
- `parser-boundary`: Recorded comparison prerequisites for future parsing and graph semantics.

### Modified Capabilities

None.

## Impact

This creates the initial repository implementation and CI configuration. Runtime dependencies support argument parsing and JSON serialization. Clilint supplies the development conformance checks. Garden files hold the original notes; this milestone's commands inspect only embedded documentation and version metadata.

## Citations

- [My/Pref/Dev/mise/Tasks](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___mise___Tasks.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/CLI/Centricity/Offline Tools](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___CLI___Centricity___Offline%20Tools.md)
