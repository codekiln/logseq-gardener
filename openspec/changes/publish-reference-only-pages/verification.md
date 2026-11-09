# Publish reference-only pages verification

Measured on 2026-11-08 on an Apple Silicon Mac. Both builds published the encode garden at commit `66b81e9543` with `mise run parity:build`'s options: `--filename-format triple-lowbar --journals --search` and all 1,065 top-level namespaces. "Before" is `lsg` 0.0.41 built from `f64db09`, the main commit this change started from; "after" is this change.

## Unresolved references

| Diagnostic | Before | After |
| --- | ---: | ---: |
| `Unresolved or excluded page reference` | 8,136 | 2 |
| `Unresolved or excluded tag` | 1,230 | 0 |
| All diagnostics | 12,555 | 3,191 |

The two that remain are the withheld pages `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav` (journal `2026_10_22`) and `GitP/A/Session/26/10/22-Thu/Asset/Audio/Trimmed/wav` (journal `2026_10_23`), which carry `public:: false`. No reference-only page was left out because of a route collision.

The build publishes 4,656 reference-only pages, 34 of them journals, beside the same 9,202 pages and 520 journals.

## Fixture pages from the analysis

| Gap | Fixture | Result |
| --- | --- | --- |
| 1 | `[[AI]]`, `[[Ableton/Live]]`, `created-by:: [[Person/Cal Newport]]` | Link to their generated namespace pages with the Namespace link style; no diagnostic. |
| 2 | `[[Deno Deploy]]` on `Deno` and on `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` | Links to `Deno Deploy`, whose page reads `2 Linked References` and lists both blocks. |
| 3 | `#10977)`; `company` in `tags::` of `Sourcegraph` and 17 other pages | Pages `10977)` and `Company`; the second shows `Pages tagged with "company"` with all 18 pages. Logseq's build titles it `Company` too. |
| 4 | `(C#/D♭)` | The tag links to `D♭)`; no page `/D♭)` exists. |
| 5 | Names in several spellings | All 4,548 reference-only titles that Logseq's build also holds have Logseq's exact spelling; see Titles below. |
| 6 | `1Password/Desktop/Settings/Developer/SSH Agent/Advanced` | It and its three missing parents are reference-only pages; `1Password/Desktop/Settings` lists the block on `1Password/Dev/ssh/Tutorial/Setup Walkthrough`. |
| 7 | Withheld `GitP/.../wav` names | Stay plain text with their diagnostic; no page. |
| 9 | `[[Nov 18th, 2024]]` | The reference-only journal `2024-11-18 Mon`, in the garden's journal title format, listed as a journal in All pages and in the journals feed. |
| 10 | `[[MacOS /Applications Folder]]` | `MacOS /Applications Folder` and `MacOS ` are reference-only pages at their `p-<digest>.html` addresses; `MacOS ` keeps its space and shows no Hierarchy, since the namespace `MacOS` is another page. |

## Comparison with Logseq's build

`parity:compare` against Logseq's publish-spa build of 2026-10-25 (garden `083106dc`, the `logseq` folder of run `20261025T043101Z`), which holds 17,412 pages:

| Group | Before | After |
| --- | ---: | ---: |
| Reference-only pages in Logseq's build and not in `lsg` | 6,595 | 2,046 |
| Pages only in `lsg` | 3,515 | 3,594 |

None of `Deno Deploy`, `company`, `10977)`, `D♭)`, `Typescript` and `Nov 18th, 2024` remains in the reference-only group. The 2,046 that remain are out of scope: 1,792 aliases, which Logseq stores as pages of their own and `lsg` resolves to their page; 83 property names (`:property-pages/enabled?`); 12 of Logseq's built-in pages; 157 block UUIDs written as `[[uuid]]`; and 2 names (`Microfreak/Oscillator`, `Microfreak/Oscillator/Timbre`) that today's garden no longer writes.

### Titles

Of the 4,622 reference-only page titles that are not journals, 4,548 match a page in Logseq's build exactly, 73 are not in that build because the garden changed after 2026-10-25, and 1 differs: `lsg` titles `Description` where Logseq titles the page `description` from the property name, a property page this change leaves out. `Typescript`, `MacOS` and `langgraph` turned out to be generated namespace pages, not reference-only pages; generated namespace titles keep their existing rule, and 4 of them differ from Logseq (`langgraph`/`LangGraph`, `Rulesync/Skill`/`rulesync/Skill` and two `GItP/A/Log` prefixes).

### Names the SDK finds

The SDK finds names from the names each file's parse already holds. To check them against the reference index, a debug build compared the 4,656 pages with the blocks that refer to each: one page, `Adam Murray/Max for Live`, has no referring block. A block property `alias::` writes it, which the SDK reads as a written name while the page's property row does not count it as a reference.

## Sizes on the encode garden

| Part | Before | After | Budget |
| --- | ---: | ---: | ---: |
| Whole site | 517.3 MB, 50,594 files | 579.7 MB, 65,850 files | 600 MB |
| Page HTML at the site root | 207.4 MB | 252.2 MB | 260 MB |
| `_refs/` | 87.7 MB | 103.4 MB | 115 MB |
| Linked References files of reference-only pages | | 6.9 MB | 10 MB |
| Unlinked References files of reference-only pages | | 7.2 MB, at most one file each | 15 MB |
| Search title index | 245 KB gzip, 769 KB decoded | 147 KB gzip, 597 KB decoded | 300 KB gzip, 1.5 MB decoded |
| All pages list | 185 KB gzip | 232 KB gzip | 300 KB gzip |

`Deno Deploy` is 9,087 bytes, 3,259 gzip; the median reference-only page is 9,055 bytes, 3,265 gzip. Before `MacOS ` lost its Hierarchy section it was the only page over 11 KB (12,877 bytes, 3,784 gzip).

## Build time

Three alternating builds of each binary, `--timings`:

| Build | Runs (s) | Median (s) |
| --- | --- | ---: |
| Before | 18.50, 19.41, 19.24 | 19.24 |
| After | 19.87, 20.47, 20.16 | 20.16 |

The median is 4.8% slower and the fastest 7.4% slower, within the 10% `build_slowdown_pct`. `parity:publish-bench --corpus encode-garden --runs 3 --record` recorded a warm build of 19.42 s with a peak of 3,407 MB in `docs/parity/publication-speed.json`.

## First paint of Deno Deploy

`parity:perf` with three loads per profile, empty cache:

| Profile | Build | Title (ms) | Data before title | Script before title | Peak JS heap |
| --- | --- | ---: | ---: | ---: | ---: |
| Desktop | Logseq | 14,570 | 18.7 MB | 8.1 MB | 2,561 MB |
| Desktop | lsg | 18 | 13.0 KB | 11.6 KB | 1.8 MB |
| Phone | Logseq | 144,040 | 18.1 MB | 8.1 MB | 1,548 MB |
| Phone | lsg | 412 | 13.0 KB | 11.6 KB | 1.8 MB |

The `lsg` values are within the `[desktop]` and `[phone]` budgets: 100 KB of data and 16 KB of script before the title, 300 ms on desktop, 1,500 ms on the phone profile and 10 MB of heap.

## Search on the encode garden

Measured 2026-11-09 with `lsg` 0.0.42 (`a25b3a1`, which includes this change) on garden commit `66b81e9`, against Logseq's 2026-10-25 build: `parity:perf` with `deno deploy` added to `scripts/parity/search-queries.txt`. The Search sections of [lsg/Ref/Logseq Compatibility/Performance](../../../docs/pages/lsg___Ref___Logseq%20Compatibility___Performance.md) hold every row, including the new Search per query table.

| Profile | Build | First page results (ms) | `deno deploy` keystroke p95 (ms) | `deno deploy` downloaded (KB, gzip) | First page result for `deno deploy` |
| --- | --- | ---: | ---: | ---: | --- |
| Desktop | Logseq | 2,921 | 1,337 | 0 | |
| Desktop | lsg | 137 | 32 | 108.8 | `Deno Deploy` |
| Phone | Logseq | 11,889 | 10,243 | 0 | |
| Phone | lsg | 2,048 | 436 | 103.4 | `Deno Deploy` |

Over the whole query list on the phone profile, `lsg`'s cold keystrokes reach 508 ms at the 95th percentile, within the 1,000 ms budget, and the heap grows 6.3 MB, within 30 MB. Warm keystrokes reach 128 ms against the `[phone]` budget of 100 ms. That excess predates this change: the same measurement on a site published by `lsg` 0.0.41 (`f64db09`), twice each, alternating, gave 105 and 106 ms before and 129 and 133 ms after. Warm keystrokes that fetched nothing took 50 to 51 ms before and 56 to 57 ms after. Before this change, the first page result for `deno deploy` was `LangSmith/Deployment/Concept/Deployment Source`, because no page `Deno Deploy` existed.

## Contrando

`Contrando`'s file publishes it as an ordinary page. Its rows on the Performance page came from `lsg` 0.0.0 on 2026-10-26 and are replaced by the 2026-11-09 run: 20.5 ms to the title on desktop and 412 ms on the phone profile, with 15.2 KB of data and 14.0 KB of script before the title, and a peak heap of 1.8 to 1.9 MB, within the `[desktop]` and `[phone]` budgets.

## Build peak memory

`parity:publish-bench`'s measurement on the encode garden at `66b81e9`, with `--journals`, `--search` and every top-level namespace. The local figures come from `lsg` 0.0.41 (`f64db09`) and this change's build, alternating, two rounds of one cold and two warm builds each; the GitHub figures from the `publication-speed` job on main before and after this change merged (runs 42521212918 and 42534438112, garden commit `5095bdf`).

| Machine | Before (MB) | After (MB) | Growth | Budget (MB) |
| --- | ---: | ---: | ---: | ---: |
| GitHub runner, Linux | 2,196 | 2,372 | +8% | 2,700 |
| Apple M2 Pro, macOS | 3,115 to 3,117 | 3,398 to 3,405 | +9% | 3,900 |

The budgets are `encode_garden_peak_mb_linux` and `encode_garden_peak_mb_macos` in the `[publication]` table of `scripts/parity/budgets.toml`, about 14% above the after figures; `parity:publish-bench --check`, which CI runs, fails a build that peaks above the budget for its system. The two systems count resident memory differently, hence two budgets. The memory is the build process's; a reader's browser loads none of it.

In the same local rounds the warm builds took 17.1 and 18.1 s before and 18.5 and 21.3 s after; the `parity:perf` run's fastest of three took 17.7 s before and 20.3 s after. Another build was running on the machine during these rounds, so these times are less reliable than the three alternating builds under Build time above.

## Graph view and right sidebar

`tests/graph.test.cjs` opens `Deno Deploy` from the graph view's Search list and from a Page graph card, and shows `TODO — not published` for `TODO` in both, reading data files that the Rust encoder writes (`reference_only_javascript_fixtures_match_the_encoder`). `tests/sidebar.test.cjs` Shift-clicks a link to `Deno Deploy` on `Deno` and gets a page card with its Linked References header while the page stays on `Deno`.
