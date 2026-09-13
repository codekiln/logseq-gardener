# Foundation verification

## Implementation

The `project-foundation` change implements all six capability specifications. The main specs have been synchronized; the change remains open for pull-request review.

| Area | Implementation | Verification |
| --- | --- | --- |
| Repository workflow | Root mise configuration and lockfile, Rust toolchain, executable tasks, RuleSync source | Aggregate CI, generated-file checks, OpenSpec skill version checks |
| Public CLI | `src/cli.rs`, `src/help.rs`, `src/output.rs` | CLI integration tests and all 18 pinned Clilint checks at score 4 |
| Licensing | AGPLv3, third-party notices, locked dependency inventory and license text | Notice consistency check and native archive inspection |
| Parser boundary | `docs/architecture.md` | Comparison prerequisites reviewed; no parser or graph dependency introduced |
| Releases | Title validation, release-plz configurations, generated cargo-dist workflow | Pre/post-1.0 version-impact tests, title tests, dist plan/check, native archive checksum |
| Documentation | README and focused guides | Every README shell example executed; local and README web links checked; assessments below |

## Executed checks

- `mise run ci` passed locally on Apple silicon macOS, including a final run after packaging changes. The initial pull-request Linux aggregate CI run also passed.
- `mise run clilint:check` passed the complete global bundle and hierarchical-help bundle. The help-quality assessment was reviewed from captured evidence and accepted by Clilint.
- `mise exec -- openspec validate --all --strict` passed the change and main specifications.
- `mise run rulesync:check` verified existing generated output, regeneration, and a second consistency check. All OpenSpec source skills match CLI 1.6.0.
- `mise run docs:check` ran every README shell command and verified local documentation links. `mise run docs:links` received HTTP 200 for every README web link.
- `mise run release:check` passed title-independent version-impact fixtures and cargo-dist configuration checks.
- `dist build --artifacts local --target aarch64-apple-darwin` produced a native archive. Its SHA-256 matched its checksum file; the extracted binary returned the expected version JSON. The archive contains license text, dependency notices, and documentation with valid local links.
- `git diff main --check` passed. Git attributes preserve upstream license files' final blank lines verbatim.

## README purpose assessment

Verdict: PASS

Problems: None.

Suggestions: None.

The opening explains the problem, intended users, value, and available foundation features. Project status distinguishes planned distribution targets from current verification and explicitly states that no binary release is published. The source-build path leads to explained help and version commands. Support, maintenance, license, and focused contributor guides are linked.

Uncertainty: A fresh installation's duration depends on network speed and native build tools. This session verified installation and command execution, but does not establish a universal five-minute installation time.

## README style assessment

Verdict: PASS

Problems: None.

Suggestions: None.

Purpose and user decisions precede implementation details. Paragraphs use concrete language, and the first-use explanation identifies what each command returns. The README keeps detailed CLI, architecture, contribution, and release information in linked guides.

Uncertainty: None beyond the installation timing described above.

## Release activation and remaining work

Automated release preparation requires a maintainer-installed GitHub App, its repository secrets, required-check and squash-merge settings, and `RELEASE_ENABLED=true`. No release has been published. Linux and other native target results are provided by the pull request's GitHub Actions checks.

The next implementation milestone is the parser comparison, followed by read-only garden commands. LSP, publishing, mutation, merge-driver behavior, and cache architecture remain future work.
