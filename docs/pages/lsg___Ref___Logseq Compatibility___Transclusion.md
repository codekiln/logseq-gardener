tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Transclusion]], [[lsg/Ref/Rendering and Assets]], [[lsg/Ref/Published Pages]]

- # Transclusion Compatibility
	- `lsg` publishes block references and embeds as Logseq does: a block reference shows the referenced block's whole first line with its inline markup and task checkbox above a dashed underline, an invalid one shows Logseq's `Block ref invalid` warning, and block and page embeds are shaded wells holding a full outline with bullets and collapse, a page embed adding a header with the page icon and a bold link, each nested level a darker shade up to Logseq's nesting limit of 5. A reference links to the block's anchor on its page, where Logseq opens the block as its own page, and it has no hover preview. A heading in a reference is a span with a heading role, and the page embed header of an unpublished page shows its name as text.
	  id:: cc714781-fc02-4374-b225-f97deb5488db
	- Measured on 2026-10-25 against the `gh-pages` build of garden commit `7736d6f` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden from `54a2b52` (`mise run parity:build`). The OpenSpec change [match-logseq-transclusion](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-26-match-logseq-transclusion) then rendered them like Logseq, and [published-transclusion](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-transclusion/spec.md), [transclusion-content](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/transclusion-content/spec.md) and [selected-content-transclusion](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/selected-content-transclusion/spec.md) hold their requirements.
	- ## How often the garden uses them
		- | Form | Uses | Files |
		  | --- | --- | --- |
		  | Block reference `((uuid))` | 506 | 164 |
		  | Labelled block reference `[label](((uuid)))` | 0 | 0 |
		  | Block embed `{{embed ((uuid))}}` | 164 | 73 |
		  | Page embed `{{embed [[Page]]}}` | 803 | 256 |
		- In the `lsg` build of 2026-10-25, 74 block references and 7 embeds fell back with a diagnostic. 13 of the 19 distinct unresolved references point at an `id::` that exists nowhere in the garden. The other 6 point at bullets whose content is a quote (`- > …` with `id::` on the next line), for which `lsg` registered no target then. Logseq resolves those references and shows the quote, because a reference shows a block's body when its title is empty.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where the checkout's CSS differs; the embed wells in particular are styled differently live than in the checkout's `block.css`.
		- ### Block references
			- ```html
			  <div class="block-ref-wrap inline" data-type="default">
			    <span class="block-ref">
			      <div class="block-content inline" id="block-content-<uuid>" blockid="<uuid>">
			        <div class="flex flex-row justify-between block-content-inner">
			          <div class="flex-1 w-full"><span class="inline done"><input type="checkbox" checked>figure out <span class="page-reference">…</span> …</span></div>
			        </div>
			      </div>
			    </span>
			  </div>
			  ```
			- Built by `block-reference` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L866-L940)) through `block-content` with `:block-ref? true`.
			- **Text:** the referenced block's title only, the first line, rendered with its full inline markup: page references with brackets, tags, code. A task marker becomes its checkbox (on `Claude/Code`, a `DONE` block shows a checked box and struck-through style). Properties are hidden and the body and children are left out ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2278), [L2292-L2301](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2292-L2301), [L2363-L2368](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2363-L2368)). A heading block renders as an inline `h1`–`h6` at 1rem ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L439-L447)).
			- **Style, live:** text in the body colour `rgb(17,24,28)`, 16px, weight 400, `2px 0` padding, a `0.5px dashed rgb(16,107,163)` bottom border, `cursor: alias`. The checkout's CSS gives a solid `#d8e1e8` border; the live build differs.
			- **Labelled references** show the label's markup inside `span.block-ref` instead of the block ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L882-L886)). The garden has none.
			- **Click** opens the referenced block as its own page, `#/page/<uuid>`; shift-click opens it in the right sidebar ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L893-L923)).
			- **Hover** shows, after 1 second, a 735px popover with a breadcrumb and the block with its children ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L925-L940)).
			- **Unknown uuid:** `span.warning` titled `Block ref invalid`, showing `((uuid))` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L942-L945)). A reference to a block whose title is empty, such as a bullet whose content is only a quote, shows the block's body ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2278-L2284)); on `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` both references show their quotes.
		- ### Block embeds
			- ```html
			  <div class="color-level embed-block bg-base-2" style="z-index:2">
			    <div class="px-3 pt-1 pb-2">
			      <div class="blocks-container flex-1">…the block and its children, with bullets…</div>
			    </div>
			  </div>
			  ```
			- Built by `block-embed` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L798-L814)). The embedded block and all its children render as a normal outline with bullets and collapse arrows; each block's own `collapsed::` sets its initial state ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2829-L2834)). The embedded root hides its reference count.
			- **Style, live** on `Spotify/Tool/TUI`: background `rgb(248,249,250)`, 8px radius, 8px margin, `12px 8px` padding; the inner `div` adds `4px 12px 8px`. There is no header.
			- An unknown uuid or an argument that is not a reference renders nothing ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1271-L1289)).
		- ### Page embeds
			- ```html
			  <div class="color-level embed embed-page bg-base-2">
			    <section class="flex items-center p-1 embed-header">
			      <div class="mr-3"><svg class="h-5 w-4">…page icon…</svg></div>
			      <a class="page-ref" data-ref="ai/coding/history/phase/1 - primitive">AI/Coding/History/Phase/1 - Primitive</a>
			    </section>
			    <div class="blocks-container">…every top-level block with its children…</div>
			  </div>
			  ```
			- Built by `page-embed` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L816-L842)).
			- **Style, live** on `AI/Coding/History`: the well has the same background, radius, margin and padding as a block embed. The header has `4px` padding, a `0 0 5px` margin and weight 600; the page link is `rgb(16,107,163)`, 19.2px, weight 700, with no brackets. The icon is a 16×20 page glyph in the theme's accent colours.
			- **Nested embeds** are wells inside wells. Live, the second level is `rgb(233,236,239)`. The checkout's CSS steps through `--color-level-2` to `--color-level-6` and then repeats levels 4, 5 and 6 ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L481-L527)); `AI/Coding/History` nests only two levels, so the deeper shades were not measured. Dark values come from `--color-level-*` in [`vars-classic.css`](https://github.com/logseq/og/blob/6e7afa8eb/packages/ui/src/vars-classic.css#L106-L173) and were not measured on 2026-10-25; on 2026-10-26 the dark second level measured `rgb(56,67,76)`.
			- **Missing page:** the well and header render with an empty body. **The page itself or the enclosing embed:** only the header renders ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L826-L830)).
		- ### Depth and cycles
			- References and embeds share one depth counter with a limit of 5 ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L96)). Past it, Logseq shows `p.warning` reading `Embed depth is too deep` or `Block ref nesting is too deep` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1142-L1143), [L1274-L1275](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1274-L1275)). The only cycle check is the page-embed self check above; other cycles stop at the depth limit.
		- ### Where the reference boards differ
			- [[lsg/Ref/Logseq Published Site/Transclusion]] gives the reference text the link colour, mixes block and page embed styles into one `.embed` rule with a 4px radius, and draws only a header for any embed already in the chain. The live build shows body-coloured reference text, 8px radius on both embed kinds, and the narrower cycle check above. This page follows the live build.
	- ## What lsg publishes
		- A resolved block reference renders as a link to the block's anchor on its page, holding `span.block-ref` with the referenced block's whole first line through the same inline renderer as any block. A task marker renders the outline's checkbox, with the done style for `DONE`; on `Claude/Code` the first reference shows a checked checkbox and a page reference with its brackets. Properties are removed and the body and children are left out. When the first line is empty, the reference shows the block's body instead, as Logseq does, so a reference to a quote bullet shows the quote. [[lsg/Ref/Rendering and Assets]] describes how pages render for readers.
		- The reference uses the body text colour, a `0.5px` dashed bottom border in the link colour, `2px 0` padding and `cursor: alias`, in both themes. A labelled reference shows its label inside `span.block-ref`.
		- Inside a reference's link, page references and tags render as spans with their usual classes, because links cannot nest; other links show their label. A heading block renders as `span.block-ref-heading` with `role="heading"`, its `aria-level` and `font-size: 1rem`.
		- A bullet with an `id::`, `custom-id::` or `custom_id::` property is a reference target, including a bullet whose content is a quote. A uuid with more than one owner is dropped. The two references on `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` resolve with no diagnostic.
		- A reference whose uuid no published block carries renders `span.warning` titled `Block ref invalid`, showing `((uuid))`, and the build reports a diagnostic.
		- A block embed renders as `div.embed-block` and a page embed as `div.embed-page`, each with Logseq's well: background, 8px radius, 8px margin and `12px 8px` padding, in both themes. The embedded blocks render through the outline renderer with bullets and collapse arrows, honouring each block's `collapsed::`, with no duplicate HTML IDs, so the outline script collapses blocks inside wells.
		- A page embed starts with a header holding Logseq's page icon, as an inline SVG, and a link to the page in the link colour at 19.2px and weight 700, with no brackets. On `AI/Coding/History` each embed's header links to its phase page.
		- Each nested well takes Logseq's next shade from level 2 to level 5, through the `data-level` attribute and the theme tokens `--lsg-embed-1` to `--lsg-embed-5`; the second level on `AI/Coding/History` is `rgb(233,236,239)` in the light theme and `rgb(56,67,76)` in the dark theme.
		- An unavailable block embed renders nothing; a page embed whose page has no published file renders the well and header with no body; a self embed renders only the header. A page that is not published appears in the header as text, without a link. Each case keeps its diagnostic.
		- References and embeds share Logseq's nesting limit of 5 levels. Past it, an embed renders `p.warning` reading `Embed depth is too deep` and a block reference renders `span.warning` reading `Block ref nesting is too deep`; a test garden that chains seven page embeds shows the warning at the sixth level. The publisher keeps its own total-work and expansion budgets behind that limit.
		- Shift-click on a block reference adds a Block references card to the right sidebar, as [[lsg/Ref/Logseq Compatibility/Right Sidebar]] describes.
		- Search keeps one entry per block; embedded copies add no search text.
		- The SDK's `transclusion` module answers what a reference or an embed shows: `ReferenceTargets` indexes the uuids of selected visible pages, `reference_content` gives a reference's first line, marker, heading and body, and `embed_outline` gives an embed's outline or why it is unavailable, applying the nesting limit. The publisher renders each answer, with the styles in `site/transclusion.css` inlined into every page.
	- ## Differences from Logseq
		- A reference links to the block's anchor on its page, and block zoom gives the focused view. Logseq opens `#/page/<uuid>`, which needs its client-side router.
		- A reference has no hover preview. Logseq shows a 735px popover with a breadcrumb and the block after 1 second.
		- A heading in a reference renders as `span.block-ref-heading` with a heading role. Logseq draws an inline `h1`–`h6`, which would close a surrounding paragraph in static HTML.
		- `Block ref nesting is too deep` is a `span.warning`. Logseq uses `p.warning`, which would close the enclosing reference's paragraph.
		- The reference markup leaves out Logseq's wrapper `div`s, since a `div` inside an inline link is invalid HTML; `span.block-ref` carries the same styles.
		- The page embed header of an unpublished page shows the name as text. Logseq links to an empty page, which a static site does not have.
		- An embed inside a referenced block's first line is left out, with a diagnostic. Logseq draws the embed inside the reference, but an outline cannot sit inside the reference's link. The encode garden has no such case.
		- The stylesheet has no sixth shade and no repeats past it, since the nesting limit of 5 keeps a well from reaching level 6.
		- In the dark theme, the reference underline and the page embed link use `lsg`'s `--lsg-ref` token. Logseq's dark `rgb(163,206,241)` comes from the theme the garden's `logseq/custom.css` imports, which `lsg` loads only through `--stylesheet`.
	- ## Performance
		- The `[transclusion]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) limits the transclusion styles every page carries to 1 KB gzip. Transclusion adds no script beyond the outline script, and the fixture page that embeds the most stays within the 100 KB first-paint budget. `parity:budgets` checks the style size in CI and fails when no fixture page holds an embed; the fixture garden's `Tools/Embeds` holds both kinds of well.
		- Measured on 2026-10-26 with the change, against `lsg` `b490d68` before it: the transclusion styles were 407 bytes gzip on every page, and first paint, gzip, grew as follows.
			- | Page | Before | After | Budget |
			  | --- | ---: | ---: | --- |
			  | `Claude/Code` | 148,879 | 149,340 | 100,000; over before this change |
			  | `Spotify/Tool/TUI` | 12,133 | 12,467 | 100,000 |
			  | `AI/Coding/History` | 12,743 | 13,567 | 100,000 |
			  | `Py/Zen of Python` | 11,058 | 11,707 | 100,000 |
			  | `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` | 9,208 | 9,545 | 100,000 |
			  | `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` | 28,665 | 30,280 | 100,000 |
			  | Largest growth in the garden (`Microfreak/UG/03 Overview/01 Front Panel/03 Bottom Row`) | 12,378 | 14,572 | recorded |
			  | Script before the title, every fixture page | 9,012 | 9,012 | no growth |
		- Six encode-garden pages were over 100 KB gzip before the change and still were after it (`Answer/AI`, `note/my`, `Claude/Code`, `Keyshort`, `Update`, `note/AI`). Their size comes from their own outlines and linked references, and transclusion added at most 1.9 KB to any of them.
		- [[lsg/Ref/Logseq Compatibility/Performance]] records the embedding fixtures on 2026-10-26: `AI/Coding/History` showed its title in 23.7 ms on desktop and 231 ms on the phone profile with 15.4 KB of data before it, and `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` in 35.3 ms and 235 ms with 32.1 KB.
	- ## Before the change
		- The `lsg` build measured on 2026-10-25, from `54a2b52`, resolved the same targets and kept withheld pages out. These facts come from `crates/logseq-gardener-sdk/src/site/render.rs` (`link` around line 1011, `embed` around line 874) and the whole-garden build.
		- **Block references** rendered `<a href="<page>#block-<uuid>">` with the target block's text as plain text: markup, brackets and task checkbox were gone (`Claude/Code` showed `figure out Claude/Code/Bug/Failed to save memory: … #`). Links took the default link style; there was no underline style and no hover preview. An unavailable target showed `((uuid))` as plain text with a diagnostic.
		- **Embeds** rendered `<section class="transclusion">` around a bare `ul` of `li` items: no bullets, no collapse, no header for page embeds, no background or shading. The page had no CSS for `.transclusion`.
		- **Unavailable targets** showed the macro source, `{{embed …}}`, as code with a diagnostic, where Logseq shows nothing (block) or an empty well (page).
		- **Limits:** a cycle check over the whole embed chain, a depth limit, 16 active embeds and a total-work budget per page, as the selected content transclusion specification required. Withheld pages never appeared.
	- ## Fixture pages
		- | Page | Why |
		  | --- | --- |
		  | `Claude/Code` | Block references to `DONE` tasks with page references inside |
		  | `Spotify/Tool/TUI` | A block embed of a heading block with links and children |
		  | `AI/Coding/History` | Page embeds of pages that embed pages: two levels of shading |
		  | `Py/Zen of Python` | Many page embeds on one page |
		  | `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` | References to quote blocks, which Logseq shows as the quote |
		  | `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` | 60 block references and 6 block embeds on one page |
		- `AI/Coding/History` and `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` are in the `parity:perf` fixture list, which measures them on the whole encode garden.
