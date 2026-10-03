# Contributing

Install the pinned tools with `mise install`. Run `mise run ci` for the same checks used by GitHub Actions. List tasks with `mise tasks`; use `mise run <task> --help` for arguments.

The CI task checks formatting, Clippy, tests, documentation, dependency notices, and release configuration. Use `mise exec -- cargo fmt` to apply Rust formatting. Workspace tests cover both the CLI and SDK, including an SDK example compiled as a library consumer.

Plan changes in OpenSpec and keep requirements and verification together. Agent-specific instructions are maintained in `.rulesync/`. After editing that source, run `mise exec -- rulesync generate` and `mise run rulesync:check`.

The [architecture guide](docs/architecture.md) explains the package responsibilities and the pending parser choice.

## CLI conformance

`mise run clilint:check` checks command behavior and help navigation. Every check must pass. Changing the help text also requires a fresh review of its examples; [help assessment maintenance](help-assessment.md) explains how.

## Pull requests and releases

Use a Conventional Emoji title such as `✨ feat: add page lookup`, `🩹 fix: correct output`, or `📝 docs: explain help`. Check it with `mise run pr:title '<title>'`. See [release policy and setup](docs/releases.md). Every dependency update also requires the [notice process](docs/licensing.md).
