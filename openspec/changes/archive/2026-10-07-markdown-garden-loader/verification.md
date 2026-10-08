# Verification

Full `mise run ci` passed, including formatting, Clippy, workspace tests, strict OpenSpec checks, dependency records, documentation, and CLI conformance. Documentation checks also passed after adding this verification record.

The SDK example loaded the public encode garden on October 6, 2026 at [b3589786 — Source garden revision](https://github.com/codekiln/logseq-encode-garden/tree/b35897866c1287128c4cf473e57b8b8349b56743). The garden checkout was clean. SHA-256 hashes of every visible Markdown source under `pages/` and `journals/` were identical before and after loading.

Run the same example from the Gardener repository:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
mise exec -- cargo run -p logseq-gardener-sdk --example load_garden -- "$garden_root"
```

Observed output:

```text
Loaded 6936 Markdown documents: 6448 pages, 488 journals; 38770 page references, 460 block references.
```

The [SDK tests](../../../../crates/logseq-gardener-sdk/tests/garden.rs) passed for original source preservation, parsed outlines and properties, page/block references, literal code, ordering, hidden entries, invalid roots, invalid UTF-8, unreadable notes, symlinks, and special files.

The [loading guide](../../../../docs/garden-loading.md) describes the supported input and syntax API. These results establish a limited source reader; they do not establish logical page identity, publication filtering, rendered-site safety, or complete Logseq graph compatibility.
