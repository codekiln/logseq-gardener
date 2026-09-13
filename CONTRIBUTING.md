# Contributing

Install the pinned tools with `mise install`. Run `mise run ci` for the same checks used by GitHub Actions. List tasks with `mise tasks`; use `mise run <task> --help` for arguments.

The aggregate checks formatting, Clippy with warnings denied, tests, locked release builds, strict OpenSpec validation, OpenSpec skill versions, generated RuleSync files, dependency notices, README examples and links, Clilint conformance, title validation, release impact, and cargo-dist configuration. Use `mise exec -- cargo fmt` to apply Rust formatting.

Plan changes in OpenSpec. Load the matching skill in `.rulesync/skills/` and obtain the current `openspec instructions` before editing an artifact. Keep observable scenarios in each requirement and verification beside each implementation area. Edit `.rulesync/` source and run `mise exec -- rulesync generate`; generated agent files are ignored. `mise run rulesync:check` checks generation, including stale existing output.

The [parser comparison](docs/architecture.md) precedes substantive graph behavior. Add a new public command only with help traversal, structured output, noninteractive process tests, and Clilint verification.

## CLI conformance

`mise run clilint:check` runs Clilint at revision `200e92a5d76420e8c47fd9e623da704b355dbc7d`'s complete global bundle plus the pinned `codekiln-help` bundle. All checks must finish at score 4 with no error. The committed help assessment is bound to captured evidence. When help changes, capture a fresh report, load Clilint's `assess-cli-help` skill, review the actual help, and replace `tests/assessments/help.json`. A stale assessment must fail CI; do not generate a passing score automatically.

## Pull requests and releases

Use a Conventional Emoji title such as `✨ feat: add page lookup`, `🩹 fix: correct output`, or `📝 docs: explain help`. Check it with `mise run pr:title '<title>'`. See [release policy and setup](docs/releases.md). Every dependency update also requires the [notice process](docs/licensing.md).
