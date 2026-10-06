# Rust parser licenses

This audit records the licenses and attribution files supplied with the lsdoc revision tested in the parser comparison. Read [the adoption guide](../../../../../docs/parser-adoption.md) for the findings and remaining work.

## Reproduce the evidence

From the Gardener repository root:

```sh
mise exec -- python3 openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/audit.py
```

The audit requires the existing `martinkoutecky/lsdoc` ghq checkout and cached Cargo dependencies. It extracts committed source into a system temporary directory outside Gardener's Cargo workspace, reads the committed Cargo.lock, and runs locked offline Cargo metadata. Add `--online` to allow Cargo to fetch missing locked registry dependencies. The audit uses the existing source checkout and leaves it unchanged.

After deliberately changing the pinned revision or evidence:

```sh
mise exec -- python3 openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/audit.py --update
```

Review both [inventory.json](inventory.json) and [license-texts.txt](license-texts.txt). The inventory contains the source revision, lockfile hash, package sources and checksums, declared licenses, and supplied license-file hashes. The combined text preserves the supplied license, copyright, and NOTICE files, including alternative license texts. It covers the entire upstream metadata package graph, including build-time packages.

The candidate lockfile is evidence about the tested source. The SDK contributor must regenerate Gardener's own notices after adopting the parser, using the SDK's resulting lockfile. These local research checks supplement the project notice check; they require upstream source and Cargo cache access.
