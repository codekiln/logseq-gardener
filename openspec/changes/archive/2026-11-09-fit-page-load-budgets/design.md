# Design

## Context

Issue #591 lists the values the #590 benchmark left over budget. Reproducing them on the encode garden at `f1c1146` with `lsg` built from `e5b6479` (`parity:perf --builds lsg --loads 3` on `tmux`, `Foam`, `All pages`, `Contents` and the feature sections) found:

- `All pages` loads `lsg-all-pages.js` (5.4 KB) from a `<script defer>` tag, so the browser fetches it during parsing, before the title shows: 17.0 KB against 16 KB. The fixture check in CI missed it because `check_search` skips All pages and the fixture garden has no page-property start-up (1.4 KB), which every encode garden page carries.
- `Contents` read 11.6 KB in this run and 26.6 KB in #590. `lsg-shell.js` loads `lsg-outline.js` and `lsg-references.js` (15.0 KB together) on the `load` event. `Contents` is nearly empty, so on desktop `load` can fire before the frame that paints the title, and those requests then count as script before the title. Whether they do depends on timing.
- The filter change on `mise` took 58.6 ms in the harness and about 44 ms in a five-run probe. A CPU profile of the click shows about 31 ms in `focus()`, which forces the style and layout of every reference group that the filter showed or hid, most of them off screen, and 11 ms rebuilding the chips.
- The phone's warm keystroke 95th percentile was 153 ms in the harness and 155 ms in a probe. The two slowest warm keystrokes (171 and 155 ms) each came 300 ms after a cold keystroke that took 486 and 460 ms; they ask for the same word shard, which is still loading, and wait for it. Keystrokes that fetched nothing had a 95th percentile of 64 ms.

## Goals / Non-Goals

**Goals:**

- Bring each over-budget value back within budget, or change the budget with a recorded reason.
- Explain the phone title time and the build time, and fix or accept each.
- Make CI fail on the All pages script regression.

**Non-Goals:**

- Draw page results before a missing word shard arrives, so a key typed while its shard loads answers from the title index at once. That is the product change behind the cold keystroke time on the phone; it is its own change to the search dialog.
- Cut the build time back toward 10.5 s.

## Decisions

### Start post-load scripts after the next paint

`lsg-shell.js` already waits for `load`. It now waits one more frame, `requestAnimationFrame(() => setTimeout(after))`, the same pattern the page-property start-up uses after `DOMContentLoaded`. The title's frame paints first, then the scripts are requested. The outline and reference sections are the same a frame later. Loading them only on pages that need them was rejected: a plugin host swaps new content into the page and relies on both scripts being present.

### Load the All pages script after the first paint

All pages drops its `<script defer>` tag, and `lsg-shell.js` loads `lsg-all-pages.js` with the scripts above when the page has an `.all-pages` box. The host's All pages initializer loads the script when it adopts an All pages screen before the script is present. Until the script runs, the static first 40 rows and paging links work, and dates show in UTC as the static HTML writes them; the spec already says dates show in the reader's time zone "when the script runs". On a fast desktop the dates can change once, a frame after the first paint. An inline script that converts the dates before paint was rejected: it adds to the script before the title on a page this change is trying to shrink, and the spec allows the later conversion.

Shrinking `lsg-all-pages.js` by 1 KB instead was rejected: it is already minified, and the page would still sit within 0.4 KB of the budget.

### Skip layout for off-screen reference groups

`.references-blocks-item` gets `content-visibility: auto` with `contain-intrinsic-size: auto 9em`. The browser then lays out only the groups near the viewport, so the work after a filter change no longer grows with the number of groups. A probe that injected the rule measured 9.6 ms against 43.6 ms, and the harness measured 13.5 ms against 58.6 ms with the change built in. The `auto` size keeps each group's last rendered height, and the 9em estimate keeps the end-of-list marker below the fold, so later fragment files still load one at a time as the reader scrolls. Find in page and fragment links still reach hidden groups, since `auto` keeps the content searchable and focusable.

The rule adds 32 bytes gzip and takes the references styles to 1,021 bytes against a 1 KB budget. Rewriting other rules to save the difference made the file larger under gzip. The budget becomes 1.05 KB: since #400 these styles live in the shared `lsg-site.css`, which a visit downloads once.

Moving the chip rebuild out of the click was rejected; it would lower the harness number without lowering the time to the drawn result.

### Count a keystroke that waits for a shard as cold

The query list is typed a key every 300 ms, and on the phone profile a word shard takes 210 to 490 ms. The key after a cold key often asks for the same shard and waits for it. The harness counted it warm because it requested nothing itself, and held it to the 100 ms warm budget. `metrics.keystroke_fetches` now also marks a keystroke cold when a shard requested earlier is still loading at its keydown; it reads each resource's `responseEnd`. Keystrokes that fetched nothing now also exclude these, so that column keeps showing the cost of the query alone.

### Count every script a page loads in CI

`check_search` now covers every page, All pages included, and adds the page-property start-up's size to a page that lacks it. With the old All pages markup, `pages.html` on the fixture search build would count 17.0 KB and fail.

### Accept the phone title time

#400 moved the page styles into the shared `lsg-site.css`, a render-blocking stylesheet. On the phone profile's 150 ms round trip with the cache disabled, the title waits one more request: about 240 ms before, 395 to 410 ms after. #400 recorded this when it merged. The change took the encode garden's site from 1,217 MB to 470 MB, under GitHub Pages' 1 GB limit, and a second page in a visit reuses the cached stylesheet. The phone title time stays under 30% of its 1,500 ms budget. Inlining the styles again would undo the site-size fix, so the regression is accepted.

### Explain the build time

The 10.5 s row came from `lsg` `c373d90` on garden commit `083106d`, built without search. Building both gardens with both binaries, interleaved on the same machine (`--journals`, every root, no search):

| `lsg` | Garden | Seconds | Files written |
| --- | --- | ---: | ---: |
| `c373d90` | `083106d` | 12.2 | 30,570 |
| `c373d90` | `66b81e9` | 11.2 | 32,614 |
| current | `083106d` | 17.2 | 59,341 |
| current | `66b81e9` | 19.0 | 62,545 |

The machine was loaded (load average 12 to 20), so single numbers vary by a few seconds; the 10.5 s row came from a quieter run. The garden grew from 8,569 to 9,205 pages, which adds about 1 to 2 seconds. Most of the increase comes from what `lsg` does now that it did not then: it writes twice the files for the same garden (reference-only pages, Page graph neighbourhoods, compressed reference fragments) and adds the graph layout and search phases. `--timings` on the current garden attributes 7.2 s to rendering, 5.6 s to writing, 2.3 s to indexing, 1.8 s to search and 1.3 s to the graph. Each feature's change recorded its own build slowdown against its budget. The total is accepted.

## Risks / Trade-offs

- [All pages dates show UTC for one frame on desktop] → The spec allows it; the static dates are correct times, only in UTC.
- [A browser without `content-visibility` lays out every group] → It behaves as before this change. Chrome, Edge, Firefox 125 and Safari 18 support it.
- [A full-page screenshot with reference sections open may paint off-screen groups at their estimated size] → `parity:screenshots` is a manual comparison outside CI; a comparison that opens the sections can scroll through them first.
- [The warm keystroke number now excludes keys that waited for a shard] → Those keys are held to the cold budget instead, and the per-query rows still show each query's 95th percentile and last keystroke.

## Resolved Questions

### 1 - Should All pages convert its dates to the reader's time zone before the first paint?

Decided by the agent without codekiln: no. The spec already ties the conversion to the script running, and an inline converter would add script before the title on the page this change shrinks.

### 2 - Should the references styles budget change, or should the filter change budget?

Decided by the agent without codekiln: the styles budget, from 1 KB to 1.05 KB gzip. The 32 bytes go into a shared stylesheet downloaded once per visit and cut the filter change by about 34 ms; the 50 ms filter budget stays.

### 3 - Is a keystroke typed while its shard loads warm or cold?

Decided by the agent without codekiln: cold. It waits for a shard download like the key that requested it, so the cold budget applies; the warm budget is for keystrokes that wait on no shard.

### 4 - Should this change fix the phone title time or the build time?

Decided by the agent without codekiln: neither. The phone title time is the cost of the shared stylesheet that fixed the site size, and the build time is the sum of features that each stayed within their own build budgets. Both are explained above and accepted.

### 5 - Should this pull request re-record the Performance page?

Decided by the agent without codekiln: no. A full `--record` needs Logseq's page loads for every fixture page, which take hours on the phone profile, and a run without them would drop Logseq's rows. The before and after numbers are in `verification.md`; the next full benchmark run rewrites the page.

## Open Questions

None.
