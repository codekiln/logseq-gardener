## Context

The pinned lsdoc public-garden runner parses Org syntax but calls `extractRefs(ast)` in its mldoc worker. The extractor defaults to Markdown, which omits Org search-link page references. The existing direct regression passes the format correctly and therefore misses the worker defect.

## Goals / Non-Goals

**Goals:** Correct the public-garden reference comparison, test the worker used by the runner, and regenerate the docs evidence.

**Non-Goals:** Current Logseq OG relationship coverage, Pengx's publishing corpus, and parser adoption continue under [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3) and [Issue #4 — Read-only page lookup](https://github.com/codekiln/logseq-gardener/issues/4).

## Decisions

### Repair the archived runner during experiment setup

Keep the tested lsdoc revision fixed and replace the missing format argument in the extracted source. Require the expected call to occur once before replacement so a source upgrade requires deliberate review. Record the repair in each generated report. Updating upstream is a useful later option, but would also change the parser version being compared.

### Test the public-garden runner with an Org fixture

Run the actual comparison program on a small synthetic garden containing an Org search link and a Markdown page link. Check the runner's comparison summary for equal reference results and successful parsing. A direct call to the extractor cannot test whether the worker supplies its format argument.

### Keep deterministic checks separate from measured reports

The small runner check should assert comparison outcomes while allowing timing and temporary paths to vary. Regenerate the docs report using the existing pinned corpus; reviewers can inspect remaining disagreements and the corpus identity.

## Risks / Trade-offs

- Upstream source changes could invalidate the repair → fail setup when the expected call is missing or repeated.
- Corpus timings vary between runs → assess reference outcomes and corpus identity independently of timings.

## Resolved Questions

### 1 - Where is the repair applied?

Only the source extracted under the experiment's ignored `node_modules/sources/lsdoc` directory is changed. The registered lsdoc checkout retains its existing files.

## Open Questions

None for this repair.
