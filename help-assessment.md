# Review help examples

Clilint checks the CLI's behavior automatically and uses a saved assessment for the quality of its help examples. The assessment only applies to the help text that was reviewed.

After changing help, capture a report with the same target and bundle used by `mise run clilint:check`. Follow Clilint's `assess-cli-help` skill to review the captured examples and update `tests/assessments/help.json`. Run `mise run clilint:check` again to verify the assessment. CI rejects an assessment when the help text changes.

The Clilint revision is pinned in `mise.toml`. [Third-party notices](THIRD_PARTY_NOTICES.md) identify the source of the bundled hierarchical-help checker.
