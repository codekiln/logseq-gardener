# Linked references, filters, unlinked references and tagged pages

Below each page's outline, Logseq shows up to four sections in this order: Pages tagged with, Linked References, Hierarchy, Unlinked References. Linked References counts referring blocks, includes blocks that refer to any page in the target's namespace, groups them by page with journals newest first, and puts a filter icon in the header that opens a dialog of include/exclude chips. Unlinked References lists blocks that mention the page name as plain text, and Logseq searches for them only when a reader opens the section.

`lsg` already publishes linked references grouped by page, with breadcrumbs and include/exclude filters, from the [published linked references spec](../../openspec/specs/published-linked-references/spec.md). The differences are what counts as a reference, the count text, the group order and look, the filter controls, and two missing sections: Pages tagged with and Unlinked References. On the encode garden, Logseq reports 589 linked references for `mise`; `lsg` reports 151, because it leaves out references to `mise/…` pages.

Matching Logseq's counting makes the reference sections larger, and they are already most of the largest pages: on `Answer/AI` they are 99% of a 2.2 MB page. The section bodies and the unlinked references should load when a reader opens them, so a page's first paint stays independent of how often other pages mention it. The relationships (references, namespace expansion, tagged pages, unlinked mentions, filter options) are graph questions that belong in the SDK; the publisher renders their answers.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `c622a4d` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `3f8e7c2` (`mise run parity:build`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from [lsg/Ref/Logseq Published Site/Backlinks](../pages/lsg___Ref___Logseq%20Published%20Site___Backlinks.md); the board's group colours, breadcrumb size and chip colours differ from the live build.

### Section order and visibility

[`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L507-L524) renders, after the outline:

- **Pages tagged with**, on every page that is not a zoomed block, when at least one page has this page in `tags::`.
- **Linked References**, on every page that is not a zoomed block. The section is left out entirely when no block refers to the page and no filter is set ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L241-L245)). On `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter` the page shows only Hierarchy and Unlinked References.
- **Hierarchy**, on pages that are not journals.
- **Unlinked References**, on every page outside the right sidebar, always shown with its header.

Each section header is a foldable row: a 14px chevron at `margin-left: -30px`, then the `h2`. The Linked References section sits in `.content.pt-6` (24px top padding); Pages tagged with and Unlinked References have a 24px top margin.

### What counts as a linked reference

- Logseq queries every block whose `:block/path-refs` include the page or one of its aliases ([`model.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1255-L1281)), then counts the blocks whose own `:block/refs` include it ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L208-L215)). Blocks on the page itself are left out.
- **Namespace parents count.** When a block refers to `[[mise/Config]]`, the parser adds `mise` to the block's references as well ([`graph_parser/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L358-L367)). So `mise` lists every block that refers to any `mise/…` page. This is the main reason `mise` has 589 linked references in Logseq and 151 referring contexts in `lsg`.
- Page properties count: a page whose `tags::` name the target appears as a group whose block is the page's property block (on `mise`, `Goose/How To/Use Mise to Set AWS Bedrock Env Vars for Desktop App` shows its `cgpt-link::` and `tags::` rows). Property names are also references in Logseq's file graphs, so names such as `card-last-interval` appear as filter options.

### Count text

- `N Linked References`, or `1 Linked Reference`; with any filter set, `F of N Linked References` ([`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L229-L237)). N counts referring blocks, before filters. There is no page count.
- Live: `h2.font-medium`, 16px, weight 500, `rgb(17,24,28)`, 25.6px row.
- Measured counts: `mise` reads `4 of 589 Linked References` on load (see filters below), `tmux` 795, `Claude/Code` 657, `Term` 193.

### Collapse

- The section starts collapsed when N reaches `:ref/linked-references-collapsed-threshold` ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L131-L133)). The encode garden sets it to 0, so every Linked References section starts collapsed and shows only its header.
- Each page group is foldable on its own and starts open.
- Groups render lazily as they scroll into view (`ui/lazy-visible`), and nothing inside the section is drawn until it is opened.

### Groups

```html
<div class="flex flex-col references-blocks-wrap">
  <div class="my-2 references-blocks-item">
    <div class="foldable-title">…chevron… <a class="page-ref" data-ref="2026-10-19 mon">2026-10-19 Mon</a></div>
    <div class="breadcrumb block-parents my-2">…</div>
    <div class="blocks-container">…the referring block and its children…</div>
  </div>
</div>
```

- One group per referring page ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3521-L3547)). When the page is an alias of the target, a muted `Alias` label follows its title.
- **Order:** journals first, newest first, then the other pages in database order ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3523)). On `mise` the first groups are `2026-10-19 Mon`, `2026-10-04 Sun`, `2026-09-10 Thu`; the last are `mise/use` and `mise/Tool/Core`. Within a group, blocks whose parent is the page come first.
- **Style, live:** background `rgb(241,243,245)`, 4px radius, 16px padding, `8px 0` margin, no border. The page title is a plain page link: 16px, weight 400, `rgb(16,107,163)`, no brackets.
- Each referring block renders with bullets and its children, as in the page outline.
- `mise` has 222 groups; `tmux` 258.

### Breadcrumbs

- A referring block that is not top-level gets a breadcrumb of up to 3 ancestors; when there are more, the oldest are replaced by `⋯` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2564-L2610)). The page is not part of the breadcrumb.
- Each crumb is the ancestor's first line with its inline markup, so page references keep their grey brackets: on `mise`, `[[Updated]] › mise tasks`.
- Crumbs are separated by a 20px `chevron-right` icon at 50% opacity with 4px side margins.
- **Style, live:** 16px body text `rgb(17,24,28)`, margin `8px 0 8px 24px`. The board's 14px muted crumbs do not match the live build.
- Clicking a crumb opens that ancestor as a zoomed block, `#/page/<uuid>` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2528-L2557)).

### Filter icon and dialog

- The header carries `a.filter` with title `Filter`: a funnel icon (`ti-filter`, 16px measured) at 60% opacity with no filter, `text-success` (`rgb(41,151,100)`) when every filter is an include, `text-error` when every filter is an exclude, `text-warning` when mixed ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L137-L157)). The red and orange values were not measured.
- Clicking it opens a centred modal, `.ls-filters` (704px wide inside a 768px panel, panel background `rgb(236,238,240)`, 6px radius) ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L43-L90), [`reference.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.css#L1-L20)):
  - A round grey funnel badge, the title `Filter` (18px, weight 500) and the hint `Click to include and shift-click to exclude. Click again to remove.` (12px).
  - `Includes:` and `Excludes:` rows listing the active filters as chips.
  - A search row (search icon, input with placeholder `Search in linked pages`, 16px, background `rgb(251,252,253)`, 4px radius). Typing narrows the chips by fuzzy match.
  - Chips for every other page referenced by the shown blocks, their ancestors or their children, with a superscript count, most frequent first. Chips are outline buttons: 24px tall, 12px text, weight 500, `0 12px` padding, 4px radius, white background, `1px solid rgb(226,232,240)` border. Include and exclude chips look the same as the others; the colour shows only on the header icon.
- **Options follow the filter.** The chip list and counts come from the filtered blocks ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L219-L240)). On `mise`, with `Diataxis/How To` included, the dialog shows 21 chips (`Diataxis 4`, `AWS 2`, …); after removing the include it shows 808 (`mise/Config 106`, `mise/Config/mise.toml 74`, `Gard 66`, …). The options include the referring pages themselves and property names.
- **Matching:** a block stays when its path references include every included page and none of the excluded pages; its ancestors are kept for context ([`handler/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/block.cljs#L253-L283)).
- **Initial filters come from the page.** Logseq reads the page's `filters::` property as the starting filter ([`handler/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/page.cljs#L748-L758)). Six garden pages carry one; `mise` has `filters:: {"diataxis/how to" true}`, so its header reads `4 of 589 Linked References` with a green icon before the reader does anything. Changes in the published build last for the session; Logseq's write-back to the file does nothing there.

### Unlinked references

- The header reads `Unlinked References` and the section starts collapsed ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L280-L292)). Opening it runs the search; the header then reads `N Unlinked References` (`1 Unlinked Reference`), where N counts blocks.
- **Search:** a case-insensitive match of the page name and each alias against the text of every block in the graph, with the logbook removed ([`model.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1320-L1347)). The name must not be preceded by a letter, digit, `#` or `[[`, and must not be followed by a letter or digit. Blocks on the page itself are left out. Blocks that also link the page are not left out, and the name can match inside another page's link text: on `tmux` the first result is `[[My/AI/Rule/Dev/Workflow/Git Worktree PR/Tmux]]`.
- Results use the same groups as linked references, without breadcrumbs and without filters.
- Measured: `mise` 2,720 unlinked references on 1,029 pages; `tmux` 856 on 250 pages.

### Pages tagged with

- Heading `Pages tagged with "<name>"`, where the name is the lower-cased page name (`Pages tagged with "claude/code"`) ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L228-L240)). The board shows the original case; the live build lower-cases it.
- The list holds every page whose `tags::` include this page, sorted by lower-cased name ([`model.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L78-L89)). Each row is a page link with no brackets.
- Foldable, open by default.
- **Style, live:** heading 16px, weight 700, opacity 0.5; list margin `8px 0 0 19.2px` with `circle` markers; rows 16px with `4px 0` margins; links `rgb(16,107,163)`.
- Measured: `Term` 178 pages, `Claude/Code` 38, `mise` 9, `tmux` 2.

### Performance in Logseq

Logseq's build downloads the whole graph (`static/js/db.gz`, 19.8 MB) before it draws any page. Every reference, filter and unlinked search then runs in the browser against that database, and the unlinked search scans every block's text each time a reader opens the section.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/backlinks.rs`, `backlinks.js` and the page styles in `site/mod.rs`, checked against the whole-garden build:

- **What counts:** page links, tags, embed targets and page-reference property values, through selected titles and aliases, deduplicated per referring block. References to namespace children (`[[mise/Config]]` for `mise`) do not count.
- **Count text:** `Linked references — 111 pages · 151 referring contexts` on `mise`, with an explanation of contexts inside the section. The page count leads; equal totals show only pages.
- **Zero references:** a heading reading `0 pages`; 3,529 pages in the build show one.
- **Collapse:** a native `<details>` that honours the collapse threshold and the open depth, so every section in the encode garden starts closed.
- **Section order:** Hierarchy comes before Linked references.
- **Groups:** one `section.backlinks-group` per page on `--lsg-surface` (`rgb(241,243,245)`), 5.6px radius, `12px 16px` padding, `16px 0` margin, with a bold `h3` link. Groups sort alphabetically by title, so journals come in date order oldest first and mix with other pages.
- **Breadcrumbs:** `p.backlink-context`, 13.6px, every ancestor as plain text separated by `›`, then a `↗` link to the referring block. There is no level limit: one `mise` breadcrumb has eight crumbs.
- **Filters:** an inline fieldset `Related pages` with a search box and one `Any / Include / Exclude` select per related page (329 on `mise`), a reset button and a live status line. Options are fixed at build time and do not follow the active filter. `filters::` is hidden from the properties and not applied.
- **Missing:** Pages tagged with and Unlinked References.
- **Size:** the reference section is inlined in each page and its script in every page. It is 449 KB of `mise`'s 511 KB, and 99% of `Answer/AI` (2.2 MB, 372 KB gzipped) and `note/my` (2.1 MB).
- **Where the code lives:** the reference index (`Backlinks::new`) is built inside the publisher's `site` module; no SDK operation answers "what refers to this page".

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Namespace parents.** A reference to `A/B` also counts as a reference to `A` (and to each ancestor namespace), as Logseq's parser does. Check: `mise` reports 589 referring blocks.
- 2. **Count text.** The header reads `N Linked References` / `1 Linked Reference` / `F of N Linked References`, counting blocks, at 16px weight 500. The page count and the context explanation go. Check: `mise`, `tmux`, `Claude/Code` headers.
- 3. **Zero references.** A page with no linked references has no Linked References section. Check: `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter`. This reverses the existing spec's zero-heading scenario.
- 4. **Section order.** Pages tagged with, Linked References, Hierarchy, Unlinked References. Check: DOM order on `mise`.
- 5. **Group order.** Journals newest first, then other pages, and within a group, top-level blocks first. Check: the first three `mise` groups are `2026-10-19 Mon`, `2026-10-04 Sun`, `2026-09-10 Thu`. Logseq's order for non-journal pages is database order; `lsg` keeps a stable order of its choosing (alphabetical) for them.
- 6. **Group look.** Background, 4px radius, 16px padding, `8px 0` margin, a regular-weight page link with no brackets, a fold control per group, and the `Alias` label for alias pages. Check: computed values on `mise`.
- 7. **Breadcrumbs.** At most 3 ancestors with `⋯` for the rest, rendered with inline markup, chevron separators, 16px text, margin `8px 0 8px 24px`; each crumb links to its block. Check: the `[[Updated]] › mise tasks` crumb on `mise`, and the eight-ancestor `mise` breadcrumb shortened to three.
- 8. **Filter control.** A funnel icon in the header with Logseq's four states, opening a dialog with Includes/Excludes rows, a search input and frequency-sorted chips with counts. Click includes, shift-click excludes, click again removes; the chips must also work from the keyboard (a key alternative to shift-click). Check: the `mise` dialog lists `mise/Config 106` first with no filter.
- 9. **Filter options follow the filter.** The chips and counts come from the blocks the current filter keeps. Check: on `mise` with `Diataxis/How To` included, `Diataxis 4` and `AWS 2` lead.
- 10. **Initial filters.** A page's `filters::` sets the starting filter, shown in the count and the icon colour; changes last for the visit. Check: `mise` opens at `4 of 589` with the green icon; `McKinsey` (`filters:: {}`) opens unfiltered.
- 11. **Unlinked references.** A collapsed `Unlinked References` section on every page, using Logseq's match rule, grouped like linked references without breadcrumbs or filters; the header gains its count once open. Check: `tmux` shows 856 after opening, `mise` 2,720.
- 12. **Pages tagged with.** The heading with the lower-cased name, the sorted list, the foldable header and the measured styles, shown only when some page tags this one. Check: `Term` lists 178 pages starting `1Password/Dev/CLI/Secret Reference`, `Abbreviation`.
- 13. **Performance.** The page's first paint stays within the `[desktop]` and `[phone]` budgets whatever the reference counts: the header and counts are in the page, and the section bodies (linked and unlinked) load when opened, with a plain link to the full list when scripts are off. The filter and loading script stays small and is not inlined into pages without references. Check: `parity:budgets` passes; `Answer/AI`, `note/my`, `mise` and `Claude/Code` stay within the first-paint budget after gap 1 grows their sections.

## Where the work belongs

- **SDK:** one reference index that the CLI, the language server and the publisher share. It answers: the blocks that refer to a page, with alias and namespace-parent expansion; their path references for filtering; the related-page frequencies for a set of blocks; the pages tagged with a page; and the unlinked mentions of a page. Unlinked mentions come from one pass over all block text that matches every page name and alias at once (a multi-pattern matcher such as Aho–Corasick), instead of one scan per page; with 6,990 pages and about 100,000 blocks, a scan per page would dominate build time. The index is derived from the Markdown and can be rebuilt incrementally for changed files. The current `Backlinks::new` moves behind these operations.
- **Publisher:** the section markup and styles, the header counts, the filter dialog and its script, writing each page's reference bodies as separate fragment files, and loading them on open.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `mise` | `filters::` include; namespace-parent references; 9 tagged pages; many unlinked mentions | `4 of 589`, 222 groups; 2,720 unlinked on 1,029 pages | 111 pages, 151 contexts |
| `tmux` | Many references, few tagged pages | 795 on 258 groups; 2 tagged; 856 unlinked on 250 pages | 131 pages, 197 contexts |
| `Claude/Code` | Largest reference section among common pages; lower-cased tagged heading | 657; 38 tagged | 211 pages, 307 contexts |
| `Term` | 178 tagged pages; counts close in both builds | 193; 178 tagged | 188 pages, 189 contexts |
| `Answer/AI`, `note/my` | Largest pages in the `lsg` build, almost all references | not measured | 2.2 MB and 2.1 MB pages |
| `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter` | No linked references | no section | `0 pages` heading |
| `McKinsey` | Empty `filters:: {}` | not measured | `filters::` hidden |

## Out of scope

- Block-level linked references (the count beside a referenced block and the references below a zoomed block), which belong with block zoom in #111.
- Shift-click on a group title or crumb to open it in the right sidebar, which belongs with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).
- Linked references on the journals home feed, which `lsg` shows as a summary link (`published-journals`).
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
