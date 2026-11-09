# Proposal

## Why

The page-load benchmark recorded in #590 left four `lsg` values over their budgets and two numbers worse than earlier rows, and issue #591 asks for each to be brought back within budget, or its budget changed with a reason, and for the regressions to be explained. Two of the four come from the browser doing work during first paint or rendering that it could skip, and one comes from the harness counting a keystroke as warm while it waits for a shard.

## What Changes

- `lsg-shell.js` starts the scripts it loads after the `load` event (the outline, reference sections, journals feed, right sidebar restore) after the next paint, so no first paint waits on them. On `Contents` they had started before the title showed on desktop, which put the page at 26.6 KB of script before the title.
- All pages loads `lsg-all-pages.js` the same way instead of from a `<script defer>` tag, which brings its script before the title from 17.0 KB to 11.7 KB. A plugin host that swaps in All pages loads the script itself.
- Reference groups that are off screen skip layout (`content-visibility: auto`), which brings a filter change on `mise` from 58.6 ms to 13.5 ms on desktop. The references styles budget goes from 1 KB to 1.05 KB gzip to fit the 32 added bytes.
- `parity:perf` counts a keystroke typed while an earlier key's shard is still loading as cold, since it waits for that shard. The phone's 95th-percentile warm keystroke was 158 ms because such keystrokes counted as warm; with them counted as cold it is 74 ms.
- `parity:budgets` counts every script a page's markup loads on All pages too, plus the page-property start-up that the encode garden adds to every page, so the All pages script regression would have failed CI.
- The phone title regression (about 240 ms to 410 ms) and the build-time increase (10.5 s to 19.0 s) are explained and accepted; see design.md.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-all-pages`: the All pages script loads after the first paint.
- `published-app-shell`: the scripts the shell loads after the `load` event wait for the next paint.
- `published-reference-sections`: the references styles budget becomes 1.05 KB gzip.
- `parity-comparison-harness`: cold keystrokes include keys typed while a shard loads, and the CI script check covers every script a page's markup loads, All pages included.

## Impact

- Publisher assets in `crates/logseq-gardener-sdk/src/site/`: `shell.js`, `host.mjs`, `all_pages.rs`, `references.css`.
- Harness: `scripts/parity/perf.py`, `metrics.py`, `budgets.py`, `budgets.toml` and their tests.
- No change to the SDK's graph answers, the search index or the page markup other than All pages' `<head>`.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
