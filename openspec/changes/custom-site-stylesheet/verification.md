# Verification

## Automated checks

- `mise exec -- cargo test --workspace stylesheet`: five focused tests passed. Coverage includes default API/options equivalence, copied CSS and link order on every document, unchanged garden source, excluded/private pages, invalid and unreadable files, and symlinks at both file and parent-directory levels. CLI coverage verifies success reports, stylesheet output, offline help, and failure with empty stdout and absent destination.
- `mise run ci`: passed formatting, Clippy, workspace and example tests, PR title tests, release build, OpenSpec validation, parser baseline, skill versions, generated rules, notices, documentation, Clilint, and release configuration checks.
- Clilint's captured root help was reassessed using `assess-cli-help`; `tests/assessments/help.json` records the matching request and evidence digest. All 18 checks passed at score four. Publishing help explains the new stylesheet flag, its validation and precedence, resource resolution, and current alias/block-reference/HTTPS media support.

## Real episode experiment

[GitP appearance experiment](experiments/gitp-appearance/README.md) records the clean public-source revision, reproduction command, generated content counts, browser inspection, and remaining homepage/navigation limitations.

## Follow-up

[Cleanup issue #51](https://github.com/codekiln/logseq-gardener/issues/51) is an actual child of [implementation issue #49](https://github.com/codekiln/logseq-gardener/issues/49). It tracks synchronization and archival of `custom-site-stylesheet` after [PR #50](https://github.com/codekiln/logseq-gardener/pull/50) merges into main.
