# Contributing

Install the pinned tools with `mise install`. Run `mise run ci` for the same checks used by GitHub Actions. List tasks with `mise tasks`; use `mise run <task> --help` for arguments.

The CI task checks formatting, Clippy, tests, documentation, dependency notices, and release configuration. Use `mise exec -- cargo fmt` to apply Rust formatting. Workspace tests cover both the CLI and SDK, including an SDK example compiled as a library consumer.

Plan changes in OpenSpec and keep requirements and verification together. Agent-specific instructions are maintained in `.rulesync/`. After editing that source, run `mise exec -- rulesync generate` and `mise run rulesync:check`.

Create a cleanup sub-issue under each OpenSpec implementation issue before handing its PR over for review. After merge, follow [OpenSpec cleanup](docs/openspec-cleanup.md) to sync requirements, archive the completed change, and submit a separate follow-up PR.

The [architecture guide](docs/architecture.md) explains the package responsibilities. [Parser adoption](docs/parser-adoption.md) records the parser decision.

## Documentation

The documentation is a Logseq graph in `docs/`, and `lsg` publishes it as the documentation site. To edit it in Logseq 0.10 desktop, open the graph menu at the top of the left sidebar, choose Add new graph, then Choose a folder, and select `docs`. Each page is a file in `docs/pages/`. [lsg/How To/Edit the Documentation](docs/pages/lsg___How%20To___Edit%20the%20Documentation.md) explains how to name, tag and write a page, and how to turn one of the remaining Markdown documents in `docs/` into pages.

`mise run docs:site` builds `lsg` from the checkout and publishes the graph, with search, into a new folder under `target/docs-site/`; open its `index.html` to read the site. The task fails when `lsg` reports any diagnostic. `mise run docs:check`, which `mise run ci` runs, builds the site the same way, runs the README commands, and checks page tags, the `:hidden` entries of the remaining Markdown documents, and local links.

## CLI conformance

`mise run clilint:check` checks command behavior and help navigation. Every check must pass. Changing the help text also requires a fresh review of its examples; [help assessment maintenance](help-assessment.md) explains how.

## Pull requests and releases

Use a Conventional Emoji title such as `✨ feat: add page lookup`, `🩹 fix: correct output`, or `📝 docs: explain help`. Check it with `mise run pr:title '<title>'`. See [release policy and setup](docs/releases.md). Every dependency update also requires the [notice process](docs/licensing.md).
