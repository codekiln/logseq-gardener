# Compare lsg with Logseq's published site

These tasks build the whole encode garden with `lsg` and compare the result with Logseq's own build of it, the `gh-pages` branch of `codekiln/logseq-encode-garden`. Each parity issue under #106 uses them to find differences and to show its progress. The [comparison harness analysis](comparison-harness.md) explains what the two builds contain.

Run them in order from the repository root:

```sh
mise run parity:build
mise run parity:compare
mise run parity:screenshots
mise run parity:perf
```

Every run writes into a new directory, `target/parity/<UTC time>/`. `parity:compare`, `parity:screenshots` and `parity:perf` use the latest run unless you pass `--run <directory>`.

## What you need

- The encode garden checked out with `ghq`. `parity:build --garden <path>` publishes another garden instead.
- Network access to fetch the garden's `gh-pages` branch. The tasks read it with `git archive`, so your garden checkout stays on its branch.
- For screenshots and measurements, Playwright's Chromium. The tasks install it through `uv` on first use, into Playwright's shared browser cache.

## The tasks

| Task | Writes | What it does |
| --- | --- | --- |
| `parity:build` | `site/`, `settings.json`, `diagnostics.txt` | Builds a release `lsg` and publishes every page whose top-level namespace `lsg` can select, with `--journals`. `settings.json` records the garden commit, the `lsg` version and commit, every option passed, and files whose namespace `lsg` cannot select. |
| `parity:compare` | `logseq/`, `report.md`, `report.json` | Extracts Logseq's build, reads its page list from `static/js/db.gz`, and lists pages present in one build and missing from the other. Logseq-only pages are grouped as journals, reference-only pages, pages hidden by `:hidden`, titles `lsg` cannot select, and other pages. Each other page needs an explanation or a fix. |
| `parity:screenshots` | `screenshots/index.html` | Captures every page in [`scripts/parity/fixtures.txt`](../../scripts/parity/fixtures.txt) from both builds at 1280 by 800 and at 390 by 844 (`--viewport desktop` or `phone` takes one) in light mode, after scrolling so Logseq draws its Linked References. The index shows the two captures side by side. |
| `parity:perf` | `perf.md`, `perf.json` | Loads each fixture page from both builds with an empty cache, on a desktop profile and on Lighthouse's mobile profile (four times CPU slowdown, 150 ms round trip, 1.6 Mbps). It records time until the page title is visible, first contentful paint, data and script transferred before the title, and peak JavaScript heap, then types the queries in [`scripts/parity/search-queries.txt`](../../scripts/parity/search-queries.txt) into each build's search. It scrolls each build's journals feed until it shows ten later batches and records the peak heap, and times `lsg publish` on the whole garden with and without `--journals`. Bold values exceed [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml). |

The local server compresses text files with gzip, as GitHub Pages does, so transfer sizes match the published sites.

A full `parity:perf` run takes hours, because Logseq's build needs minutes per load on the phone profile. Use `--profiles desktop`, `--loads 1` or `--fixtures <file>` for a quicker check. `--record` rewrites [Parity performance](performance.md), the committed results that parity pull requests compare against.

## The CI check

`mise run parity:budgets` runs in `mise run ci`. It publishes the fixture garden in `tests/fixtures/parity-garden/`, then publishes it again with 2,000 added pages that nothing links to. It fails when a page's first-paint size or script size exceeds the budget, or when a page grows because the garden grew. It also checks the `[shell]` budgets: `lsg-shell.js` and each page's header and sidebar markup must stay within their sizes and keep them when the garden grows. It publishes the fixture garden once more with 1,000 older journals and checks the `[journals]` budget: `lsg-journals.js` stays within its size, and it and the home page keep their sizes.

## Change the fixtures or budgets

Add a page title to `scripts/parity/fixtures.txt` when a parity issue needs it compared; `Home` and `All pages` name Logseq's journals home and All pages routes. Change `scripts/parity/budgets.toml` in the proposal of the feature that needs a different budget, with the measurements that justify it.
