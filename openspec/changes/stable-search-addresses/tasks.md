## 1. Reproduce

- [x] 1.1 Add a publication test that appends one block to one page of a 41-page garden and checks which output files change; confirm it fails on main.
- [x] 1.2 Measure the same edit on the encode garden with main's `lsg`.

## 2. Stable search files

- [x] 2.1 Write the manifest at `lsg-search-manifest.json` with each page's block count, and link it from every page without a revision.
- [x] 2.2 Name blocks in shards by page number and block number within the page, and drop the counts from the title index.
- [x] 2.3 Start record chunk segments at blocks whose page and fragment digest is a multiple of 64.
- [x] 2.4 Teach the engine format version 3, and the loader to revalidate the manifest, to load the engine and title index together, and to forget the publication when a named file is missing or invalid.

## 3. Verification and handoff

- [x] 3.1 Regenerate the shared search fixture and update the Rust, engine, loader, parity and browser checks, including recovery after a later publication.
- [x] 3.2 Measure the edit, the search file sizes, search latency and build time on the encode garden against main, and record them in `verification.md`.
- [x] 3.3 Update the Static Search Data and Host Integration API reference pages.
- [x] 3.4 Run `mise run ci`, open the pull request and create the cleanup sub-issue.
