# Licensing and dependencies

Logseq Gardener uses AGPL-3.0-only. [LICENSE](../LICENSE) contains the complete AGPLv3 text. [Third-party notices](../THIRD_PARTY_NOTICES.md) record adapted source, its revision, attribution, modifications, and distribution terms. [The dependency inventory](dependencies.md) lists all locked Cargo packages and their declared licenses.

## Update dependencies

After changing Cargo dependencies, run `mise run notices:update` and review the inventory and copied license texts under `licenses/dependencies/`. `mise run notices:check` rejects stale metadata or license files. Keep Cargo.lock committed and build with `--locked`.

For each new dependency or copied source, record its version or revision, upstream source, license, copyright attribution, modifications, required notices, and source-distribution obligations. A license identifier alone is not a completed review. Preserve upstream license and NOTICE files. The current inventory includes all transitive Cargo packages, including platform-specific dependencies.

## Distribute builds

Include LICENSE, THIRD_PARTY_NOTICES.md, the dependency inventory, and dependency license files with binaries. Publish the corresponding project source at the release tag and provide the exact dependency sources and build scripts needed to reproduce it. Cargo-dist's source archive covers the repository; before distribution, also run `cargo vendor --locked` and retain the resulting dependency sources with the release materials. Verify that any upstream NOTICE files remain included.

Modified AGPL components require the applicable modification notices and corresponding source. If future software offers network interaction, implement the required corresponding-source offer for remote users. The foundation itself provides offline documentation and version reporting.
