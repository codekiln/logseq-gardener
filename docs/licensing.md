# Licensing and dependencies

Logseq Gardener uses AGPL-3.0-only. [LICENSE](../LICENSE) contains the complete AGPLv3 text. [Third-party notices](../THIRD_PARTY_NOTICES.md) record adapted source, its revision, attribution, modifications, and distribution terms. [Third-party dependency licenses](../THIRD_PARTY_LICENSES.txt) collect upstream license texts in one file. The [dependency source inventory](dependency-sources.json) records locked Cargo packages, sources, checksums, and declared licenses. The [dependency guide](dependencies.md) explains why the CLI uses its direct dependencies.

## Update dependencies

For the parser candidate's pinned source and license evidence, see [Parser adoption](parser-adoption.md).

After changing Cargo dependencies, explain the reason for a new direct dependency in Cargo.toml and the [dependency guide](dependencies.md). Run `mise run notices:update` and review the source inventory and combined license file. `mise run notices:check` rejects stale generated files. Keep Cargo.lock committed and build with `--locked`.

For each new dependency or copied source, record its version or revision, upstream source, license, copyright attribution, modifications, required notices, and source-distribution obligations. A license identifier alone is not a completed review. Preserve upstream license and NOTICE text in the combined file. The current inventory includes all transitive Cargo packages, including platform-specific dependencies.

## Distribute builds

Include LICENSE, THIRD_PARTY_NOTICES.md, THIRD_PARTY_LICENSES.txt, and the dependency inventory with binaries. Publish the corresponding project source at the release tag and provide the exact dependency sources and build scripts needed to reproduce it. Cargo-dist's source archive covers the repository; before distribution, also run `cargo vendor --locked` and retain the resulting dependency sources with the release materials. Verify that any upstream NOTICE text remains included.

Modified AGPL components require the applicable modification notices and corresponding source. If future software offers network interaction, implement the required corresponding-source offer for remote users. The current CLI provides offline documentation and version reporting.
