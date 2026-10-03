# Foundation verification

## Implementation

The `project-foundation` change has separate SDK and CLI packages, with the main specs synchronized. The pinned release-plz version handles the unpublished SDK workspace, and the local aggregate checks pass. The change remains open for pull-request review.

| Area | Implementation | Verification |
| --- | --- | --- |
| Repository workflow | Root mise configuration and lockfile, Rust toolchain, executable tasks, RuleSync source | Local and GitHub aggregate CI pass |
| Public CLI and SDK | CLI modules and `crates/logseq-gardener-sdk/` | CLI integration tests, SDK documentation test, an external SDK consumer, and all pinned Clilint checks at score 4 |
| Licensing | AGPLv3, third-party notices, locked dependency inventory and license text | Notice consistency check and native archive inspection |
| Parser boundary | `docs/architecture.md` | Comparison prerequisites reviewed; no parser or graph dependency introduced |
| Releases | Title validation, release-plz configurations, generated cargo-dist workflow | Title, workspace version-impact, and distribution configuration tests pass |
| Documentation | README and focused guides | Every README shell example executed; local and README web links checked; assessments below |

## Executed checks

- `mise run ci` passes with release-plz 0.3.169, including formatting, Clippy, Rust tests, documentation, notices, Clilint, release impact, and distribution configuration checks.
- `mise run clilint:check` passed the complete global bundle and hierarchical-help bundle after refreshing the help-quality assessment from captured evidence.
- A separate temporary Rust application compiled against the SDK path and reported its version; its dependency tree contained the SDK without Clap or Serde.
- `mise exec -- openspec validate --all --strict` passed the change and main specifications.
- `mise run rulesync:check` verified existing generated output, regeneration, and a second consistency check. All OpenSpec source skills match CLI 1.6.0.
- `mise run docs:check` ran every README shell command and verified local documentation links. `mise run docs:links` received HTTP 200 for every README web link.
- `mise run dist:check` validated the configured native archives and their included notices and licenses.
- `mise run release:impact-test` verifies feature, fix, breaking, and documentation-only changes before and after 1.0, including changes in the SDK package. Each temporary workspace commits a Cargo.lock before testing a release.
- `dist build --artifacts local --target aarch64-apple-darwin` produced a native archive. Its SHA-256 matched its checksum file; the extracted binary returned the expected version JSON. The archive contains license text, dependency notices, and documentation with valid local links.
- `git diff main --check` passed.

## README purpose assessment

Verdict: PASS

Problems: None.

Suggestions: None.

The opening explains the problem, intended users, value, and currently available commands. Project status distinguishes planned distribution targets from current verification and explicitly states that no binary release is published. The source-build path leads to explained help and version commands. Support, maintenance, license, and focused contributor guides are linked.

Uncertainty: A fresh installation's duration depends on network speed and native build tools. This session verified installation and command execution, but does not establish a universal five-minute installation time.

## README style assessment

Verdict: PASS

Problems: None.

Suggestions: None.

Purpose and user decisions precede implementation details. Paragraphs use concrete language, and the first-use explanation identifies what each command returns. The README keeps detailed CLI, architecture, contribution, and release information in linked guides.

Uncertainty: None beyond the installation timing described above.

## Release activation and remaining work

Automated release preparation requires a maintainer-installed GitHub App, its repository secrets, required-check and squash-merge settings, and `RELEASE_ENABLED=true`. The project pins release-plz 0.3.169, which includes [Git-only workspace reconstruction](https://github.com/release-plz/release-plz/releases/tag/release-plz-v0.3.168) and passes the local workspace release tests. No release has been published. Linux and other native target results are provided by the pull request's GitHub Actions checks.

The next implementation milestone is the parser comparison, followed by read-only garden commands. LSP, publishing, mutation, merge-driver behavior, and cache architecture remain future work.
