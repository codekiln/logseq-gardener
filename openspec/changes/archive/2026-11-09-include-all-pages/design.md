# Design

## Context

`NamespaceSelection` in the SDK holds included and excluded roots. `select_with_urls` asks it about each page's title after the title is derived, so aliases and `title::` are already applied. Journals skip the namespace check and are published only when the caller passes `journals: true`. Visibility (`public::` and nesting) is checked after selection, so it applies to every selected page. See proposal.md for why a whole-garden flag is needed.

## Goals / Non-Goals

**Goals:**

- One flag selects every page, with the same exclusion and visibility rules as any other selection.

**Non-Goals:**

None.

## Decisions

### Selection lives in `NamespaceSelection`

`NamespaceSelection::new(included, excluded)?.including_all()` returns a policy that selects every valid page name no excluded root matches. `includes_all()` reports it. Every SDK operation that already takes a `NamespaceSelection` gets the behaviour without a new parameter. A separate boolean on `select` and `SiteOptions` was considered; it would need threading through every caller and would let the two disagree.

### The CLI passes the choice through

`lsg publish` builds the selection from `--include` and `--exclude` as before, and calls `including_all()` when `--include-all` is set. `--journals` keeps deciding whether journals are published. The CLI makes no selection decision of its own.

### `--include` beside `--include-all` is allowed

The include roots are still validated, so a typo fails the same way with or without the flag, and then they change nothing. Rejecting the pair was considered; `scripts/parity/perf.py` adds `--include Parity` to a run's recorded options, and that would break once the run records `--include-all`.

### Malformed candidate names stay unselected

Under `--include-all`, a page whose title has an empty or whitespace-only segment, or a control character, is still unselected, as the existing matching rule says. Selecting them would change the rule for one mode and let such titles reach routes and links that were never tested with them.

### `parity:build` uses the flag

`scripts/parity/build.py` passes `--include-all --journals --search` with the filename format, and no longer derives roots or records `skipped_files`. Nothing reads `skipped_files`. `build.roots`, `build.page_title` and `build.valid_root` stay, because `build_bench.py`, `publish_bench.py` and `compare.py` use them; `outside_files` is removed with its test because only `build.py`'s old root derivation used it.

## Risks / Trade-offs

- [A garden with many private pages publishes more than a caller expects] → visibility rules still withhold every page that is not public, and the report counts withheld pages.
- [Bench scripts still derive roots, so their page set can differ from `parity:build`'s] → their measurements compare builds with each other, not with `parity:build`; switching them is a separate change.

## Resolved Questions

### 1 - Does `--include-all` publish journals, or does it still need `--journals`?

Decided by the agent without codekiln: it still needs `--journals`. Journals are not subject to namespace selection: `--include` and `--exclude` never applied to them, and `--journals` alone already publishes every journal, so `--include-all --journals` publishes every page and journal. Keeping the two flags separate also keeps `parity:perf` working, which times a build without journals by removing `--journals` from a run's recorded options. The issue's measure of done, a whole-garden build without title derivation, holds either way, since the encode garden's `site:build` already passes `--journals`.

### 2 - What happens when `--include` is given with `--include-all`?

Decided by the agent without codekiln: the include roots are validated and then change nothing. Validation keeps error behaviour the same with or without the flag, and accepting the pair keeps `perf.py`, which appends `--include Parity` to a run's options, working.

### 3 - Does `--exclude` still apply?

Decided by the agent without codekiln: yes, and exclusion wins as it does today. The issue says so, and it lets a garden publish everything except a namespace such as `My/Private`.

### 4 - Are withheld pages published under `--include-all`?

Decided by the agent without codekiln: no. Visibility is checked after selection, so pages marked `public:: false`, pages withheld by nesting and pages not marked `public:: true` in a garden without the all-pages-public setting stay withheld and counted. Selecting a page has never made it public.

### 5 - Where does the selection logic live?

Decided by the agent without codekiln: in the SDK's `NamespaceSelection`, as `including_all()`. Other SDK callers then get the same rule, and the CLI only maps its flag onto the SDK call.

### 6 - Does `parity:build` switch to the flag in this change?

Decided by the agent without codekiln: yes. It removes the file-name derivation that misses alias-titled pages such as `SolveSpace`, which is the bug the issue's comment describes. The encode garden's `site:build` lives in another repository and is left for a follow-up there.

## Open Questions

None.
