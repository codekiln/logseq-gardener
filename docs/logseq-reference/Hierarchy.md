# Spec: namespace hierarchy

This board specifies how a `/` in a Logseq page name makes a tree, shown as a flat list of paths in a Hierarchy section.

- Board title: "A / in a page name makes a tree, shown as a flat list of paths".
- Gardener specs: `published-namespace-hierarchy`, `markdown-page-titles`.
- Logseq source: `hierarchy.cljs › structures, get-relation` (65 lines; the board says "port it directly").

## 1 · Files become namespaced pages

Garden source:

```markdown
pages/tmux.md
pages/tmux___Config.md
pages/tmux___Cheat Sheet.md
pages/tmux___Command___new-session.md
pages/tmux___Docs___Getting Started.md

# no file needed for tmux/Command
# or tmux/Docs: they exist because
# a child names them
```

Logseq renders: on page `tmux/Command/new-session`, the title "tmux/Command/new-session" (36px, weight 300, colour `#000000`, line-height 1.3) and the note "Title is the full original name, slashes included."

DOM and rules:

- `___` in a filename is `/` in the page name (older gardens use `%2F` or `.`; see `:file/name-format`).
- Every prefix (`tmux/Command`) is a real page even with no file: it opens, shows no blocks, and still gets Linked References and Hierarchy.

## 2 · Hierarchy section on the root page

Garden source:

```markdown
# on page: tmux
```

Logseq renders: a header "Hierarchy" (chevron-down icon 14px; text 16px, weight 700, opacity 0.3) and this list, one row per page, brackets grey:

```
[[tmux]]/[[Cheat Sheet]]
[[tmux]]/[[Command]]
[[tmux]]/[[Command]]/[[new-session]]
[[tmux]]/[[Config]]
[[tmux]]/[[Docs]]
[[tmux]]/[[Docs]]/[[Getting Started]]
```

DOM and rules:

- `div.page-hierarchy.mt-6` containing a foldable `h2.font-bold.opacity-30` "Hierarchy", open by default; the title toggles it.
- `ul.namespaces`, margin `12px 24px`; one `li.my-2` per page in the subtree, every depth, sorted by lower-cased name (board CSS: `.ns{list-style:none;margin:12px 24px;padding:0}`, `.ns li{margin:8px 0}`).
- Each segment is a page link labelled with that segment but pointing at the full prefix path (`Command` points to `tmux/Command`).
- Brackets follow `show-brackets?`.
- Separator: `span.mx-2.opacity-30` containing "/" (board CSS: margin `0 8px`, opacity .3).

## 3 · On a child page

Garden source:

```markdown
# on page: tmux/Config
```

Logseq renders: "Hierarchy" (16px, weight 700, opacity 0.3) and two rows:

```
[[tmux]]/[[Config]]
[[tmux]]/[[Config]]/[[local]]
```

DOM and rules:

- The list unions the page's descendants with its ancestor routes, de-duplicated and sorted.
- A leaf with no children still shows one row: its own full path, which doubles as a breadcrumb back up.
- A page with no `/` and no children has no Hierarchy section.
- If the page is an alias, the first namespaced alias is used.

## 4 · Where it sits

Garden source:

```markdown
# on page: tmux
```

Logseq renders: the section order on the page (15px list, Hierarchy in weight 600):

1. Page blocks
2. Pages tagged with "tmux"
3. 795 Linked References
4. Hierarchy
5. Unlinked References

DOM and rules:

- Order measured on the published `tmux` page.
- All Pages lists every namespaced page by its full name; there is no tree view outside this section.
