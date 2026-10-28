## Context

`.outline li` in `outline.css` sets no wrapping rule, so a page reference such as `Ableton/Live/Mode/Arrangement` keeps its full width. In nested blocks at 390px, the two references on `GitP/A/Session/26/10/24-Sat` end at 406px and 437px, and the mobile layout viewport grows to 449px. Logseq sets `overflow-wrap: break-word` and `word-break: break-word` on `.block-content`.

## Goals / Non-Goals

**Goals:**

- Every outline block fits the screen width at phone sizes, as on Logseq's site.

**Non-Goals:**

- Code blocks: they keep `white-space: pre` and scroll inside their box, as in Logseq.

## Decisions

- `.outline li` gets `overflow-wrap: anywhere`. It breaks a long word only when the word does not fit, as Logseq's `word-break: break-word` does, and it is the rule `lsg` already uses for linked references, hierarchy rows, breadcrumbs and search results. Embeds inside a block inherit it.

## Risks / Trade-offs

- A long word breaks at any character, not at `/`. → Logseq breaks the same way; a break at `/` would need markup in every page name.

## Resolved Questions

### 1 - Which wrapping declaration should `lsg` use?

Decided by the agent without codekiln: `overflow-wrap: anywhere`, not Logseq's pair of `overflow-wrap: break-word` and the deprecated `word-break: break-word`. Both break a word only when it does not fit and both count the breaks when sizing content, so the result looks the same, and `anywhere` matches the rule used elsewhere in `lsg`.
