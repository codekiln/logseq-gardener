# Verification: match-logseq-references

`lsg` now shows Logseq's four sections below the outline with Logseq's counts. On the encode garden `mise` reads `4 of 589 Linked References` with the green funnel, in 222 groups, and lists 9 pages tagged with it; `tmux` has 795 linked references in 258 groups and 856 unlinked references. These are the numbers Logseq's own build of the same garden commit gives. The section bodies moved to fragment files, so the largest pages shrank from about 2.2 MB to under 110 KB.

Measured on 2026-10-26 against the `gh-pages` build `8132c95` of garden commit `083106d`, which is the garden's current `main`. Logseq's counts come from its `db.gz` (blocks whose own references name the page, outside the page) and from its rendered page in headless Chromium; `lsg`'s come from a whole-garden build of this branch (`parity:build`'s options) served locally.

## Counts

| Page | Check | Logseq | `lsg` |
| --- | --- | --- | --- |
| `mise` | Linked header on load | `4 of 589 Linked References`, green funnel | Same |
| `mise` | Referring blocks, pages | 589, 222 | Same |
| `mise` | First groups | `2026-10-19 Mon`, `2026-10-04 Sun`, `2026-09-10 Thu` | Same |
| `mise` | Filter options with `Diataxis/How To` included | `Diataxis` 4, `AWS` 2, … (21 chips) | `Diataxis` 4, `AWS` 2, … (15 chips; see below) |
| `mise` | Filter options with no filter | `mise/Config` 106, `mise/Config/mise.toml` 74, `Gard` 66 | `mise/Config` 103, `mise/Config/mise.toml` 71, `Gard` 66 |
| `mise` | Pages tagged with | 9 | Same |
| `tmux` | Referring blocks, pages | 795, 258 | Same |
| `tmux` | Unlinked references, pages | 856, 250 | Same |
| `McKinsey` (`filters:: {}`) | Linked header | 5 blocks on 4 pages, unfiltered | `5 Linked References`, 4 groups, grey funnel |
| `Term` | Pages tagged with | 178 | Same |
| `Term` | Referring blocks | 192 in `db.gz`, 193 live | 193 |
| `Claude/Code` | Referring blocks | 616 in `db.gz` own references, 657 live with aliases | 638 |
| `Claude/Code` | Tagged heading | `Pages tagged with "claude/code"`, 38 | Same |
| `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter` | Linked References | no section | Same |

Unlinked references on `mise` are 2,720 in Logseq and 1,317 in `lsg`. The 2,344 Logseq matches on pages both builds publish include blocks that Logseq reads from files `lsg` does not load into those pages (for example `Microfreak/Preset/001 NervousKeys`, whose page file never mentions mise); on `tmux`, where every match is on page files, the counts are identical.

## Styles

Computed in headless Chromium at 1280×800 and 390×844, in both themes.

| Element | Logseq | `lsg` |
| --- | --- | --- |
| Linked References heading | 16px, weight 500 | Same |
| Group card, light | `rgb(241,243,245)`, 16px padding, 4px radius, `8px 0` margin | Same |
| Group card, dark | `rgb(32,36,37)` | Same |
| Group card at 390 wide | 14px padding, 3.5px radius, `7px 0` margin (Logseq scales its root font on phones) | 16px, 4px, `8px 0` |
| Breadcrumb | 16px, `8px 0 8px 24px` (from the analysis; collapsed on load here) | Same |
| Pages tagged with heading and list | 16px, weight 700, opacity 0.5; `circle`, `8px 0 0 19.2px` | Same |
| Funnel with no filter | text colour at 60% opacity | Same |

Screenshots of `mise` and `tmux` in both builds were taken at both sizes and themes during the check; no page in the fixtures scrolls horizontally at 390 wide.

## Size and build time

| Measure | Before (`origin/main` `84e9278`) | After |
| --- | --- | --- |
| `Answer/AI` page | 2.2 MB | 26 KB |
| `note/my` page | 2.1 MB | 27 KB |
| `mise` page | 511 KB | 70 KB (outline and hierarchy; no groups) |
| `Claude/Code` page | — | 106 KB raw |
| Fragment files in `_refs/` | — | 13,653 files, 312 MB |
| `lsg-references.js` | 5 KB inlined in every page | 5,980 bytes, loaded after `load` |
| References styles, gzip | — | 976 bytes |
| Whole-garden build, this Mac (`parity:perf`'s timing) | 5.2 s | 8.1 s |

On `mise`, opening Linked References shows its first group in 22 ms on the desktop profile and 346 ms on the phone profile (budgets 300 and 1,500); a filter change takes 46 ms on the desktop (budget 50); after opening both sections and scrolling to the end, the phone heap peaks at 3.4 MB (budget 20). The rows are in [`docs/parity/performance.md`](../../../../docs/parity/performance.md). Only `lsg` was measured: the full `parity:perf` run spends about 20 seconds on each Logseq page load and would have taken over four hours, so the page-load rows keep their earlier values.

`parity:budgets` passes: script, styles, every fixture fragment under 100 KB gzip, and every fixture page unchanged when 2,000 pages are added.

The build is about 56% slower than before, over the 25% budget. Profiling shows the time goes to creating files: `open` calls dominate after rendering, because the build now writes 13,600 more files. The first layout (a directory and two JSON files per page) took 14 seconds; the flat layout with the data inside the first fragment (design, question 9) and writing on eight threads brought it to about 8.

## Differences that remain

- **Property groups.** Logseq shows the referring page's property rows (`tags: Mise, Diataxis/How To`); `lsg` shows them only when the garden enables page properties, and otherwise a `Page reference` link. The encode garden's settings give the link.
- **Group order for pages.** Logseq orders non-journal groups by database order; `lsg` orders them by title (design, question 1).
- **Filter options.** Logseq also offers property names and pages without a published page as chips (design, question 2), so its lists are longer.
- **Funnel position.** Logseq pushes the funnel to the right edge of the header; `lsg` places it after the heading.
- **Alias label.** `lsg` publishes an alias as another name for the same page, so no group is an alias page and the `Alias` label never appears.
