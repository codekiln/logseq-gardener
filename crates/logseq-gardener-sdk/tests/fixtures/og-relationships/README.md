# Logseq OG relationship fixtures

`tests/og_relationships.rs` loads the notes in `garden/` through the SDK and compares the SDK's page relationships with the ones current Logseq OG built from the same notes. `mise run test` runs the comparison, and `mise run ci` also checks that these files still match the recorded evidence. Neither needs a Logseq checkout.

## What the test compares

For the default filename format and for `:file/name-format :triple-lowbar`, the test compares:

- each note's page name, title, file and journal date;
- each note's `alias::` names, and which files an alias name finds, including `Other Name`, which two pages share;
- namespace parents, such as `Names` for `Names/Nested`; and
- pages that notes link to but that have no file, such as `Missing Target`.

The SDK answers come from `PageCatalog`, `page_titles`, `journals`, `page_names`, `ReferenceIndex` and `GraphIndex`. The publisher is not involved. `tests/relationships/mod.rs` holds the code that derives them, which `mise run docs:parser-check` also runs on the docs graph.

## Where the expected results come from

`expected.json` is derived from `relationships.json` in the [current Logseq OG comparison](../../../../../openspec/changes/current-og-relationships/experiments/file-relationships/README.md). That comparison ran Logseq OG's `version/file` branch at revision `6efedb75588763af256bc7dfd0ed5526dc91fe7c` with mldoc 1.5.7; Logseq 0.9.8 produced the same results. `expected.json` repeats the revision, the runtime versions and each note's SHA-256 hash. The notes are synthetic test notes written for this project.

Logseq records more than the SDK models, so the derivation keeps the current Logseq results with these changes:

- Logseq's built-in pages, such as `TODO` and `card`, are dropped: they are the file-less pages that no note links to.
- Pages named by a block UUID are dropped. Block references are not page names.
- A file-less page whose name is an `alias::` of a stored page appears under `aliases` with the files that declare it, since the SDK finds those files rather than a separate page.
- The remaining file-less pages that some note links to appear under `fileless`.
- Journal dates are written as `YYYY-MM-DD`; block content, block order, properties and references are left out.

## Review or regenerate the expected results

After the Logseq evidence or the fixture notes change on purpose, regenerate the evidence with `mise run parser:og-update` as its guide describes, then run:

```sh
mise run parser:sdk-fixtures-update
mise run parser:sdk-fixtures-check
mise exec -- cargo test -p logseq-gardener-sdk --test og_relationships
```

The update copies the recorded notes from the OpenSpec fixtures whose hashes match and rewrites `expected.json`. Review its diff the way you review `relationships.json`: a changed name, alias, namespace or file-less page is a change in Logseq's behavior, and the SDK must follow it before the test passes again.
