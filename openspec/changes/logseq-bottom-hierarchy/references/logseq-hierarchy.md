# Logseq hierarchy reference

Inspected local `github.com/logseq/logseq` source at tag `0.10.6` and file-branch commit `6efedb75588763af256bc7dfd0ed5526dc91fe7c`.

- [hierarchy.cljs](https://github.com/logseq/logseq/blob/6efedb75588763af256bc7dfd0ed5526dc91fe7c/src/main/frontend/components/hierarchy.cljs): `get-relation` gathers recursive namespace pages and sorts normalized page names. `structures` renders a flat list of complete paths, separate links for each cumulative prefix, slash separators, and a whole-section disclosure expanded by default.
- [page.cljs](https://github.com/logseq/logseq/blob/6efedb75588763af256bc7dfd0ed5526dc91fe7c/src/main/frontend/components/page.cljs): places hierarchy after page content and references.
- [model.cljs](https://github.com/logseq/logseq/blob/6efedb75588763af256bc7dfd0ed5526dc91fe7c/src/main/frontend/db/model.cljs): namespace queries retrieve recursive descendants. `get-page-namespace-routes` handles namespace suffix lookup for absent pages; it does not require additional ancestor rows for an existing page.

The requested publisher behavior explicitly adds the current full path as its first row, including on a namespaced leaf. The inspected Logseq component can show only a parent path for such a leaf. Retain the requested current row while using Logseq's flat full-path presentation and cumulative links.
