# Verification: keep-fragment-targets-below-header

A fragment target now lands with its block's first line below the sticky header, at desktop and phone widths, whether the browser scrolls to it on page load, `fragments.js` scrolls to it on a hash change, or a reader chooses a note result in search.

Checked on 2026-11-01 with headless Chromium (Playwright 1.60) at 1280×800 and 390×844. The fixture garden has one page, `Long`, with 200 one-line blocks, published with `lsg publish --include Long --search` by an `lsg` built from `origin/main` (before) and from this branch (after). "Block top" is the top of the block's `li`, which starts 5px above its `source-block` anchor. The header's bottom edge is at 48px in both widths.

| Action | Width | Before: block top | After: block top |
| --- | --- | --- | --- |
| Open the page at `#source-block-2528` | 1280px | −5px, under the header | 51px, below the header |
| Open the page at `#source-block-2528` | 390px | −5px, under the header | 51px, below the header |
| Change the address to `#source-block-2571` (`fragments.js` calls `scrollIntoView()`) | 1280px | −5px, under the header | 51px, below the header |
| Change the address to `#source-block-2571` | 390px | −5px, under the header | 51px, below the header |
| Search for `Block 61 with` and choose the note result, `#source-block-2571` | 1280px | −5px, under the header | 51px, below the header |
| Search for `Block 61 with` and choose the note result | 390px | −5px, under the header | 51px, below the header |

With `scroll-padding-top` set to 48px instead, the same checks put the block's top at 43px, 5px under the header, which is why the rule uses 56px.
