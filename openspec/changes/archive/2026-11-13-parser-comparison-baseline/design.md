## Context

The [parser comparison plan](https://github.com/codekiln/logseq-gardener/blob/85eaa574062cbe8b8d816b30ec1718e3bb350986/openspec/changes/project-foundation/parser-comparison.md) proposes testing `lsdoc`, `mldoc`, Logseq's graph parser, and graph-validator, but has no runnable cases or recorded reference results. The local `logseq/logseq` checkout at `63b76c5` declares `mldoc` `^1.5.9` in `deps/graph-parser/package.json`. Its graph-parser tests provide examples for a small, shareable synthetic corpus. The separate candidate repositories are not present in `ghq`.

## Goals / Non-Goals

**Goals:** Check in a compact fixture garden, pin the Logseq-used `mldoc` package for this experiment, and record reproducible raw AST and reference output. A future parser adapter can consume exactly the same Markdown files.

**Non-Goals:** Select the SDK parser, compare Logseq graph relationships, check text-preserving writes, and implement read-only garden commands. We will compare parsers and text-preserving writes under [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3), then add the first read-only garden command through the SDK under [Issue #4 Add the first read-only garden command through the SDK](https://github.com/codekiln/logseq-gardener/issues/4).

## Decisions

- Store the experiment under this change's `experiments/mldoc-baseline/` directory, as required by the existing comparison plan. Use synthetic cases shaped by local Logseq tests, rather than copying whole user notes or upstream fixtures.
- Pin `mldoc` 1.5.9 in an isolated npm package and commit its lockfile. Logseq's local graph-parser accepts this version; keeping it out of the Rust workspace avoids a production dependency.
- Save `parseJson` and `getReferences` output with each source file's SHA-256. The runner reports when a fixture or parser result differs from the saved result; `mise run parser:baseline-update` refreshes it. Raw output preserves details that a later comparison might need; the fixture set stays small enough for review.
- Run `mise run parser:baseline-check` from `mise run ci`, which GitHub Actions invokes. The task installs the locked npm dependencies with the Node version pinned in `mise.toml`.
- Record source revisions and the limits of this baseline in the experiment README. Logseq graph relationships and text-preserving writes need separate evidence before parser adoption.

## Risks / Trade-offs

- `mldoc` output alone can look compatible while Logseq's graph parser interprets relationships differently → label it a syntax baseline and track graph compatibility under [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3).
- npm and Node behavior can change → pin the package and lockfile, use the Node version in `mise.toml`, and compare committed output.
- Raw AST snapshots can be noisy → keep fixtures short and require a deliberate update command when the parser version or inputs change.

## Resolved Questions

### 1 - Which `mldoc` version should produce the reference output?

Pin 1.5.9. The locally checked-out Logseq graph-parser accepts `^1.5.9`, and the exact npm archive is recorded in the lockfile.

### 2 - Where should the comparison files live?

Keep the fixtures, runner, and results together under this change's named `experiments/mldoc-baseline/` directory so they remain with the OpenSpec record.

## Open Questions

None for this reference result. [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3) tracks the parser choice and the remaining comparisons.
