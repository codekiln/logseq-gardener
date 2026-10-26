# Verification: match-logseq-transclusion

`lsg`'s block references and embeds now match Logseq's published site on the fixture pages: a reference shows the target's first line with its markup and checkbox above a dashed underline, and embeds are shaded wells with bullets, collapse and page headers, with the same computed styles in both themes. The transclusion styles are 407 bytes gzip on every page and no script was added. The pages that embed the most grow by at most 2.2 KB gzip and stay far inside the 100 KB first-paint budget.

Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (`gh-pages` `8132c95`), served locally and opened in the desktop app's browser pane, using Logseq's own theme toggle for each theme. The `lsg` builds are `mise run parity:build` of the whole encode garden from this branch and, for "before", from `origin/main` at `b490d68`.

## Fixture pages

"Same" means the two builds gave the same value.

| Fixture | Check | Logseq | `lsg` |
| --- | --- | --- | --- |
| `Claude/Code` | First reference | checked checkbox, struck-through `figure out [[Claude/Code/Bug/Failed to save memory: …]] … #` | Same |
| `Claude/Code` | Reference style, light | text `rgb(17,24,28)`, `0.5px dashed rgb(16,107,163)`, `2px 0`, `cursor: alias` | Same |
| `Claude/Code` | Reference style, dark | text `rgb(236,237,238)`, `0.5px dashed rgb(163,206,241)` | text same, underline `rgb(138,187,187)`; see below |
| `Claude/Code` | Destination | `#/page/<uuid>` | the block's anchor on `Claude/Code/Bug/…` (design, question 1) |
| `Spotify/Tool/TUI` | Block embeds, bullets inside wells | 1, 15 | Same |
| `Spotify/Tool/TUI` | Well, light | `rgb(248,249,250)`, 8px radius, 8px margin, `12px 8px`; inner `4px 12px 8px` | Same |
| `Spotify/Tool/TUI` | Well, dark | `rgb(21,23,24)` | Same |
| `Spotify/Tool/TUI` | Collapse inside the well | arrow folds the embedded heading | Same; the outline script's arrow folds it |
| `AI/Coding/History` | Page embeds, first four headers | 16; `…/Phase/1 - Primitive`, `…/IDE Autocomplete`, `…/Chat Model Copy Paste`, `…/Phase/2 - Basic Single Edits` | Same |
| `AI/Coding/History` | Bullets inside wells | 41 | 42 |
| `AI/Coding/History` | Second-level well | light `rgb(233,236,239)`, dark `rgb(56,67,76)` | Same |
| `AI/Coding/History` | Header | `3.5px` padding, `0 0 5px` margin, weight 600; link 19.2px, weight 700, no underline, light `rgb(16,107,163)` | Same |
| `Py/Zen of Python` | Page embeds | 14 | Same |
| `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` | Two references to quote bullets | both show their quotes | Same, with no diagnostic |
| `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` | Embeds and references | rendered lazily, so not counted | 10 block embeds at levels 1 and 2, 41 references, 16 `Block ref invalid` warnings for 10 uuids that exist nowhere in the garden |

At 390×844 `AI/Coding/History` has no horizontal scroll and nested wells keep their shades; at 1280×800 `Spotify/Tool/TUI` has none either. No fixture page has duplicate HTML IDs.

Logseq's deeper shades were read by nesting `.color-level` elements in its page under each theme: light `rgb(222,226,230)`, `rgb(185,188,190)`, `rgb(170,174,180)` for levels 3 to 5; dark `rgb(64,75,85)`, `rgb(82,95,107)`, `rgb(96,110,122)`. `lsg` uses the same values.

### Differences that remain

- **Dark link colour.** Logseq's dark underline and header link `rgb(163,206,241)` come from the theme the garden's `logseq/custom.css` imports; `lsg` uses its own dark reference colour unless `--stylesheet` loads that file.
- **Counts on long pages.** Logseq leaves collapsed children and the end of long pages out of the DOM until they open or scroll into view, so DOM counts on `Claude/Code` (9 references against 11) and `VSCode/…/03 Basic Python` compare different amounts of the page.
- **Hover previews and shift-click** stay with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).

## Whole-garden fallbacks

| Diagnostic | Before | After |
| --- | ---: | ---: |
| Unavailable or ambiguous block reference | 74 | 55 |
| Unavailable embed target | 7 | 7 |
| Depth or self-embed warnings | 0 | 0 |

The 19 references that now resolve point at quote bullets. No page in the garden reaches the nesting limit of 5.

## Size

First paint is `parity:budgets`' `page_sizes`: the page plus the files it loads before rendering, gzip.

| Measure | Before | After | Budget |
| --- | ---: | ---: | --- |
| Transclusion styles on every page, gzip | — | 407 | 1,000 |
| `Claude/Code` | 148,879 | 149,340 | 100,000; over before this change |
| `Spotify/Tool/TUI` | 12,133 | 12,467 | 100,000 |
| `AI/Coding/History` | 12,743 | 13,567 | 100,000 |
| `Py/Zen of Python` | 11,058 | 11,707 | 100,000 |
| `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` | 9,208 | 9,545 | 100,000 |
| `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` | 28,665 | 30,280 | 100,000 |
| Largest growth in the garden (`Microfreak/UG/03 Overview/01 Front Panel/03 Bottom Row`) | 12,378 | 14,572 | recorded |
| Script before the title, every fixture page | 9,012 | 9,012 | no growth |

Six encode-garden pages were over 100 KB gzip before this change and still are (`Answer/AI`, `note/my`, `Claude/Code`, `Keyshort`, `Update`, `note/AI`); their size comes from their own outlines and linked references, and transclusion adds at most 1.9 KB to any of them. In the fixture garden, `parity:budgets` passes with `Tools/Embeds` holding both kinds of well.

## Performance

`mise run parity:perf` on the same run (Logseq one load per page, `lsg` three) is recorded in [`docs/parity/performance.md`](../../../docs/parity/performance.md); `AI/Coding/History` and `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` now appear there. The run header names `b490d68`, the branch's base, because the build ran from the uncommitted branch tree.

| Page | Profile | Logseq title (ms) | `lsg` title (ms) | `lsg` data before title (KB gzip) | `lsg` peak heap (MB) |
| --- | --- | ---: | ---: | ---: | ---: |
| `AI/Coding/History` | desktop | 19,211 | 23.7 | 15.4 | 1.8 |
| `AI/Coding/History` | phone | 188,363 | 231 | 15.4 | 1.8 |
| `VSCode/…/03 Basic Python` | desktop | 23,196 | 35.3 | 32.1 | 2.0 |
| `VSCode/…/03 Basic Python` | phone | 188,662 | 235 | 32.1 | 1.8 |

Both embedding pages are within every page budget. The two pages marked over budget in that file, `tmux` (101 KB) and `All pages` (271 KB), were over before this change: `tmux` crossed 100 KB with fenced-code highlighting, and `All pages` belongs to [Issue #117 — Publish Logseq's All pages table](https://github.com/codekiln/logseq-gardener/issues/117).
