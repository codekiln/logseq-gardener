# Verification

## Timings leave the output unchanged

`tests/publish.rs` publishes one garden with and without `--timings`. Stdout matches once the output directory name is replaced, the two sites contain the same files with the same contents, stderr gains one final line starting `Timings in milliseconds: read`, and that line contains no page title or page text. The JSON report lists the nine phases in order, `total_ms` equals their sum to within rounding, and a report without the flag has neither field.

`crates/logseq-gardener-sdk/tests/garden.rs` checks that `load_garden_timed` returns the same garden as `load_garden` and records `read` then `parse`. The existing stylesheet test in `tests/site.rs` compared two SDK reports for equality; it now compares them without their timings, which differ between any two builds.

## The generated garden is the same on every machine

The generated garden's digest was `c86afa4f…` on macOS 26 with Python 3.14 and on GitHub's Ubuntu runner, so rows from both compare the same input.

## Benchmark and check

`mise run parity:publish-bench --corpus all --record` recorded the local rows in `docs/parity/publication-speed.json`. The `publication-speed` CI job ran the same benchmark with `--check` on GitHub's runners five times, on an AMD EPYC 7763 three times, an Intel Xeon Platinum 8370C and an Intel Xeon 6975P-C, and uploaded its rows, which were added to the same file. With those rows recorded, each model's limit is 20% over its slowest encode garden build: 35.7 seconds on the EPYC 7763, 33.8 on the 8370C and 25.4 on the 6975P-C. Unit tests in `scripts/parity/test_parity.py` cover the three limit rules: rows on the same processor model, the fallback for another model, and rows for another corpus or runner.

## Profile

During a local encode garden build, `/usr/bin/sample` recorded the main thread for 7 seconds of the render phase. Of 4,103 samples under `publish_site_with_options`, 1,861 were in `site::references::References::render`. [Issue #290 · Render each Linked References context once](https://github.com/codekiln/logseq-gardener/issues/290) records the finding.

## Checks

- `mise run ci`: format, lint, tests, title tests, release build, parity budgets, OpenSpec validation, parser baseline, SDK fixtures, skill versions, RuleSync and notices passed. `docs:check` stopped at its `mise install` step because a machine-wide mise configuration pins Python 3.12.1, which has no GitHub attestation; the changed documents' local links resolve. `clilint:check` (18 checks at score 4) and `release:check` passed when run individually. GitHub CI runs the full task.
- `openspec validate publication-speed --strict` passes.
