## Context

A page shows blocks of other pages in three places: embeds, Linked References and query results. The renderer marks a diagnostic from such a block by putting the block's file in front of the message, as `<file>: <message>`. The publish loop then reports every message from a page's render, and from its reference sections, under the page's own file. A block's problem is therefore reported once by its own page and again by each page that shows it, with two file names in front.

Every block a page can show comes from a selected page, and every selected page renders as a page of its own. The block's own page reports the problem whenever the copy renders the same way. A copy can also produce a diagnostic its page does not: a `{{query}}` inside a copied block, for example, renders as literal text in the copy only.

## Goals / Non-Goals

**Goals:**

- Each problem appears once in the report, under the file that holds it.

**Non-Goals:**

- Report flashcard diagnostics under the card's file. The flashcards page reports them under `flashcards`, and this change leaves them there.

## Decisions

- A new `page_diagnostics` module records the publish loop's diagnostics. A message that starts with the file of a selected page followed by `: ` is moved to that file. When the build ends, it keeps each moved diagnostic once, and drops it when that file's own page reports the same message. The page's own diagnostics are kept as they are, so two identical problems on one page still give two diagnostics. Alternatives considered:
  - Silence copies, as Unlinked References already do. That would lose the diagnostics only a copy produces.
  - Carry the file as a field through the renderer, the reference sections and the query boxes instead of in the message. That is cleaner, but it changes the diagnostic type in every renderer and query function for the same result.
- The module recognises a file only from the set of selected page files, so a message that happens to contain `: ` stays where it is, and a file name that contains `: ` is still found.
- The kept diagnostics take the place of the first time they were reported, so the report keeps its order by page.

## Risks / Trade-offs

- [A message from the page's own text starts with the name of another selected file and `: `] → It would move to that file. The renderer writes no such message; messages start with a description of the problem.
- [Moving the file into the message is a format the renderer and this module share] → `page_diagnostics.rs` documents the format, and its unit test covers a file name that contains `: `.

## Resolved Questions

### 1 - Is the repeat caused by `--homepage`?

Decided by the agent without codekiln: no. The docs graph showed it with `--homepage lsg`, but the same build without `--homepage` reports the same repeats. The repeats come from `lsg` listing the new page's block in its Linked References, because the block links to a page in the `lsg` namespace.

### 2 - Should embeds change as well as Linked References?

Decided by the agent without codekiln: yes. Embeds put the embedded file in front of the message in the same way, and the issue asks for each diagnostic once under the file that holds the problem.

## Open Questions

None.
