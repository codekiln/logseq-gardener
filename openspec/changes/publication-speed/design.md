## Context

`lsg publish` runs one SDK function, `site::publish_site_with_options`, which loads the garden, builds its indexes, renders every page, writes search files and graph data, and writes the site. Before this change it reported counts only, so a slow build gave no hint of where the time went.

The repository already measures speed in places. `parity:build-bench` builds a pinned encode garden commit five times per variant in CI and checks the time build plugins add, relative to a build without them on the same runner. `parity:perf` times whole-garden builds against a baseline binary on a contributor's machine. Neither records how long publication takes in absolute terms, on which machine, or which phase dominates.

GitHub's shared runners vary. In the `build-plugins` job's logs from 16 runs on 2026-10-31, the fastest of five encode garden builds without plugins took between 15.5 and 28.8 seconds for nearly the same code. The times fall into groups near 16, 21 and 27 seconds; within a group they spread by about 6%, and within one job two variants of the same build agree to within 2%.

Gitpa's garden on 2026-10-31 had 2,359 pages, 3.3 MB of Markdown and about 300,000 words, with namespaces up to eleven levels deep and about seven page links, seven blocks and eight properties per page. Gitpa publishes with `--search`, `--homepage` and `--stylesheet`. The encode garden has about 8,700 pages, 41 MB of Markdown and 1.9 million words.

## Goals / Non-Goals

**Goals:**

- Show where publication time goes, in a form every SDK caller can read.
- Measure cold and warm publication of a Gitpa-sized garden and the encode garden the same way on any machine, and keep the results.
- Fail CI when publication becomes clearly slower, without failing because the runner was slow.
- Report the bottleneck the measurements show.

**Non-Goals:**

- Speed up the rendering of Linked References, which took 45% of the main thread's samples during rendering in a profile of the encode garden. [Issue #290 · Render each Linked References context once](https://github.com/codekiln/logseq-gardener/issues/290) tracks it.
- Measure Gitpa's own preparation work and shared query support, which [gitpa issue #102](https://github.com/codekiln/gitpa/issues/102) tracks.

## Decisions

### The SDK owns the timings; the publisher records them; the CLI prints them

`logseq_gardener_sdk::timing` holds `Stopwatch`, `Timings` and `PhaseTime`. A stopwatch adds the time since its previous lap to a named phase, so a phase that runs in several intervals is reported once. The loader, `garden::load_garden_timed`, adds time to `read` and `parse`; the publisher passes its own stopwatch to the loader and laps the remaining phases; `SiteReport::timings` returns the result. The CLI only formats the report. A language server or another tool that loads a garden calls `load_garden_timed` with its own stopwatch and reads the same phase names.

The publisher always records timings. A few dozen clock reads cost nothing measurable, and an option would add a field to `SiteOptions` that every caller must fill.

### Phases follow the code's existing order

The phases are `read`, `parse`, `index`, `plugins`, `render`, `history`, `search`, `graph` and `write`. Git history, build plugins and graph layouts already run on their own threads alongside the main thread; their phases measure only the time the main thread waited for them, which is the time they add to the build. To measure the graph wait before writing starts, the publisher now collects the graph data just before creating the output directory instead of between writes. Every phase appears in every report, even with no work, so tools can read a fixed list.

### `--timings` is a flag, and its report sits outside the ordinary output

A flag appears in help and in the usage line, where an environment variable would be hidden. In human output the times go to stderr after the diagnostics, so stdout stays a single result line. In JSON output they join the report as `timings` and `total_ms`, which a script reads without parsing text. The report holds phase names and numbers, never a title or path.

### The benchmark extends the parity tasks

`parity:publish-bench` reuses the parity scripts' release build, namespace selection, git and budget helpers, and follows `parity:build-bench` in counting the fastest of several warm builds, because noise on a shared machine only adds time. It also reports the median. Every build uses `--journals` and `--search`, so the search files and journal pages are always measured.

The cold build is the first build after the garden is written or checked out. The operating system's file cache is not cleared, which needs administrator rights, so the cold time measures a fresh process rather than a fresh disk.

### A generated garden stands in for Gitpa's

`scripts/parity/sample_garden.py` writes a garden from a fixed seed with Gitpa's shape and size, including pages that must say `public:: true` and a few that are withheld, and commits it with a fixed date so the git history step has work. The benchmark records its digest. Generating it takes a few seconds and keeps 3 MB of generated text out of the repository.

### Recorded rows and a limit per processor model

Rows live in `docs/parity/publication-speed.json`. The check compares the fastest warm build with rows for the same corpus identity and runner. With rows for the same processor model, the limit is the slowest of them plus 20%, more than three times the spread observed within one group of runs. Without them, the limit is the slowest row for the corpus plus 100%, wide enough for the gap between the fastest and slowest runner groups. A new processor model therefore catches only a doubling until a contributor records a row for it.

### A separate CI job

The `publication-speed` job runs beside `ci` and `build-plugins`, so it adds no time to the other checks. It runs both corpora with `--check` and uploads its rows, which is how GitHub rows are recorded. `mise run ci` stays unchanged: a whole-garden benchmark would add minutes to every local run.

## Risks / Trade-offs

- [GitHub adds a processor model with no recorded row] → The wide limit still catches a doubling, and the job prints the model so a contributor can record a row from the uploaded results.
- [The pinned encode garden commit changes] → The workflow comment says to change the commit and the rows together; until then the check reports the encode garden as not checked.
- [The generator changes] → Its digest changes, the check reports the corpus as not checked, and new rows must be recorded; `generator_version` in each row marks the change.
- [Fastest of five hides a slower median] → Rows keep the median beside the fastest time, so a widening gap is visible when rows are reviewed.

## Resolved Questions

### 1 - Where should the timing hooks live?

Decided by the agent without codekiln: in the SDK. The `timing` module and `load_garden_timed` serve the CLI, the publisher and any future language server alike; the publisher only records its own phases, and the CLI only prints them.

### 2 - Should timings be a flag or an environment variable?

Decided by the agent without codekiln: a `--timings` flag on `lsg publish`. It appears in help, and scripts already pass flags to lsg. The help assessment was refreshed for the changed help text.

### 3 - Where should the human report go?

Decided by the agent without codekiln: one final stderr line. Stdout keeps its single result line, and the JSON report gains `timings` and `total_ms` only when the flag is given.

### 4 - What counts as a cold build?

Decided by the agent without codekiln: the first build after the garden is written or checked out, without clearing the operating system's file cache, because clearing it needs administrator rights on both macOS and Linux runners.

### 5 - What stands in for Gitpa's garden?

Decided by the agent without codekiln: a garden generated from a fixed seed with Gitpa's size and shape. Gitpa's content belongs to another repository, and committing 3 MB of generated text adds nothing a seed and a digest do not.

### 6 - How wide should the regression limit be?

Decided by the agent without codekiln: 20% over the slowest row on the same processor model, and 100% over the slowest row for the corpus otherwise. The October 2026 `build-plugins` logs show about 6% spread within a group of runners and almost a factor of two between groups.

### 7 - Should the check run in `mise run ci` or in its own job?

Decided by the agent without codekiln: its own job. It needs the encode garden checkout that `build-plugins` already uses, and a local `mise run ci` should not spend minutes on a whole-garden benchmark.

### 8 - Should this change also fix the bottleneck it finds?

Decided by the agent without codekiln: no. The issue asks to investigate. Rendering Linked References, which renders each referencing block again for every page it references, is the largest share of render time, and changing it touches link and reference correctness. [Issue #290](https://github.com/codekiln/logseq-gardener/issues/290) records the measurements.

## Open Questions

None.
