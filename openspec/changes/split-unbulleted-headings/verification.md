# Verification: split-unbulleted-headings

`{{query (between -30d today)}}` now returns 76 blocks, the same as Logseq's build.

Measured on 2026-10-28 by publishing garden commit `083106d` with `--journals`, a page holding the query, and `SOURCE_DATE_EPOCH` set to 2026-10-27 12:00 UTC.

| Measure | Before (`main` f54a8d7) | After | Logseq `gh-pages` `8132c95` |
| --- | --- | --- | --- |
| `(between -30d today)` results | 75 | 76 | 76 |

Logseq's `db.gz` holds `## [[AI Coding]]`, the first line of `journals/2025_04_23.md` written without a bullet, as a block of its own. In `lsg`'s journal page for that day, the heading now renders as a bulleted `<h2>` block with the `#Updated` bullet nested inside it; before, it rendered without a bullet above a separate outline.

The tests `unbulleted_heading_starts_a_block` and `headings_inside_quotes_stay_text` in `garden.rs` cover the block split. Without `parse_document`, lsdoc returns the heading as a `Heading` node and the outline has three blocks instead of four.
