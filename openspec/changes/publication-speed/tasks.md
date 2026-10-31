## 1. Phase timings in the SDK

- [x] 1.1 Add the `timing` module with `Stopwatch`, `Timings` and `PhaseTime`, and a doc test
- [x] 1.2 Add `garden::load_garden_timed`, adding to the `read` and `parse` phases, and test that it loads what `load_garden` loads
- [x] 1.3 Record the nine publication phases in `SiteReport::timings`, collecting graph data before the output directory is created
- [x] 1.4 Compare SDK reports without their timings in the existing stylesheet test

## 2. CLI report

- [x] 2.1 Add `lsg publish --timings`: a final stderr line in human output, `timings` and `total_ms` in JSON
- [x] 2.2 Test that stdout and the published files match a build without the flag, and that the report holds no titles
- [x] 2.3 Describe the flag in help and refresh the help assessment

## 3. Benchmark and regression check

- [x] 3.1 Add `scripts/parity/sample_garden.py`, a seeded Gitpa-sized garden with a digest
- [x] 3.2 Add `scripts/parity/publish_bench.py` and the `parity:publish-bench` task: cold and warm builds, rows, `--record`, `--check` and `--output`
- [x] 3.3 Add the `[publication]` tolerances to `scripts/parity/budgets.toml` and unit tests for the limit rules
- [x] 3.4 Add the `publication-speed` CI job on the generated garden and the pinned encode garden commit
- [x] 3.5 Record rows from a local machine and from GitHub runners

## 4. Results

- [x] 4.1 Write `docs/parity/publication-speed.md` with the measured results and the bottleneck, and link it from the parity guide and the publishing guide
- [x] 4.2 File a follow-up issue for the Linked References rendering bottleneck
- [x] 4.3 Run `mise run ci` and `openspec validate publication-speed --strict`
