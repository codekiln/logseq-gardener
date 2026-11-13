## 1. Reproduce relationships

- [x] 1.1 Pin current and historical source revisions, lock the shared runtime, and extract source without changing upstream checkouts.
- [x] 1.2 Run the shared fixtures through both parsers under default and triple-lowbar filename configuration; save input hashes and relationship results.
- [x] 1.3 Compare current results with historical results and the earlier saved snapshot; document disagreements and page lookup implications.

## 2. Verify and document

- [x] 2.1 Add mise check/update tasks and reproduction instructions, including source versions, licenses, and normalization limits.
- [x] 2.2 Demonstrate that a changed fixture fails the check, run strict OpenSpec validation and project CI, and record outcomes.
