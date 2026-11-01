# Namespace hierarchy section

Logseq's Hierarchy section lists a page's namespace descendants, sorted by lower-cased name, one path per row, with each segment linking to its prefix. It leaves out the page's own row. A namespaced page with no descendants shows its parent's path instead. A page that is neither namespaced nor a namespace root has no section. An alias page shows the hierarchy of its first namespaced alias.

`lsg` publishes the same kind of list from the [published namespace hierarchy spec](../../openspec/specs/published-namespace-hierarchy/spec.md), but always starts with the current page's row, so 525 pages without a `/` or children show a one-row section that Logseq leaves out. It also ignores aliases and uses its own styles. `lsg` also leaves out rows for namespace pages that have no file. That divergence is deliberate: codekiln asked for it in [Issue #91 — Omit redundant generated-only rows from the bottom hierarchy](https://github.com/codekiln/logseq-gardener/issues/91), and this analysis keeps it.

The section is inlined into the page and open, so a large namespace root carries every row in its HTML: `Person` has 1,067 rows, 480 KB of its 562 KB page. The rows are a graph answer that belongs in the SDK; the publisher renders them.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `c622a4d` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `914431f` (`mise run parity:build`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb`. Measured live values come first and win where they differ from the checkout or from [lsg/Ref/Logseq Published Site/Namespace Hierarchy](../pages/lsg___Ref___Logseq%20Published%20Site___Namespace%20Hierarchy.md). The board says a page shows its own row and that a leaf shows its own path; the live build shows neither.

### Which rows appear

[`hierarchy.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/hierarchy.cljs#L13-L42) (`get-relation`) picks the rows:

- **Which page.** The page itself when its name has a `/` or it has namespace children. Otherwise, the first of its aliases whose name has a `/`. Otherwise the page has no section. `AGI` (alias `AI/General/Intelligence`) shows the row `AI/General`; `Abbreviation` shows no section.
- **Descendants.** Every page in the namespace subtree at every depth ([`get-namespace-pages`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1545-L1560)), including pages that exist only because a child or a reference names them. The page's own row is not in the list. `tmux` has 200 rows starting `tmux/Bug`, `tmux/Bug/Vim Right Pane Insert Shows Up in Left Pane`, `tmux/Cheat Sheet`; `tmux/Command`, which has no file, has 19.
- **Leaf.** A namespaced page with no descendants shows one row, its parent namespace's path. `tmux/Config/local` shows `tmux/Config`; `tmux/Config` shows only `tmux/Config/local`.
- **Order.** Sorted by `:block/name`, the lower-cased full name, so a row follows its parent.
- **Labels.** Each row is its page's original name split on `/`, so a segment's label comes from the row's own spelling. Each segment links to the cumulative prefix (`Command` links to `tmux/command`).
- **Where.** After Linked References and before Unlinked References, on pages and file-less namespace pages; journals have no section. The section order belongs to [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115) (`match-logseq-references`).

Measured row counts: `Person` 2,358, `Microfreak` 2,548, `Logseq` 430, `tmux` 200, `GitP/A/Session/26/10/20-Tue` 40, `tmux/Command` 19, `1Password/Dev/Doc/SSHnGit` 10.

### Markup and style

```html
<div class="page-hierarchy mt-6">
  <div class="flex flex-row items-center foldable-title">…chevron… <h2 class="font-bold opacity-30">Hierarchy</h2></div>
  <ul class="namespaces" style="margin: 12px 24px">
    <li class="my-2"><a class="page-ref" data-ref="tmux">tmux</a><span class="mx-2 opacity-30">/</span><a class="page-ref" data-ref="tmux/config">Config</a>…</li>
  </ul>
</div>
```

- Foldable, open by default; clicking the title toggles it ([`ui/foldable`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/hierarchy.cljs#L64-L65)).
- Live: section `margin-top` 24px; `h2` 16px, weight 700, opacity 0.3, `rgb(17,24,28)`; list margin `12px 24px`, padding 0, `circle` markers; rows `8px 0` margins; separator `/` with `0 8px` margin at opacity 0.3.
- Each segment is a page reference, so it shows grey `[[` `]]` brackets, as the garden keeps `:ui/show-brackets?` at its default of true. Bracket styling belongs with [Issue #112](https://github.com/codekiln/logseq-gardener/issues/112).

### Performance in Logseq

Logseq builds every row in the browser from the whole graph after downloading `db.gz`, and renders all of them at once: 2,548 rows on `Microfreak`.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/hierarchy.rs` and the page styles in `site/mod.rs`, checked against the whole-garden build:

- **Rows.** The current page's row first, then every stored descendant at every depth, sorted by lower-cased title. Descendants with no file are left out (#91); their prefixes stay linked inside other rows. `tmux` has 161 rows, `tmux/Command` 13, `Person` 1,067, `Microfreak` 745.
- **Own row.** Every page shows its own row. 5,297 namespaced leaves and 525 pages with no `/` or children show a one-row section.
- **Aliases.** Not used; `AGI` shows the row `AGI`.
- **Generated pages.** A namespace page with no file is generated and shows its own row with a `Namespace` badge (`tmux/Command Namespace`).
- **Labels.** A segment's label comes from the prefix page's own title, and the current page's last segment carries `aria-current="page"`. Unavailable prefixes (private or withheld) show as plain text.
- **Markup and style.** `section.namespace-hierarchy` with a native `<details open>`; summary 20px, weight 600, full opacity; list without markers, `12px 0 0` margin; separator margin `0.2em`; no brackets.
- **Where.** Before Linked References.
- **Size.** Rows are inlined in the page. The largest sections are `Person` (1,067 rows, 480 KB, 43 KB gzip, page 58 KB gzip), `Microfreak` (745 rows, 374 KB) and `Microfreak/UG` (367 rows, 256 KB). Each row repeats a link for every segment, so the size grows with rows times depth.
- **Where the code lives.** `Hierarchy::new` and `bottom_view` are in the publisher's `site` module; no SDK operation answers "what is in this namespace".

## Gaps

Each gap names a check the Apply step can run against the fixture pages. Rows for namespace pages without a file stay out, per #91.

- 1. **No own row.** The list holds the descendants only. Check: `tmux/Config` shows one row, `tmux/Config/local`.
- 2. **Leaf row.** A namespaced page with no published descendants shows its parent's path. Check: `tmux/Config/local` shows `tmux/Config`.
- 3. **No section for plain pages.** A page without a `/`, without descendants and without a namespaced alias has no Hierarchy section. Check: `Abbreviation` has none; the 525 one-row sections go.
- 4. **Aliases.** A page that is not namespaced uses its first namespaced alias. Check: `AGI` shows `AI/General`.
- 5. **Labels.** Each segment's label comes from the row's own spelling. Check: a row under a prefix spelled differently in another title keeps its own case.
- 6. **Look.** Foldable header with the chevron, `h2` at 16px, weight 700, opacity 0.3; list margin `12px 24px` with circle markers; rows with `8px 0` margins; separator with `0 8px` margin at opacity 0.3; section margin-top 24px; segments as page references with brackets. The `Namespace` badge leaves the rows. Check: computed styles on `tmux`.
- 7. **Size.** The section keeps large namespace roots within the `[desktop]` and `[phone]` first-paint budgets and bounds phone memory: the first rows are in the page and the rest load as the reader scrolls to them, with a plain link for readers without scripts. Check: `Person` and `Microfreak` stay within the first-paint budget, and adding 2,000 pages to one namespace in the `parity:budgets` fixture garden does not grow the root page beyond its budget.

## Where the work belongs

- **SDK:** a namespace operation that the CLI, the language server and the publisher share. Given a page, it returns the rows Logseq would show (the alias choice, the descendants or the leaf's parent, the order and each row's segments with their prefixes), from the same incremental index as references. It keeps the #91 rule as an option the publisher sets, so other tools can still list every namespace page. `Hierarchy::new` and `bottom_view` move behind it.
- **Publisher:** the section markup and styles, splitting long lists into inline rows plus fragments, and routes for generated namespace pages, which stay as they are.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `tmux` | Namespace root with many file-less descendants | 200 rows | 161 rows, own row first |
| `tmux/Config` | One descendant | `tmux/Config/local` | own row plus `tmux/Config/local` |
| `tmux/Config/local` | Namespaced leaf | `tmux/Config` | own row |
| `tmux/Command` | Namespace page without a file | 19 rows | 13 rows, own row with badge |
| `AGI` | Alias `AI/General/Intelligence` | `AI/General` | own row `AGI` |
| `Abbreviation` | No `/`, no children | no section | own row |
| `GitP/A/Session/26/10/20-Tue` | Gitpa session with deep asset paths (#91's case) | 40 rows | 25 rows |
| `Person`, `Microfreak` | Largest sections | 2,358 and 2,548 rows | 1,067 rows (480 KB) and 745 rows (374 KB) |

## Out of scope

- Rows for namespace pages without a file, which codekiln chose to leave out in #91.
- Reference-only pages (a page named by a reference with no file and no children), which `lsg` does not publish; they belong with [Issue #111](https://github.com/codekiln/logseq-gardener/issues/111) and [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115).
- The position of the section among the reference sections, which [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115) changes.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
