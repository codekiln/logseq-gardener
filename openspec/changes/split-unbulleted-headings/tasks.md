## 1. SDK

- [x] 1.1 Add tests in `garden.rs` that an unbulleted `# [[Filed]]` after a bulleted block starts its own block with its indented child, and that a heading inside a quote stays text
- [x] 1.2 Add `garden::parse_document`, which turns top-level `Heading` nodes into bullets, and use it in `load_garden`

## 2. Verification

- [x] 2.1 Publish garden commit `083106d` with a `{{query (between -30d today)}}` page and `SOURCE_DATE_EPOCH` set to 2026-10-27, before and after; record the counts in `verification.md`
- [x] 2.2 Check in Logseq's `gh-pages` database that an unbulleted heading line is a block of its own
- [x] 2.3 Run `mise run ci`
