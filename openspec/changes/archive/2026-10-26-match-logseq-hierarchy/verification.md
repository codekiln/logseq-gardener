# Verification: match-logseq-hierarchy

`lsg`'s Hierarchy section now lists the same rows as Logseq's published site, in the same order, with Logseq's look in both themes, except where #91 deliberately leaves out namespace pages that have no file. Pages without a namespace have no section, a leaf shows its parent's path, and `AGI` shows its alias's hierarchy. The page holds at most 100 rows, so `Person` drops from 508 KB to 69 KB and `Microfreak` from 972 KB to 94 KB; scrolling loads the rest from `_refs/` fragments.

Measured on 2026-10-26 against the `gh-pages` build of garden commit `083106d`, served locally and opened with Playwright's Chromium at 1280×800 and 390×844 in the light and dark color schemes. The `lsg` builds are `mise run parity:build` of the whole encode garden from this branch and, for "before", the same options with `lsg` built from `origin/main` at `c373d90`.

## Rows on the fixture pages

Logseq counts every row; `lsg` counts the rows after scrolling the list to its end. "Shared order" means the rows both builds show appear in the same order.

| Fixture | Logseq rows | `lsg` rows | Shared order | Notes |
| --- | ---: | ---: | --- | --- |
| `tmux` | 200 | 160 (100 in the page) | Same | Logseq's extra rows are namespace pages without a file, such as `tmux/Bug` (#91), and reference-only pages |
| `tmux/Config` | 1 | 1 | Same | `tmux/Config/local`; the own row is gone |
| `tmux/Config/local` | 1 | 1 | Same | The parent `tmux/Config` |
| `tmux/Command` | 19 | 12 | Same | The 7 missing rows are reference-only pages |
| `AGI` | 1 | 1 | Same | `AI/General`, from the alias `AI/General/Intelligence`; `General` is plain text in `lsg`, see below |
| `Abbreviation` | 0 | 0 | Same | No section in either build |
| `GitP/A/Session/26/10/20-Tue` | 40 | 24 | Same | Logseq's extra rows are file-less namespace pages such as `…/Asset` |
| `Person` | 2,358 | 1,067 (100 in the page) | Same | All 1,067 loaded on scroll |
| `Microfreak` | 2,548 | 1,394 (100 in the page) | Same | All 1,394 loaded on scroll |

Every `lsg` row also appears in Logseq's list. A comparison script that joined only the linked segments reported three `Person` rows such as `Person/GitHub/producer-pal`; the full row is `Person/Adam Murray/GitHub/producer-pal`, whose `Adam Murray` segment is unavailable plain text.

## Look

Computed styles on `tmux` at 1280×800:

| Element | Logseq | `lsg` |
| --- | --- | --- |
| Section | `margin-top` 24px | Same |
| Heading | 16px, weight 700, opacity 0.3; light `rgb(17,24,28)`, dark `rgb(236,237,238)` | Same |
| List | margin `12px 24px`, padding 0, `circle` markers | Same |
| Row | margin `8px 0` | Same |
| Separator | margin `0 8px`, opacity 0.3 | Same |
| Segment | page reference with grey `[[` `]]` | Same; `lsg` draws the brackets with `::before` and `::after`, as its inline references do |

At 390×844 Logseq's root font is 14px, so its rem-based margins become 21px, 7px and 7px while `lsg` keeps 24px, 8px and 8px. The same difference applies to every Logseq rem value on phones and belongs with the app shell, not this section.

Links to generated namespace pages keep their dotted underline and `(Namespace)` accessible name; no row shows a `Namespace` badge.

## Page size

Bytes of the page's HTML, with gzip in parentheses.

| Fixture | Rows in the page, before | After | Page before | Page after |
| --- | ---: | ---: | ---: | ---: |
| `tmux` | 161 | 100 | 99,159 (13,919) | 77,344 (12,044) |
| `tmux/Config` | 2 | 1 | 27,536 (7,701) | 27,437 (7,683) |
| `tmux/Config/local` | 1 | 1 | 27,068 (7,647) | 27,021 (7,622) |
| `tmux/Command` | 13 | 12 | 31,467 (7,841) | 32,309 (7,816) |
| `AGI` | 1 | 1 | 25,880 (7,338) | 26,062 (7,378) |
| `Abbreviation` | 1 | 0 | 26,035 (7,410) | 25,736 (7,341) |
| `GitP/A/Session/26/10/20-Tue` | 25 | 24 | 152,472 (25,486) | 160,326 (25,538) |
| `Person` | 1,068 | 100 | 508,078 (52,839) | 69,280 (12,505) |
| `Microfreak` | 1,395 | 100 | 971,659 (60,798) | 94,462 (11,984) |

Pages with short lists grow by up to 8 KB uncompressed because each segment now carries the page-reference wrapper; the gzip size changes by less than 100 bytes.

## Budgets

`mise run parity:budgets` passes: hierarchy styles 195 bytes gzip (budget 500), `lsg-references.js` 6,695 bytes (5,995 before plus at most 1,000), no page over 100 rows, every hierarchy fragment under 100 KB gzip, and the root of a fixture namespace with 200 pages is the same size with 2,200. The reference-section styles stay at 973 bytes gzip once the hierarchy rules, which have their own budget, are counted separately.

`mise run parity:perf --builds lsg --baseline-lsg <origin/main lsg>` on the whole garden: scrolling `Microfreak`'s list to its end loads 1,394 rows with a peak JS heap of 3.1 MB on the phone profile (budget 20 MB). A whole-garden build takes 10.5 seconds against 10.8 seconds for the baseline (−3%, budget +10%). Hierarchy rows add nothing to the search corpus. The rows are in [`docs/parity/performance.md`](../../../../docs/parity/performance.md).

## Known differences

- **Rows for namespace pages without a file.** Left out on purpose, as #91 asked (design, question 1).
- **Reference-only pages.** Logseq lists pages that exist only because a link names them; `lsg` does not publish those pages yet (non-goal, #111 and #115).
- **Alias-implied namespace pages.** On `AGI`, Logseq links `General` because its database creates `AI/General` from the alias. `lsg` creates namespace destinations only from page titles, so `General` is plain text. This belongs with reference-only pages.
- **Shift-click** to open a segment in the right sidebar waits for #119.
