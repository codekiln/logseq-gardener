# Garden loading

A Rust application can load Logseq Markdown files through the SDK before resolving page identities or rendering a site. The loader returns original source, relative file location, page/journal directory kind, parsed blocks, and parser-extracted page and block references.

Run the SDK example from the repository root, replacing the path with your garden:

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example load_garden -- /path/to/garden
```

The example reports document and reference counts. It does not print note contents or modify the garden.

## Supported input

The root must be a directory containing `pages/`, `journals/`, or both. The loader recursively reads visible lowercase `.md` files under those directories. It ignores ordinary non-Markdown files and entries whose names start with `.`. Documents are returned in relative-path order. Empty note directories are valid; an unrelated directory produces an error.

The loader rejects visible symlinks and special files. Unreadable files and invalid UTF-8 stop loading with a file-location error. The result is an in-memory snapshot of the files read during the call. Use the loader against a stable local checkout when comparing repeated results.

The SDK's `garden` module exposes `load_garden`, `Garden`, `GardenDocument`, and `DocumentKind`. Parsed syntax uses the re-exported `ast` module from [lsdoc's tested revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d). That initial syntax API is tied to the pinned parser; a future parser update can require SDK changes.

## Publishing follow-up

This loader preserves parser results without interpreting filenames as logical page names. Garden configuration, titles, aliases, journals, publication properties, and reference resolution still need the SDK's Logseq graph layer. Loading does not select public content or generate a website. [Issue #4 — Page lookup](https://github.com/codekiln/logseq-gardener/issues/4) tracks identity work, and [Issue #11 — Static site publishing](https://github.com/codekiln/logseq-gardener/issues/11) tracks the end-to-end outcome.

The [merged comparison findings](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/findings.md) support the limited Markdown reader, including source preservation and reference extraction. A known math-inside-emphasis difference remains relevant to rendering. Broader corpus validation and the final compatibility decision continue under [Issue #3 — Parser comparison](https://github.com/codekiln/logseq-gardener/issues/3). See [Parser adoption](parser-adoption.md) for the pinned dependency and evidence.
