## Why

Gitpa publishes with Logseq Gardener partly because it is fast, but nothing measures that speed or stops a change from losing it ([Issue #259 · Measure and protect garden publication speed](https://github.com/codekiln/logseq-gardener/issues/259)). One hosted Gitpa run showed most of the time going to Gitpa's own homepage query before Gardener started ([gitpa issue #102 · Reduce repeated work in site publication](https://github.com/codekiln/gitpa/issues/102)), so Gardener's share needs its own measurement before anyone decides where to spend effort.

## What Changes

- The SDK records the wall-clock time of each publication phase: reading files, parsing, building indexes, waiting for build plugins, rendering, waiting for git history, writing search files, waiting for the graph and writing the site. `SiteReport` carries the times, and a timed garden loader serves any other SDK caller.
- `lsg publish --timings` reports those times on stderr, or in the JSON report with `--format json`. Without the flag, output is unchanged; with it, the published files are unchanged.
- `mise run parity:publish-bench` publishes a generated garden the size and shape of Gitpa's and, when available, the encode garden. It measures a cold build and several warm ones and records a row with the lsg commit, the corpus, the machine, the times, the phase times, peak memory and the output's size.
- A new CI job runs the benchmark on both corpora and fails when the fastest warm build exceeds a limit derived from rows recorded on GitHub's runners.
- `docs/parity/publication-speed.md` reports the measured results and the bottleneck they show.

## Capabilities

### New Capabilities

- `publication-speed`: phase timings in the SDK, the publication benchmark and its recorded rows, and the CI regression check.

### Modified Capabilities

- `cli-static-publishing`: add the optional `--timings` report to `lsg publish`.

## Impact

The SDK gains the `timing` module, `garden::load_garden_timed` and `SiteReport::timings`; code that builds a `SiteReport` by hand needs the new field. The CLI gains `--timings`, with help text and a refreshed help assessment. New files: `scripts/parity/publish_bench.py`, `scripts/parity/sample_garden.py`, the `parity:publish-bench` mise task, `docs/parity/publication-speed.json` and `docs/parity/publication-speed.md`. `scripts/parity/budgets.toml` gains a `[publication]` table, and `.github/workflows/ci.yml` gains the `publication-speed` job. No dependency changes.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/AI/Rule/How to Communicate Effectively With Me/Words and Phrases to Avoid](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___Words%20and%20Phrases%20to%20Avoid.md)
