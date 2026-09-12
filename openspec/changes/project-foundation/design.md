## Context

The project starts in a new Git repository. Clilint at commit `200e92a5d76420e8c47fd9e623da704b355dbc7d` supplies the working repository model. The foundation must be useful to contributors and expose a dependable public interface before graph implementation begins.

## Goals / Non-Goals

**Goals:** Build `lsg`, publish embedded help and version metadata, automate verification and release preparation, and document licensing and parser prerequisites.

**Non-Goals:** Parser adoption, graph queries, LSP, publishing, mutation, merge-driver behavior, and cache architecture belong to later milestones.

## Decisions

Use one binary crate with internal `cli`, `help`, and `output` modules. Clap validates arguments; an embedded document model supplies both human and JSON help. A generic parser trait would force decisions about an untested AST, so document the future boundary and preserve room for `lsdoc` without adding a runtime parser dependency.

Expose the root and `version` command. Treat `help`, `help outline`, and `help section` as documentation operations at each command path, rather than recursively advertising help as another product command. The same document supplies `help`, `--help`, and `-h`. Heading identifiers remain stable and section retrieval can include descendants. JSON responses include `format_version: 1` and command identity. Invalid arguments return 2 with a diagnostic on stderr; output failures return 1; a closed downstream pipe is successful early termination.

Pin Clilint at revision `200e92a5d76420e8c47fd9e623da704b355dbc7d` and vendor its MIT-licensed `codekiln-help` checker at the inspected commit, since the help bundle is distributed as source. Run the entire global bundle, including its evidence-bound help assessment, plus the hierarchical checker. Committed assessments must be renewed when captured help changes. CI must reject stale assessments and all incomplete or substandard results.

Reuse Clilint's executable task and release patterns. The root mise configuration pins all development runtimes. RuleSync generates ignored agent files from committed source; CI checks generation and OpenSpec skill versions. Release-plz selects versions from validated titles, and cargo-dist generates the tag-triggered workflow. Preserve the source licenses and track adapted files in third-party notices.

## Risks / Trade-offs

- Fresh installation needs network access and may exceed the first-use time on slow connections → distinguish source installation from future binary releases in the README.
- Automated releases require GitHub App secrets and repository settings → document the exact setup, with release preparation enabled only when the App is configured.
- A source-distributed help checker adds maintenance work → pin the upstream revision and test the bundled checker alongside the CLI.
- Native platforms require their own runners → describe configured targets separately from locally verified support.

## Migration Plan

Build and review the foundation on a feature branch. Merge after CI passes. Configure the release App before enabling automated preparation. Reverting the foundation affects only the new project; help and version commands never access garden files.

## Resolved Questions

### 1 - Which repository holds the new project?

Use the user's prepared ghq directory and a new public `codekiln/logseq-gardener` remote. The user has renamed and archived the earlier Nx repository as `logseq-gardener-nx-old`.

### 2 - Which license and executable name apply?

Use AGPL-3.0-only and the executable name `lsg`.

### 3 - What can users run before parser selection?

Users can read offline documentation and version metadata. Garden operations follow the recorded parser comparison.

## Open Questions

None for foundation implementation. Release activation requires the maintainer's GitHub App credentials, as documented in the release guide.
