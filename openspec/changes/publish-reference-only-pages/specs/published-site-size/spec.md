## MODIFIED Requirements

### Requirement: Keep the encode garden's site within size budgets

The `[site]` table in `scripts/parity/budgets.toml` SHALL set the encode garden's site at most 600 MB, its page HTML at the site root at most 260 MB and its `_refs/` directory at most 115 MB, measured as the total size of the files `lsg publish` writes with `--journals`, `--search` and every top-level namespace included. `parity:publish-bench --check` SHALL fail when the encode garden's output exceeds any of these, naming the part and its size. CI SHALL run that check on the pinned encode garden commit. The publication SHALL need no post-processing to meet these budgets. The same table SHALL set `build_slowdown_pct` to 10, and `parity:perf --baseline-lsg` SHALL mark a whole encode garden build more than 10% slower than the build before this change.

#### Scenario: Within budget
- **WHEN** CI publishes the pinned encode garden commit
- **THEN** the site, its page HTML and `_refs/` are within their budgets and the check passes

#### Scenario: Oversized part
- **WHEN** a change makes the encode garden's `_refs/` 130 MB
- **THEN** `parity:publish-bench --check` fails and reports `_refs/` at 130 MB against its 115 MB budget
