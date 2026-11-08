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

## Not measured in this change

- The phone heap, the per-keystroke search latency and the page's time to title in a browser, in `lsg` and in a Logseq build of the same copy (`parity:perf --advanced-queries`, task 4.3). The page loads no script or stylesheet beyond what any query page loads, and the `Tools/Queries/Advanced` check above found no difference in linked files, so no change is expected.
- The `gitpa-sized` corpus row and the CI check for it (task 4.4). Changing the corpus changes its digest, so `parity:publish-bench --check` would find no recorded row for GitHub's runners until one is recorded from CI.
