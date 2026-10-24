## Context

The shared publisher already counts source pages and referring contexts separately. Logseq 0.10.6 groups referring blocks by page and shows one reference heading. Gardener currently prints both totals even when equal, which obscures their meaning.

## Goals / Non-Goals

Goals: concise source-page counts, useful context detail, accurate filtered totals and short filter instructions.

Non-goals: backlink card redesign and property rendering, tracked separately in issues 89 and 92.

## Decisions

The heading uses “1 page” or “2 pages.” A second span includes “· 3 referring contexts” only when context and page counts differ. A short explanation inside the disclosure defines a context as a referring block or page properties. Equal totals convey all useful information through the page count.

During filtering, both visible and original totals determine whether context detail is useful. For example, “1 of 2 pages · 1 of 3 referring contexts” preserves the original distinction even when the visible counts become equal. The browser updates the same elements as the initial publisher markup, including hidden state, with no change to matching predicates.

## Risks / Trade-offs

- Server/browser count labels could disagree. Publication and browser fixtures cover zero, equal and unequal counts, filtering and reset.
- Short instructions leave include-all/exclude-any detail implicit. Existing selection behavior and combined-filter regression tests retain those semantics.

## Resolved Questions

### 1 - When should the extra context count appear?

Show it when visible counts differ or, during filtering, when original counts differ. Explain the term inside the disclosure only when the extra count appears.

## Open Questions

None.
