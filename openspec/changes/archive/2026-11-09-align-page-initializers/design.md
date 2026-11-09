## Context

The plugin host in `host.mjs` replaces `<main>` on a same-document navigation and then runs the functions in its `inits` list. The "Page initializers" requirement of `published-plugin-host` names media, slides, flashcards, graph and sidebar wiring among them, and the code registers none of these.

A browser check on 2026-11-09, in a fixture garden published with an after-load plugin, followed a link in place and then tried each feature:

| Feature | After a same-document navigation |
| --- | --- |
| Video Play and YouTube timestamp | Works. `media.js` listens for clicks on the document. |
| View as slides | Works. The shell's More menu item opens a slides card that reads the live outline. |
| Flashcard review | Works. The shell opens the review from a document click; its keys are a document listener. |
| Right sidebar, Shift-click | Works. The sidebar sits outside `<main>` and its listeners are on the document and the sidebar. |
| Page graph card | Stale. The card keeps the previous page's graph, because it reads `location` once when it is filled. |

The graph view, `graph.html`, has no page metadata, so the host loads it as an ordinary page.

## Goals / Non-Goals

**Goals:**

- An open Page graph card draws the page the reader is on after every navigation.
- The "Page initializers" requirement matches the initializers the host runs.

**Non-Goals:**

None.

## Decisions

`sidebar.js` defines `window.logseqGardenerInitSidebar`, and `host.mjs` adds it to `inits` beside the other `logseqGardenerInit*` functions. The function marks each filled Page graph card as empty and fills it again when the sidebar is open. A closed sidebar fills the card when it next opens, through its existing render. Other card types keep their content, since each names a fixed page.

`pageGraph` in `graph.js` ends the previous drawing's layout worker before it starts a new one. The sidebar holds at most one Page graph card, so 20 navigations with the card open leave one worker.

The requirement lists the registered initializers by name and adds scenarios for the document-level features, so a reader of the spec sees which behaviour needs an initializer and which keeps working through document listeners.

## Risks / Trade-offs

[A Page graph card refetches a neighbourhood file on every navigation while the sidebar is open] → The file is small and per page; the host's first paint and navigation time are unchanged, because the fetch starts after the host adopts the page.

[`mise run plugins:browser-test` runs outside `mise run ci`] → The unit tests in `tests/sidebar.test.cjs` and `tests/graph.test.cjs`, which CI runs, cover the initializer and the worker ending.

## Resolved Questions

### 1 - Does the change register initializers for media, slides and flashcards, or narrow the requirement?

Decided by the agent without codekiln: narrow the requirement. The browser check found all three working after a same-document navigation, and an initializer that does nothing would add host code with no behaviour behind it.

### 2 - Where does the Page graph initializer live?

Decided by the agent without codekiln: in `sidebar.js`, exposed as `window.logseqGardenerInitSidebar` and listed in `host.mjs`'s `inits`. The sidebar owns its cards, and the other initializers follow the same pattern. `window.logseqGardenerPage` would also work, but it runs the function at once on registration, which the sidebar has no use for.

### 3 - Does a card in a closed sidebar redraw at once?

Decided by the agent without codekiln: it redraws when the sidebar opens. Drawing a hidden card would fetch and lay out a graph the reader cannot see.

### 4 - Can the change be proposed, implemented and synced in one pull request?

Decided by the agent without codekiln: proposed and implemented in one pull request; synced and archived in a follow-up pull request tracked by a cleanup sub-issue, as `lsg/How To/Finish an OpenSpec Change After Merge` describes.

### 5 - How does the previous Page graph drawing stop?

Decided by the agent without codekiln: `graph.js` ends the previous drawing's worker when it draws the card again. An abort signal for every sidebar card also stopped the worker when a card closed, but it took `lsg-sidebar.js` to 6,530 gzip, over its 6,500 budget. Ending the worker on the next drawing keeps the sidebar script within budget and covers the navigation case.

## Open Questions

None.
