# Verification

The outline now matches Logseq's published build in size, weight, spacing, colour and bullet position on every fixture page measured, and block zoom on `rw-cli` shows the breadcrumb Logseq shows. Two of `rw-cli`'s four collapsed blocks still render open; design question 11 explains why and leaves it for a follow-up.

Compared on 2026-10-25: Logseq's `gh-pages` build of garden commit `09cf7fb`, served locally, against `mise run parity:build` from this branch (run `20261025T065337Z`, 6,984 pages). Values are computed styles read in the browser at 1280×800 unless noted.

## Title, bullets and guide lines

`1Password/Account/Service`, dark theme:

| Measure | Logseq | lsg |
| --- | --- | --- |
| Title size, weight, line height, space below | 36px, 300, 40px, 24px | 36px, 300, 40px, 24px |
| Title colour | `#fff` | `#fff` |
| Bullet column left edge, levels 1–5 | 238, 269, 300, 331, 362 | 236, 267, 298, 329, 360 (2px left, as the title is) |
| Bullet column, dot | 14px, 6px dot `rgb(32,36,37)` at 0.8 | 14px, 6px dot `rgb(32,36,37)` at 0.8 |
| Guide line | 2px `rgba(226,240,253,0.082)`, 6px into the bullet column | the same |
| Block padding, line height | 2px, 25.6px | 2px, 25.6px |
| Text start after the bullet column | 19px | 19px |
| List markers on outline items | none | none (0 of all items) |
| Empty paragraph after page properties | none | none |

Light theme (Logseq's root class switched to `light`; `lsg` with the light colour scheme): dot `rgb(241,243,245)`, guide `rgba(2,28,55,0.075)`, h1/h2 rule `rgb(236,238,240)`, title `#000` in both builds.

At 375×812 the page has no horizontal scroll.

## Headings

| Level | Logseq | lsg |
| --- | --- | --- |
| h1 (`1Password/Account/Service`) | 32px, 300, 1px rule `rgb(38,41,43)`, margins 4px | the same |
| h2 | 24px, 300, 1px rule, margins 3px | the same |
| h3 | 19.2px, 600, top margin 5.76px, no rule | the same |
| h4, h5, h6 (`rulesync/log/25/10/04 Sat - Issues with gitignore`) | 16px, 13.28px, 12px at 600 | 16px, 13.28px, 12px at 600 |

## Tasks

`AI/Coding/Todos`: DOING and both TODO blocks show a disabled unchecked checkbox and their label; the DONE block shows a disabled checked checkbox and struck-through text. The `logbook` text after two items is the drawer rendering that #112 covers.

## Collapse and zoom

- `rw-cli` shows its authored collapsed blocks with the filled closed bullet and no native triangle. Two of Logseq's four closed blocks render open in `lsg`, because their `collapsed:: true` follows a child block (design question 11).
- `rw-cli#zoom/1.1.1.2` (the collapsed table block) shows `rw-cli › rw-cli - An open-source Python CLI by Scarvy that interacts with the Readwise R… › Commands › rw-cli list - list items in Readwise/Reader`, the block expanded, and hides the title and Linked References. Logseq shows the same breadcrumb for this block.
- Clicking that block's bullet on the page sets `#zoom/1.1.1.2`; Back restores the page view with the same two blocks closed.
- `tests/outline.test.cjs` covers zoom by path and by UUID, the `⋯` breadcrumb, restoring reader collapse, unknown addresses, the hover arrow, the guide strip, content swapped into `main`, and feed bullets opening the journal page. The local Gitpa checkout does not call `lsg`, so the swapped-content case rests on that test.

## Size

| Measure | Budget | Result |
| --- | --- | --- |
| `lsg-outline.js` | 8 KB | 6,885 bytes |
| Outline styles, gzip | 1.5 KB | under 0.9 KB |
| `tmux` alone (`--include tmux`), gzip | +2 KB | 52,116 → 52,855 bytes (+739) |
| `tmux` in the whole-garden build, gzip | under 100 KB | 95,156 bytes |
| Script before the title on `tmux` | 16 KB | 9,012 bytes |

## Zoom time

`parity:perf` on `tmux`, one load per build and profile, clicking the bullet of a nested block halfway down the page:

| Profile | Logseq | lsg | Budget |
| --- | ---: | ---: | ---: |
| desktop | 165 ms | 16 ms | 50 ms |
| phone | 258 ms | 25 ms | 150 ms |

The same run measured `tmux` at 97 KB before the title on the desktop profile, under the 100 KB budget. The rows are in `docs/parity/performance.md`.
