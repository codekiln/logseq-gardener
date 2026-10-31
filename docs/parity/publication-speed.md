# Publication speed

A warm build of the whole encode garden takes 21 to 30 seconds on GitHub's 4-processor runners, depending on the processor, and 18 seconds on an Apple M2 Pro laptop. A generated garden the size of Gitpa's takes 4 to 5 seconds on the runners and 3.5 seconds on the laptop. Rendering pages takes most of the time, and about half of rendering goes to the Linked References sections.

## Measured results

Each measurement is the fastest of five warm builds, after one cold build, with `--journals`, `--search` and every top-level namespace included. Times are in seconds. [publication-speed.json](publication-speed.json) holds every recorded row with its lsg commit, corpus identity and machine; the GitHub rows come from five runs of the `publication-speed` job on this change.

| Machine | Runs | Gitpa-sized warm | Encode garden warm | Encode garden render |
| --- | --- | --- | --- | --- |
| GitHub runner, AMD EPYC 7763 | 3 | 5.1 to 5.3 | 28.1 to 29.8 | 19.1 to 20.4 |
| GitHub runner, Intel Xeon Platinum 8370C | 1 | 5.1 | 28.2 | 19.8 |
| GitHub runner, Intel Xeon 6975P-C | 1 | 3.8 | 21.1 | 14.6 |
| Apple M2 Pro, 12 processors | 1 | 3.5 | 17.7 | 8.7 |

The generated Gitpa-sized garden has 2,410 Markdown files and 3 MB of text, and its site has 9,900 files and 236 MB. The encode garden, at commit `5095bdf` on GitHub and `4d37b9f` on the laptop, has 9,246 files and 16 MB of text, and its site has 46,767 files and 1,212 MB. Peak memory for the encode garden was 2.5 GB on the runners and 3.5 GB on the laptop.

The cold build is the first build after the garden is written or checked out. It was 1% to 13% slower than the fastest warm build. The benchmark does not clear the operating system's file cache, which needs administrator rights, so the difference shows the cost of a fresh process rather than of reading from disk.

Phase times for one encode garden build, in seconds:

| Phase | GitHub runner, AMD EPYC 7763 | Apple M2 Pro | What it covers |
| --- | --- | --- | --- |
| `read` | 0.1 | 1.3 | Finding and reading source files, checking options |
| `parse` | 0.5 | 0.3 | Parsing Markdown |
| `index` | 3.4 | 1.9 | Selection, page names, references, queries and search records |
| `render` | 20.4 | 8.7 | Pages, lists, feeds and fragments |
| `search` | 4.2 | 1.7 | Search files |
| `write` | 0.8 | 3.6 | Writing the site |

The `plugins`, `history` and `graph` phases measure only how long the main thread waited for work running on other threads: build plugins, the git history read and the graph layouts. All three were under 0.1 seconds, so that work finishes while pages render.

## The bottleneck

Rendering runs on one thread and takes 68% to 74% of a warm build on the GitHub runners and 49% on the laptop. A 7-second profile of the laptop build during rendering put 45% of the main thread's samples in `References::render`, which renders Linked References. It copies each referencing block, with its children, and renders it again for every page the block references. [Issue #290 · Render each Linked References context once](https://github.com/codekiln/logseq-gardener/issues/290) tracks reducing that work without changing the output.

Writing the site takes 3.6 seconds on macOS and 0.8 seconds on the Linux runner for the same 46,767 files, and reading the garden 1.3 and 0.1 seconds, so file access costs far more on the laptop than on the runner.

Gitpa's [hosted run 41195021354](https://github.com/codekiln/gitpa/actions/runs/41195021354/job/124519359328) logged Gardener's result about two seconds after its own query step. That run is not directly comparable with these rows: Gitpa publishes without `--journals` and with its own homepage and stylesheet. Gitpa's homepage query runs in Gitpa's tooling before Gardener starts; [gitpa issue #102 · Reduce repeated work in site publication](https://github.com/codekiln/gitpa/issues/102) tracks that work. Passing `--timings` in Gitpa's build would show Gardener's phases in that workflow.

## See where a build spends its time

Add `--timings` to any `lsg publish` command:

```text
lsg publish --graph <garden> --output <site> --filename-format triple-lowbar --include <namespace> --timings
```

The last line of stderr lists each phase in milliseconds, after any diagnostics. With `--format json`, the report gains `timings`, a list of `phase` and `ms`, and `total_ms`. Without the flag, the output is unchanged; with it, the published files are the same. Rust callers read the same phases from `SiteReport::timings`, and `garden::load_garden_timed` times the `read` and `parse` phases of any garden load.

## Run the benchmark

```sh
mise run parity:publish-bench --corpus all
```

`--corpus` chooses `gitpa-sized` (the default), `encode-garden` or `all`. The encode garden comes from `ghq` unless `--garden <path>` names another checkout. `--runs` sets the number of warm builds. `--output <file>` writes the rows as JSON, and `--record` appends them to `publication-speed.json`.

The `gitpa-sized` corpus is generated by [`scripts/parity/sample_garden.py`](../../scripts/parity/sample_garden.py) from a fixed seed. It has Gitpa's page count, word count, namespace depth and link, block and property density as of 2026-10-31. Pages must say `public:: true` to be published, as in Gitpa, and 25 are withheld. Every machine generates the same files; each row records their digest.

## The CI check

The `publication-speed` job runs `parity:publish-bench --corpus all --check` on the generated garden and on the encode garden commit pinned in `.github/workflows/ci.yml`. It compares each fastest warm build with the GitHub rows for the same corpus and fails when the build exceeds its limit:

- With rows on the same processor model, the limit is the slowest of them plus 20%.
- Without such a row, the limit is the slowest GitHub row for the corpus plus 100%.

GitHub assigns runners with different processors. In October 2026 the `build-plugins` job's encode garden build took between 15.5 and 28.8 seconds for nearly the same code; within a group of similar times, runs differed by about 6%. This change's runs show the processor model explains the groups: three runs on the AMD EPYC 7763 differed by 6%, and the Intel Xeon 6975P-C took about 25% less time than either other model. The 20% limit is more than three times the spread on one model, and the 100% limit covers the gap between the fastest and slowest groups. The tolerances live in the `[publication]` table of [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

The job uploads its rows as the `publication-speed` artifact. Record new GitHub rows by adding them to `publication-speed.json` when:

- the job reports a processor model with no row;
- the pinned encode garden commit changes, which leaves the encode garden unchecked until new rows exist; or
- `sample_garden.py` changes, which changes the digest and leaves the generated garden unchecked.

A change that makes publication intentionally slower records new rows in the same pull request and explains the cost.
