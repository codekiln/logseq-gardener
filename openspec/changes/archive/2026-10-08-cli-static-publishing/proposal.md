## Why

A user should be able to generate readable HTML for selected garden namespaces through `lsg`, without invoking an SDK example. [Issue #20 — Publish through lsg](https://github.com/codekiln/logseq-gardener/issues/20) brings the demonstrated Rust publisher into the normal command-line workflow for the garden trial.

The [publishing brief](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md#3-static-knowledge-garden-publishing) calls for independently useful HTML from the shared graph. The [earlier proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md) reserves `lsg export` for Markdown export and `--graph` for source selection; this HTML command uses `lsg publish --graph`.

## What Changes

- Add `lsg publish` with explicit garden, fresh output, filename format, and repeated include/exclude selectors.
- Delegate publication to the SDK and provide offline navigable help, human/JSON reports, and separate diagnostics.
- Verify the CLI pipeline preserves sources and excludes withheld content; document a real-garden trial.

## Capabilities

### New Capabilities

- `cli-static-publishing`: invoke the SDK publisher through lsg and report success, unsupported content, and fatal failures.

### Modified Capabilities

- `cli-contract`: distinguish fatal generation failures from invalid invocations while retaining output and closed-pipe behavior.

## Impact

CLI argument dispatch, embedded help, process reporting, integration tests, and focused CLI/publishing documentation. The SDK remains the publication implementation.

## Citations

- [My/Principle/CLI/Centricity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___CLI___Centricity.md) supports making publication accessible through the normal CLI.
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md) supports explicit source, destination, filename format, and outcome reporting.
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md) supports discoverable offline help and focused trial instructions.
