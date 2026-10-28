# Verification: wrap-long-block-words

At 390×844, `GitP/A/Session/26/10/24-Sat` now fits the screen: nothing in the page extends past 390px.

Measured on 2026-10-28 with `mise run parity:build` on the encode garden, with every block expanded.

| Measure | Before (`main` 6406199) | After |
| --- | --- | --- |
| Layout viewport width | 449px | 390px |
| Right edge of the two `Ableton/Live/Mode/Arrangement` links | 406px, 437px | 369px, 373px |
| Elements past the right edge | 2 | 0 |

`outline_blocks_break_long_words_to_fit_a_phone_screen` in `tests/site_collapse.rs` failed before the change and passes after it.
