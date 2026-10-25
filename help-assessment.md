# Review help examples

Clilint checks the CLI's behavior automatically and uses a saved assessment for the quality of its help examples. The assessment only applies to the help text that was reviewed.

After changing help, capture a report with the same target and bundle used by `mise run clilint:check`. Follow Clilint's `assess-cli-help` skill to review the captured examples and update `tests/assessments/help.json`. Run `mise run clilint:check` again to verify the assessment. CI rejects an assessment when the help text changes.

The Clilint revision is pinned in `mise.toml`. [Third-party notices](THIRD_PARTY_NOTICES.md) identify the source of the bundled hierarchical-help checker.

Search option review: the publication usage includes `--search`; its behavior explains external artifact generation and lazy consumption. The existing copyable publication example remains valid with search omitted. Search API details and exclusions live in `docs/static-search.md`.
