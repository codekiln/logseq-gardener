## Why

Logseq's Hierarchy section lists a page's namespace descendants, or its parent's path when it has none, and leaves the section off pages that have no namespace. `lsg` starts every list with the current page's row, so 525 plain pages and 5,297 namespaced leaves carry a section that only repeats their title. It also ignores aliases, uses its own look, and inlines every row: `Person` carries 1,067 rows, 480 KB of a 562 KB page ([hierarchy analysis](../../../../docs/parity/hierarchy.md), [Issue #116 — Match Logseq's namespace hierarchy section](https://github.com/codekiln/logseq-gardener/issues/116)).

The rows are a graph answer, and today the publisher computes them. Moving them into the SDK lets the CLI and the language server list a namespace the same way, and splitting long lists keeps a namespace root's first paint independent of its size.

## What Changes

- The SDK gains a namespace index shared by the CLI, the language server and the publisher. Given a page, it returns the rows Logseq shows: the page's descendants at every depth, or its parent's path when it has none, chosen through the first namespaced alias for a page without a `/`, sorted by lower-cased name, each row split into segments with their prefixes. A caller chooses whether rows for namespace pages without a file appear; the publisher leaves them out, as [Issue #91](https://github.com/codekiln/logseq-gardener/issues/91) asked. The publisher's `Hierarchy::bottom_view` moves behind it.
- The current page's own row leaves the list. A namespaced page with no published descendants shows its parent's path.
- A page with no `/`, no namespace children and no namespaced alias has no Hierarchy section.
- A page such as `AGI`, whose alias is `AI/General/Intelligence`, shows the hierarchy of that alias.
- Each segment's label comes from the row's own spelling.
- The section takes Logseq's look: the foldable header the reference sections use, a 16px bold heading at 30% opacity, circle markers, 8px row spacing, separators with 8px margins at 30% opacity, and segments shown as page references with brackets when `:ui/show-brackets?` is on. The `Namespace` badge leaves the rows; links to generated namespace pages keep their distinct style.
- The first 100 rows are in the page. Longer lists continue in fragment files in `_refs/` that load as the reader scrolls, with a plain link for readers without scripts.
- New budgets cap the inline rows, the hierarchy styles, the fragment size, the phone heap after scrolling `Microfreak`'s list to the end, and the build-time cost.
- **BREAKING** for readers of `lsg` output only: the own row, the one-row sections and the `Namespace` badge in rows are gone.

## Capabilities

### New Capabilities

- `namespace-index`: the SDK operation that answers a page's hierarchy rows from the page names and aliases, with the choice of page, descendants or parent, order, segments and the option to leave out pages without a file.

### Modified Capabilities

- `published-namespace-hierarchy`: which rows appear, the parent row for leaves, no section on plain pages, alias pages, labels, the look, the badge leaving the rows, splitting long lists, and the separator margins.

## Impact

- SDK: a new `namespaces` module beside `references` and `page_names`; `site/hierarchy.rs` keeps the generated destinations and renders the rows the SDK returns.
- Publisher: the section markup and styles in `site/hierarchy.rs`, `site/references.css` and `site/mod.rs`; hierarchy fragments in `_refs/`; `site/references.js` loads them on scroll.
- Parity harness: `scripts/parity/budgets.toml` gains a `[hierarchy]` table checked by `parity:budgets` and marked by `parity:perf`.
- Gitpa receives the same section in swapped `<main>` content; the references script initializes it again after a swap, as it does the reference sections.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
