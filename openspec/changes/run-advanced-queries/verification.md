# Verification

Measured on 2026-11-08 with lsg built from this branch, Logseq 0.10.15 (`parity:logseq-export`, frontend from `Logseq.app`) and gitpa `410f4d9`.

## Step count and limits (task 2.5)

- Sorting all 16,693 rows with gitpa's comparator and showing the first 1,000 through a view takes 4,169,942 steps, 161 ms in a release build. The design estimated 5 million steps; the 20,000,000 step limit leaves almost five times the room (`sorting_every_page_of_the_encode_garden_fits_the_step_limit`).
- A chain of 40 `concat` forms that doubles a collection each time stops at the step limit before it allocates, in well under 2 seconds in a debug test build (`a_runaway_function_reaches_the_step_limit`). Self-application stops at the 64-call depth limit.

## Logseq 0.10.15 against lsg (tasks 2.2 and 4.1)

`mise run parity:logseq-export --garden tests/fixtures/parity-garden -- --filename-format triple-lowbar --include Tools --journals` built Logseq's own export of the fixture garden. Its report lists differences on pages that already differed before this change (property rows on `Tools/Code`, `Tools/Embeds`, `Tools/tmux`), on the new typed pages (the same property-row difference), and words on `Tools/Queries`. The advanced query boxes on `Tools/Queries`, read from Logseq's rendered page with Playwright, give:

| Box | Logseq 0.10.15 | lsg |
| --- | --- | --- |
| Typed pages (transform and view) | `3 results`: One, Five, Six | the same three links in the same order, with `Five` (`+02:00`) before `Six` as strings sort |
| Without a view (`reverse`) | `3 results`: Six, Seven, One | the same rows in the same order |
| Matches nothing (with a view) | `No matched result` | `No matched result` (the analysis expected an empty body; the build now matches Logseq) |
| Transform that throws | `6 results`, untransformed | the same rows, plus one build diagnostic |
| View that throws | `1 result` above `Custom view failed:` and `Cannot compare a to 1` in an `#error` map | `1 result` above `Custom view failed:` and `Cannot compare a to 1` |
| Function outside the subset (`partition`) | runs, shows `No matched result` | `Query failed:` naming `partition`, by design |

A second probe page, built the same way and removed afterwards, printed 42 expressions with `pr-str` in Logseq. They are recorded as `matches_values_recorded_from_logseq_0_10_15` in `query_script.rs`. They covered `compare` for `nil`, an astral character against U+FF5E, booleans, keywords, vectors and mixed time zones, `sort-by` with a boolean comparator, a nil value against a default, `str`, `into`, `replace` with `$$` and `$&`, `join`, and `<` between a string and a number, which gives `false` and does not throw. The one difference found was `<` on mixed types, now fixed.

## Gitpa's homepage (task 5.1)

`mise run site:query:check` in gitpa lists 15 episodes, newest first, from `gitp/a/session/26/11/06-fri` to `gitp/a/session/26/09/25-fri`. The same prepared garden, published with plain `lsg publish` (`--homepage "Ghost in the Patch" --readable-urls --redirect-old-urls`, one `--include` per public page, no `nbb-logseq` and no query rows written afterwards), shows `15 results` and the same 15 links in the same order, each linked to the episode's published page. Gitpa's `scripts/prepare_site.py` and `scripts/build_site.py` were imported read-only; nothing in gitpa changed.

## Budgets

- `mise run parity:budgets` passes. The gitpa-form page `Tools/Queries/Advanced` (18 episode rows) grows by 331 bytes gzip over the same page without the query (budget 3 KB), its view is 1,341 bytes, it links the same scripts and stylesheets as `Tools/Queries`, and holds no element that runs code.
- Whole-garden build, on a copy of the encode garden pages without git history (9,150 published pages), best of three warm builds with `--journals --search`: this branch 17.5 s without the two query pages and 16.7 s with them; `origin/main` 16.4 s and 16.7 s. The first figure includes a colder file cache (the read phase took 1.6 s against 0.6 s). With the query pages, the render phase took 5.9 s against 5.6 s on `origin/main`. The slowdown is within the `[site]` and `[advanced_queries]` 10%.

## Whole garden in a browser, against Logseq (tasks 4.3 and 5.2)

Measured on 2026-11-08 on an Apple M2 Pro with `mise run parity:perf --advanced-queries --loads 3 --record --baseline-lsg <lsg built from a6ea102>`, on encode garden commit `a7d948e` with `lsg` `154e0ef`. The run adds `Parity/AdvancedQueries` (the gitpa-form query over the six proxy pages) and `Parity/AdvancedQueries/All Pages` (every page, sorted by `tags`, a view of the first 1,000) to a copy of the garden, and builds the copy with `lsg` and with Logseq 0.10.15's own export, as `parity:logseq-export` builds it. The `gh-pages` branch of the encode garden now holds an `lsg` build, so it could not serve as Logseq's side. The Advanced queries section of the Performance page (`docs/pages/lsg___Ref___Logseq Compatibility___Performance.md`) holds every row; medians of three loads:

| Page | Profile | Build | Title (ms) | First row (ms) | Peak heap (MB) |
| --- | --- | --- | ---: | ---: | ---: |
| `Parity/AdvancedQueries` | phone | Logseq | 191,193 | 191,782 | 2,085 |
| `Parity/AdvancedQueries` | phone | lsg | 414 | 227 | 1.9 |
| `Parity/AdvancedQueries/All Pages` | phone | Logseq | 198,196 | 200,656 | 2,128 |
| `Parity/AdvancedQueries/All Pages` | phone | lsg | 402 | 217 | 2.1 |

Against the budgets:

- Time to title: 19 and 30 ms on the desktop profile (budget 300 ms), 414 and 402 ms on the phone profile (budget 1,500 ms).
- Data and script before the title: 15.0 and 34.4 KB gzip (budget 100 KB) and 13.6 KB of script (budget 16 KB) on both pages. The all-pages page carries its 1,000 links in its HTML; the proxy page is no larger than other pages with a query.
- Phone heap: 1.9 and 2.1 MB at the peak (budget 10 MB), 1.2 and 1.3 MB after garbage collection with the rows shown. Logseq's peak was about 2.1 GB on both pages, because it loads the whole graph before it runs the query.
- Search with the pages in the corpus, per keystroke: warm p95 23 ms on the desktop profile (budget 50 ms) and 47 ms on the phone profile (budget 100 ms); keystrokes that fetched a shard, p95 45 and 416 ms (budgets 150 and 1,000 ms). Logseq's p95 was 1,727 and 13,692 ms.
- The all-pages query costs 95 ms of build time (budget 500 ms), measured as the growth of the render phase with the query ten times on one page, divided by ten. 9,674 published pages match, not the 16,693 the design counted: the design counted every page Logseq knows, including pages that are only referenced, and the query runs over published pages.
- A transform that reaches the 20,000,000-step limit stops after 0.08 s on the whole garden (budget 2 s).
- Whole-garden build with both pages: 16.69 s, against 16.67 s for `lsg` built from `a6ea102`, before advanced queries ran (+0.1%, budget 10%).

## The gitpa-sized corpus (task 4.4)

`scripts/parity/sample_garden.py` version 2 adds 15 episode pages and the homepage `Ghost in the Patch` with gitpa's homepage query, and `parity:publish-bench` publishes the corpus with `--homepage "Ghost in the Patch"`. The homepage shows `15 results`. The `publication-speed` job of the pull request's first CI run (run 42476238310) recorded the GitHub row (AMD EPYC 9V74, warm 4.29 s), copied from its artifact into `docs/parity/publication-speed.json` as the earlier GitHub rows were, beside a laptop row (Apple M2 Pro, warm 2.98 s, the same as generator version 1). Later CI runs check the corpus against these rows.
