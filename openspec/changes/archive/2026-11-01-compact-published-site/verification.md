# Verification: compact-published-site

[PR #400 ⚡️ perf: share page styles and compress reference fragments](https://github.com/codekiln/logseq-gardener/pull/400) implemented this change. It merged into `main` as `8b3e3e6` on 2026-11-01 and shipped in v0.0.15. The encode garden's site went from 1,216.8 MB to 470.2 MB with no post-processing, within the 550 MB `[site]` budget.

Checked on 2026-11-01 from `main` at `4390c88`.

## Tasks

27 of 28 tasks are done. One is partly done.

- **6.3 is partly done.** The Performance page records both sites' sizes and file counts in its Site size section (`parity:perf --site-size --record`): `lsg` 476.9 MB in 47,766 files and Logseq's build 227.0 MB in 506 files. `lsg`'s page loads, reference sections and search were measured and are within budget, as PR #400 reports. A full `parity:perf --record` against Logseq was not run: Logseq's phone loads take about 200 s per page, and the garden's `gh-pages` branch now holds `lsg`'s site, so Logseq's build came from a cached copy of `gh-pages` commit `8132c95`. The page-load rows on the Performance page are still those of earlier commits, so the requirement Measure the encode garden against Logseq's build is only partly evidenced.
- **7.1** was a follow-up after a release. v0.0.15 contains the change, and [logseq-encode-garden#303 Remove the repeated-styles step from the site build](https://github.com/codekiln/logseq-encode-garden/issues/303) tracks removing `share_repeated_styles`.

## Known gaps on the whole encode garden

PR #400 reports these. No CI check runs them on the whole garden.

- The filter change on `mise` took 57.7 ms on desktop, over its 50 ms budget. The change does not touch the filter code; `mise` now has 1,199 groups.
- The largest Linked References fragment is 223 KB as stored, over the 100 KB budget. It was 226 KB gzip before the change.
- Some readable Hierarchy fragments reach 412 KB as stored. They are within their own gzip budget.

## Implementation on main

- `lsg-site.css`, `lsg-theme.js` and `lsg-search-boot.js` are named in `crates/logseq-gardener-sdk/src/site/mod.rs`. The journals feed and All pages styles also became shared files, `lsg-journals.css` and `lsg-all-pages.css`.
- `references.rs` writes `<page>-linked-<n>.html.gz` and `-unlinked-<n>.html.gz`. `references.js` decompresses files that start with the gzip header and shows `References could not be loaded.` when a file fails.
- `scripts/parity/budgets.toml` has the `[site]` table (550 / 200 / 100 MB, 1.5 KB inline scripts, 10% build slowdown), and `[references] fragment_kb = 100`.
- The `ci`, `publication-speed` and `build-plugins` checks passed on PR #400.
