## Context

The saved graph results use Logseq 0.9.8 with mldoc 1.5.9. The current `version/file` branch contains Logseq OG's Markdown graph parser and declares mldoc 1.5.7. Its graph-loading API still accepts explicit files and graph configuration.

## Goals / Non-Goals

**Goals:** Reproduce current Logseq OG page and block relationships on the existing fixtures, compare with historical behavior, and explain changes relevant to page lookup.

**Non-Goals:** Pengx's publishing corpus and parser adoption continue under [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3) and [Issue #4 — Page lookup](https://github.com/codekiln/logseq-gardener/issues/4).

## Decisions

### Run both parser revisions with mldoc 1.5.7

Use the exact mldoc version declared by current OG for both parser runs. That version also satisfies the historical parser's declared range. Compare the historical result with the saved mldoc 1.5.9 snapshot separately, so runtime changes remain visible. Separate npm environments for different mldoc versions would make parser-source changes harder to isolate.

### Compare graph relationships directly

Run a shared page/block projection through each parser in fresh processes. The historical validator remains part of the existing saved evidence; this experiment compares graph relationships directly. Reusing the validator against a different parser would mix its historical assumptions into the current-OG comparison.

### Compare page identities, references, and hierarchy

Reuse page names, relative file paths, aliases, properties, namespaces, block parents, previous siblings, and references from the earlier experiment. Normalize generated UUIDs and retain explicit UUID properties. Run default and triple-lowbar filename configurations with the same sorted inputs. This makes the new evidence directly comparable with the saved historical graph.

## Risks / Trade-offs

- Projection omits some internal database fields → document the selected fields and qualify conclusions as fixture relationship coverage.
- A newer OG revision may change behavior → save the tested revision and require an intentional snapshot update for later comparisons.
- Runtime differences may explain changed historical results → report comparison with the earlier snapshot independently of source-revision differences.

## Resolved Questions

### 1 - Which source branch represents current Logseq OG?

Use Logseq's `version/file` branch, whose [tested revision](https://github.com/logseq/logseq/tree/6efedb75588763af256bc7dfd0ed5526dc91fe7c) retains the file-garden parser. The repository's default branch serves the newer database graph implementation.

## Open Questions

None for running the comparison.
