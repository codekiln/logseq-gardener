## Context

The hierarchy already records whether a destination comes from selected visible source content. Its bottom view currently lists every generated prefix as another full path. Logseq does so too; the explicit reader preference here is to omit redundant intermediate rows.

## Goals / Non-Goals

Goals: retain current-page navigation, all stored descendants and every available path segment.

Non-goals: transcript timestamp controls and static search.

## Decisions

Filter generated descendants only when enumerating bottom-view rows. Keep the current row even when the current page is generated. Retain the destination graph and complete index so each omitted prefix remains reachable from stored descendant paths.

Use the existing selected-content classification. Inspecting filesystem existence would distinguish unpublished stored ancestors from absent files and change the publication boundary.

## Risks / Trade-offs

A generated page with many prefix levels has fewer rows. Its current path and complete stored descendant paths preserve namespace navigation.

## Resolved Questions

### 1 - Does a generated current page keep its own row?

Yes. Only generated descendant rows are omitted.

## Open Questions

None.
