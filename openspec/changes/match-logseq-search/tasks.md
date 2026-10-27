## 1. Measure the starting point

- [ ] 1.1 Make `mise run parity:build` pass `--search`, run `parity:perf --search-site` on the whole encode garden, and verify the committed search rows in `docs/parity/performance.md` hold numbers for both builds
- [ ] 1.2 Index the same `lsg` build with Pagefind in `experiments/pagefind/` (with a README saying how to run it) and record its index size, bytes per query and keystroke latency in `verification.md`

## 2. SDK search index

- [x] 2.1 Move record building from `site/search.rs` into a new SDK `search` module that keeps parents to the nearest three, and verify the existing privacy tests pass against it unchanged
- [x] 2.2 Add the word index with the word-splitting rule and verify a unit test for `tmux.conf: set -g mouse on`
- [x] 2.3 Add the Rust query with fuzzy title matching, Logseq's fuzzy score, block word-prefix matching, groups and full counts, and verify unit tests for `tmux`, `tmx`, `logseq query` and `tmux.conf.local`
- [x] 2.4 Add `tests/search-fixture.json` with queries and expected results and verify `cargo test` runs it against the Rust query
- [x] 2.5 Add sharding and record chunks named by content digest and verify a test that editing one block changes only the manifest, the affected shards and one chunk

## 3. Published files and engine

- [x] 3.1 Write the format version 2 manifest, title index, shards and chunks from `site/search.rs`, and verify the validator tests reject version 1, revision mismatch and digest mismatch
- [x] 3.2 Rewrite `search.mjs` for shard and chunk evaluation with the same matching and ranking, and verify the Node tests run `tests/search-fixture.json` with the same results as Rust
- [x] 3.3 Update `docs/static-search.md` for format version 2 and verify its examples match a fixture build

## 4. Dialog

- [x] 4.1 Change the loader in `search-ui.mjs` to start on button hover, focus or Cmd/Ctrl+K and to fetch shards and chunks per keystroke, and verify a browser test that `tm` then `tmux` fetches the `tm` shard once and no other shard
- [x] 4.2 Add the groups, count badges, five-row limit, Show more and Show less, and verify browser tests for `tmux` on the home page and for Current page on a page whose blocks match
- [x] 4.3 Add highlighting, block breadcrumbs and `alias of`, and verify browser tests for `tmux` and `tmux.conf.local`
- [x] 4.4 Add the `/` filters, the chip and its removal, and verify a browser test that `/`, `Search only pages`, `tmux` lists every matching page and nothing else
- [x] 4.5 Add the published commands and the keyboard handling, and verify browser tests for an empty input, wrapping, Enter on a command and the three-step Escape
- [x] 4.6 Restyle `search-ui.css` with Logseq's values and verify computed styles at 1280×800 against the analysis, with screenshots beside the `gh-pages` build in `verification.md`
- [x] 4.7 Build the encode garden, run Gitpa's navigation tests against it, and record the result in `verification.md`

## 5. Budgets

- [ ] 5.1 Replace the `[search]` corpus keys in `scripts/parity/budgets.toml` with the design's budgets and verify `parity:budgets` fails on a fixture with an oversized shard
- [ ] 5.2 Run `parity:perf --search-site` on the whole encode garden, commit the rows to `docs/parity/performance.md` and `docs/parity/search-performance.md`, and verify every `lsg` value is within its budget
- [ ] 5.3 Run `mise run ci` and verify it passes

## Workflow follow-up

- Create the cleanup sub-issue that `docs/openspec-cleanup.md` requires before the Apply PR is handed over.
- After the Apply PR merges, sync and archive this change in a separate PR.
