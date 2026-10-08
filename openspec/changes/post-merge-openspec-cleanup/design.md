## Context

The repository already requires spec synchronization before archival. Merged implementation changes still remain active, and contributors need an assigned follow-up that survives closure of the implementation issue.

## Goals / Non-Goals

**Goals:** Track cleanup under its implementation issue and document verification, synchronization, archival, and review from current main.

**Non-Goals:** Automated scheduling and GitHub Actions integration.

## Decisions

### Create a cleanup sub-issue for each implementation

Use GitHub's parent-child relationship to keep the follow-up visible after the implementation issue closes. The child records the source PR, exact change name, and acceptance checks. Contributors inspect existing children and cleanup PRs before creating another.

### Perform cleanup in a separate worktree after merge

Start from fetched main and verify the merged implementation against its artifacts. A separate follow-up PR allows review of main spec changes and archive moves while other implementation branches remain open.

### Verify every capability before archiving

Use the existing verification, synchronization, and archive skills. Run synchronization inline and compare every delta with its resulting main spec. Preserve later requirements on main and reconcile overlapping deltas in merge order before moving the change.

## Risks / Trade-offs

- A contributor can overlook an older merged change. Open cleanup children provide a discovery list; reconcile the active OpenSpec list with merged implementation PRs to recover missing children.
- A later implementation can change the same requirement. Compare the implementation and both deltas, retain later behavior, and record the reconciliation in the follow-up PR. Stop for an unresolved contradiction.

## Resolved Questions

### 1 - Where is cleanup tracked?

In a GitHub sub-issue of the implementation issue, with a separate follow-up PR targeting main.

### 2 - Does this need a repository-specific skill?

The focused guide supplies repository policy and links to the existing OpenSpec skills. A separate skill would repeat those execution steps.

## Open Questions

None.
