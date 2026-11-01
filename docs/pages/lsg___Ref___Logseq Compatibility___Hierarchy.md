tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Namespace Hierarchy]], [[lsg/Ref/Published Pages]]

- # Hierarchy Compatibility
	- `lsg` publishes Logseq's Hierarchy section: a page's namespace descendants at every depth, sorted by lower-cased name, one path per row, each segment in the row's own spelling linking to its prefix and shown with page-reference brackets; a namespaced page with no descendants shows its parent's path, a page without a `/` uses its first namespaced alias, and the section sits between Linked References and Unlinked References with Logseq's look. It leaves out the rows of namespace pages that have no file, and it puts the first 100 rows in the page and loads the rest as the reader scrolls, where Logseq renders every row at once.
	  id:: dd3b36b7-b73d-4c1c-ae59-d685c3ee1cbd
	- Measured on 2026-10-25 against the `gh-pages` build of garden commit `c622a4d` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `914431f` (`mise run parity:build`). The OpenSpec change [match-logseq-hierarchy](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-26-match-logseq-hierarchy) then rebuilt the section, and [published-namespace-hierarchy](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-namespace-hierarchy/spec.md) holds its requirements.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb`. Measured live values come first and win where they differ from the checkout or from [[lsg/Ref/Logseq Published Site/Namespace Hierarchy]]. The board says a page shows its own row and that a leaf shows its own path; the live build shows neither.
		- ### Which rows appear
			- [`hierarchy.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/hierarchy.cljs#L13-L42) (`get-relation`) picks the rows.
			- **Which page.** The page itself when its name has a `/` or it has namespace children. Otherwise, the first of its aliases whose name has a `/`. Otherwise the page has no section. `AGI` (alias `AI/General/Intelligence`) shows the row `AI/General`; `Abbreviation` shows no section.
			- **Descendants.** Every page in the namespace subtree at every depth ([`get-namespace-pages`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1545-L1560)), including pages that exist only because a child or a reference names them. The page's own row is not in the list. `tmux` has 200 rows starting `tmux/Bug`, `tmux/Bug/Vim Right Pane Insert Shows Up in Left Pane`, `tmux/Cheat Sheet`; `tmux/Command`, which has no file, has 19.
			- **Leaf.** A namespaced page with no descendants shows one row, its parent namespace's path. `tmux/Config/local` shows `tmux/Config`; `tmux/Config` shows only `tmux/Config/local`.
			- **Order.** Sorted by `:block/name`, the lower-cased full name, so a row follows its parent.
			- **Labels.** Each row is its page's original name split on `/`, so a segment's label comes from the row's own spelling. Each segment links to the cumulative prefix (`Command` links to `tmux/command`).
			- **Where.** After Linked References and before Unlinked References, on pages and file-less namespace pages; journals have no section.
			- Measured row counts: `Person` 2,358, `Microfreak` 2,548, `Logseq` 430, `tmux` 200, `GitP/A/Session/26/10/20-Tue` 40, `tmux/Command` 19, `1Password/Dev/Doc/SSHnGit` 10.
		- ### Markup and style
			- ```html
			  <div class="page-hierarchy mt-6">
			    <div class="flex flex-row items-center foldable-title">…chevron… <h2 class="font-bold opacity-30">Hierarchy</h2></div>
			    <ul class="namespaces" style="margin: 12px 24px">
			      <li class="my-2"><a class="page-ref" data-ref="tmux">tmux</a><span class="mx-2 opacity-30">/</span><a class="page-ref" data-ref="tmux/config">Config</a>…</li>
			    </ul>
			  </div>
			  ```
			- Foldable, open by default; clicking the title toggles it ([`ui/foldable`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/hierarchy.cljs#L64-L65)).
			- Live: section `margin-top` 24px; `h2` 16px, weight 700, opacity 0.3, `rgb(17,24,28)`; list margin `12px 24px`, padding 0, `circle` markers; rows `8px 0` margins; separator `/` with `0 8px` margin at opacity 0.3.
			- Each segment is a page reference, so it shows grey `[[` `]]` brackets, as the garden keeps `:ui/show-brackets?` at its default of true. [[lsg/Ref/Logseq Compatibility/Inline Markup]] describes the brackets.
		- ### Performance in Logseq
			- Logseq builds every row in the browser from the whole graph after downloading `db.gz`, and renders all of them at once: 2,548 rows on `Microfreak`.
	- ## What lsg publishes
		- Every published page and generated namespace page that has a hierarchy shows a Hierarchy section between Linked References and Unlinked References, in Logseq's order. Journal pages and pages without a hierarchy have no section, so `Abbreviation`, with no `/`, no descendants and no namespaced alias, has none. [[lsg/Ref/Published Pages]] describes the section for readers.
		- The rows are the page's namespace descendants at every depth, sorted by lower-cased name. The page has no row of its own: `tmux/Config` shows the single row `tmux/Config/local`.
		- A namespaced page with no descendants shows its parent's path: `tmux/Config/local` shows `tmux/Config`, with `tmux` and `Config` linked.
		- A page without a `/` uses the first namespaced alias in the order the author wrote them: `AGI` shows `AI/General`.
		- Each row shows its full path with slash separators. Each segment is labelled with the row's own spelling and links to its cumulative prefix when that prefix has a destination. A private, withheld or alias-conflicting prefix stays plain text in its row and gets no destination.
		- When the garden's `:ui/show-brackets?` is on, each segment shows the page-reference brackets that inline page references show.
		- The section uses the reference sections' foldable header, `details.references-section` with the chevron, open by default and operable from the keyboard. The `Hierarchy` heading is 16px, weight 700, opacity 0.3; the list has a `12px 24px` margin, no padding and circle markers; rows have `8px 0` margins; each `/` has an 8px margin on either side at opacity 0.3. Long paths wrap on narrow screens.
		- Links to generated namespace destinations keep their dotted underline and accessible Namespace name, and rows carry no Namespace badge. A generated namespace page itself shows a Namespace notice.
		- A page's HTML holds at most the first 100 rows. The remaining rows go to fragment files in `_refs/`, about 500 rows each, and `lsg-references.js` appends the next file as the reader scrolls near the end of the list. Without scripts, the section ends with a link to the first fragment, and each fragment links to the next.
		- The SDK's `namespaces` module answers the rows: `NamespaceIndex::hierarchy` gives the subject, chosen from the page or its first namespaced alias, and its rows. `RowFilter::StoredOnly` leaves out descendants without a stored page, as the publisher uses it; `RowFilter::All` keeps them, as Logseq's rows do. The publisher renders the section, splits long lists into fragments and keeps its generated namespace destinations.
		- On `Microfreak`, measured on 2026-10-26 with the change at `c373d90` on garden commit `083106d`, the page held 100 rows and scrolling to the end of the list loaded 1,394, with a peak JavaScript heap of 3.0 MB on desktop and 3.1 MB on the phone profile. [[lsg/Ref/Logseq Compatibility/Performance]] holds the rows, and the `[hierarchy]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) holds the budgets: 100 inline rows, 0.5 KB gzip of hierarchy styles on every page, 1 KB of growth in `lsg-references.js`, 100 KB gzip per fragment, 1 KB of root page growth when a namespace gains 2,000 pages, 20 MB of phone heap after scrolling `Microfreak`'s list, and a whole-garden build at most 10% slower.
	- ## Differences from Logseq
		- `lsg` leaves out the rows of namespace pages that have no file, as codekiln asked in [issue 91](https://github.com/codekiln/logseq-gardener/issues/91); their prefixes stay linked inside the rows of their descendants. Logseq lists them, so it shows 200 rows on `tmux` and 19 on `tmux/Command`.
		- `lsg` lists only published pages. Logseq also lists pages that exist only because a reference names them.
		- `lsg` puts the first 100 rows in the page and loads the rest from fragment files as the reader scrolls. Logseq renders every row at once from the downloaded graph.
		- A generated namespace page shows a Namespace notice, and links to it have a dotted underline. Logseq shows a page without a file like any other page.
		- Logseq's dark-theme values were not measured; the section uses `lsg`'s theme tokens.
	- ## Before the change
		- The `lsg` build measured on 2026-10-25, from `914431f` on garden commit `0ce74d6`, wrote the section from `crates/logseq-gardener-sdk/src/site/hierarchy.rs` and the page styles in `site/mod.rs`.
		- **Rows.** The current page's row first, then every stored descendant at every depth, sorted by lower-cased title. Descendants with no file were left out, as issue 91 asked; their prefixes stayed linked inside other rows. `tmux` had 161 rows, `tmux/Command` 13, `Person` 1,067, `Microfreak` 745.
		- **Own row.** Every page showed its own row. 5,297 namespaced leaves and 525 pages with no `/` or children showed a one-row section that Logseq leaves out.
		- **Aliases.** Not used; `AGI` showed the row `AGI`.
		- **Generated pages.** A namespace page with no file was generated and showed its own row with a `Namespace` badge (`tmux/Command Namespace`).
		- **Labels.** A segment's label came from the prefix page's own title, and the current page's last segment carried `aria-current="page"`. Unavailable prefixes (private or withheld) showed as plain text.
		- **Markup and style.** `section.namespace-hierarchy` with a native `<details open>`; summary 20px, weight 600, full opacity; list without markers, `12px 0 0` margin; separator margin `0.2em`; no brackets.
		- **Where.** Before Linked References.
		- **Size.** Rows were inlined in the page and the section was open. The largest sections were `Person` (1,067 rows, 480 KB of its 562 KB page, 43 KB gzip, page 58 KB gzip), `Microfreak` (745 rows, 374 KB) and `Microfreak/UG` (367 rows, 256 KB). Each row repeated a link for every segment, so the size grew with rows times depth.
		- **Where the code lived.** `Hierarchy::new` and `bottom_view` were in the publisher's `site` module; no SDK operation answered what is in a namespace.
	- ## Fixture pages
		- | Page | Why | Logseq | `lsg` on 2026-10-25, before the change |
		  | --- | --- | --- | --- |
		  | `tmux` | Namespace root with many file-less descendants | 200 rows | 161 rows, own row first |
		  | `tmux/Config` | One descendant | `tmux/Config/local` | own row plus `tmux/Config/local` |
		  | `tmux/Config/local` | Namespaced leaf | `tmux/Config` | own row |
		  | `tmux/Command` | Namespace page without a file | 19 rows | 13 rows, own row with badge |
		  | `AGI` | Alias `AI/General/Intelligence` | `AI/General` | own row `AGI` |
		  | `Abbreviation` | No `/`, no children | no section | own row |
		  | `GitP/A/Session/26/10/20-Tue` | Gitpa session with deep asset paths, the case in issue 91 | 40 rows | 25 rows |
		  | `Person`, `Microfreak` | Largest sections | 2,358 and 2,548 rows | 1,067 rows (480 KB) and 745 rows (374 KB) |
