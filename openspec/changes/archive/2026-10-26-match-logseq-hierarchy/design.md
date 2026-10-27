## Context

The [hierarchy analysis](../../../../docs/parity/hierarchy.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `c622a4d`) with an `lsg` build of garden commit `0ce74d6`. Logseq's [`hierarchy.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/hierarchy.cljs#L13-L42) picks the page or its first namespaced alias, lists every namespace descendant or, for a leaf, its parent's path, sorts by lower-cased name and renders every row in the browser from the whole graph.

Today `crates/logseq-gardener-sdk/src/site/hierarchy.rs` both derives the generated namespace destinations and answers the rows, inside the publisher. `bottom_view` starts with the current page's row, ignores aliases, adds a `Namespace` badge, and inlines every row: `Person` has 1,067 rows, 480 KB of its 562 KB page.

[Issue #91](https://github.com/codekiln/logseq-gardener/issues/91) asked for rows of namespace pages without a file to stay out; this change keeps that rule. The [published reference sections spec](../../../specs/published-reference-sections/spec.md) already places Hierarchy between Linked and Unlinked References and loads section bodies from `_refs/` fragments through `references.js`.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's Hierarchy rows, labels and look on every page that has a namespace, and no section on pages that have none.
- The CLI and the language server list a namespace the same way the publisher does, from one SDK index.
- A namespace root's first paint stays within the `[desktop]` and `[phone]` budgets however many descendants it has.

**Non-Goals:**

- Rows for reference-only pages (a name used in a link, with no file and no children), which Logseq lists and `lsg` does not yet publish; they belong with [Issue #111](https://github.com/codekiln/logseq-gardener/issues/111) and [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115).
- Shift-click on a segment to open it in the right sidebar, which belongs with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).

## Decisions

**Where the work lives.** A new SDK module, `namespaces`, answers the rows; the publisher renders them and keeps the generated destinations it already derives.

| Operation | Answer |
| --- | --- |
| `NamespaceIndex::build(entries)` | The index over selected visible page titles, each `NamespaceEntry` holding the page key, its title, its aliases in authored order and whether it has a stored page. |
| `replace(entry)` / `remove(key)` | Swap one page's entry, so a renamed or deleted file touches only its own titles. |
| `hierarchy(page, RowFilter) -> Option<Hierarchy>` | The subject (the page or its first namespaced alias) and its rows: descendants at every depth, or the parent's path for a leaf, sorted by lower-cased name. `RowFilter::StoredOnly` leaves out descendants without a stored page; `RowFilter::All` keeps them. |
| `Row::segments() -> [Segment { label, prefix }]` | Each segment's label in the row's spelling and the lower-cased prefix it leads to. |

The index keeps titles in a `BTreeMap` keyed by lower-cased title, so a subtree is one range scan, as `bottom_view` does today. Generated prefixes are part of the index, so `RowFilter::All` matches Logseq's rows for pages that have files or children. `site/hierarchy.rs` keeps `Hierarchy::new` for the generated destinations and the collision check, and builds the `NamespaceIndex` from the same titles; `bottom_view` becomes a renderer of `hierarchy(page, StoredOnly)`.

**Markup.**

```html
<section class="page-hierarchy references" aria-label="Hierarchy" data-hierarchy-src="_refs/p-…-hierarchy-1.html">
  <details class="references-section" open>
    <summary><span class="ref-chevron"></span><h2>Hierarchy</h2></summary>
    <ul class="namespaces">
      <li><a class="page-ref" href="…"><span class="bracket">[[</span>tmux<span class="bracket">]]</span></a><span class="namespace-separator">/</span>…</li>
    </ul>
    <noscript><a href="_refs/p-…-hierarchy-1.html">Show more of the hierarchy</a></noscript>
  </details>
</section>
```

The header reuses the reference sections' `details.references-section` and chevron, so they fold, focus and look alike. Brackets reuse the inline page-reference markup and follow `settings.show_brackets`, as the reference sections do. Generated segments keep `page_status::link_attributes`.

**Long lists.** The first 100 rows are in the page. The rest go to `_refs/<page>-hierarchy-<n>.html`, about 500 rows per file, each with `<base href="../">` and a link to the next. `references.js` watches the last row with an `IntersectionObserver` and appends the next file's rows, as it does for reference groups. At about 40 bytes of gzip per row, 500 rows stay well under the 100 KB fragment budget, and `Microfreak`'s 745 stored rows need two requests at most.

**Styles.** The hierarchy rules move from the inline `site/mod.rs` styles to `site/references.css`, with the measured light values from the analysis: section margin-top 24px, `h2` 16px, weight 700, opacity 0.3; list margin `12px 24px`, padding 0, `circle` markers; rows `8px 0`; separator `0 8px` at opacity 0.3. Dark values reuse the existing tokens and are checked against the `gh-pages` build in dark mode during Apply.

**Budgets.** Measured by the [comparison harness](../../../specs/parity-comparison-harness/spec.md) on the whole encode garden:

| Measure | Budget |
| --- | --- |
| First paint of `Person`, `Microfreak`, `tmux`, `Logseq` | within the existing `[desktop]` and `[phone]` `data_before_title_kb` of 100 KB |
| Inline rows | 100 |
| Hierarchy styles on every page, gzip | 0.5 KB |
| `references.js` growth, minified | 1 KB |
| Each hierarchy fragment, gzip | 100 KB |
| Root page growth when a fixture namespace gains 2,000 pages | 1 KB |
| Peak phone heap after scrolling `Microfreak`'s list to the end | 20 MB |
| Whole-garden build time | at most 10% above the build without this change |
| Search latency per keystroke | unchanged; hierarchy rows add nothing to the search corpus |

`scripts/parity/budgets.toml` gains a `[hierarchy]` table: `parity:budgets` checks the row, style, script and fragment sizes and the grown-namespace fixture in CI, and `parity:perf` marks the heap and build values in `docs/parity/performance.md`.

## Risks / Trade-offs

- [Leaving out own rows removes the only navigation a namespaced leaf had to its ancestors] → The leaf's row is its parent's path, which links every ancestor, as in Logseq.
- [Rows split across files make in-page find miss later rows until they load] → The no-script link and the complete page index still reach every page; the first 100 rows cover every namespace in the garden except 17, counted from page files on garden commit `0ce74d6`.
- [Two indexes of titles, one for destinations and one for rows, could drift] → Both are built from the same `routes` map in one pass, and a test checks that every row's prefix has a destination or is withheld.
- [Dark values are not measured] → Apply checks them against the `gh-pages` build in dark mode before merging.

## Resolved Questions

### 1 - Do rows for namespace pages without a file appear?

Decided by the agent without codekiln: no, in the published site, as codekiln asked in Issue #91. The SDK offers both, so a CLI listing can show every prefix. Logseq shows 200 rows on `tmux`; `lsg` will show 160.

### 2 - What does a leaf whose parent has no file show?

Decided by the agent without codekiln: its parent's path, as Logseq does. The #91 rule removes generated rows from a descendant list, where the descendant's path already links them; a leaf's parent row is its only link upward, so it stays.

### 3 - Which alias does a page without a `/` use?

Decided by the agent without codekiln: the first namespaced alias in the order the author wrote them, as Logseq's `get-relation` takes the first of `:block/alias`. Authored order is stable across builds; Logseq's database order usually matches it.

### 4 - Do segments show brackets?

Decided by the agent without codekiln: yes, when `:ui/show-brackets?` is on, because Logseq renders each segment as a page reference. The reference sections already follow the same setting.

### 5 - Does the Namespace badge stay in the rows?

Decided by the agent without codekiln: no. Logseq has no badge, and the generated links keep their distinct style and accessible Namespace name, so the distinction from the existing spec survives without the extra text. The badge stays on generated pages and in the page index.

### 6 - How many rows are in the page?

Decided by the agent without codekiln: 100, with the rest in `_refs/` fragments of about 500 rows. Logseq renders every row from the downloaded graph; inlining them would keep `Person` at 480 KB of rows, and the epic requires first paint independent of garden size. 100 rows fill several screens, and only 17 namespaces in the garden have more.

### 7 - Where does the section sit?

Decided by the agent without codekiln: where the references change put it, between Linked and Unlinked References. This change adds no ordering rule of its own.

## Open Questions

None.
