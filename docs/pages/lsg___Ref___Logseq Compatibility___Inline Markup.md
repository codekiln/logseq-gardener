tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Inline Syntax]], [[lsg/Ref/Rendering and Assets]]

- # Inline Markup Compatibility
	- `lsg` publishes Logseq's inline markup: page references between faint `[[` `]]` brackets in Logseq's link colour, tags as small accent-coloured `#tag` links, outside links that open a new tab with a 1px underline, bold at weight 600, Logseq's highlight and grey inline code chips, images at their authored size, and a warning chip for an unknown macro. Its page references keep real links and draw their brackets with the stylesheet, so copied text leaves the brackets out; references and tags to pages it does not publish show their text without a link; and it has no hover previews, footnotes or math rendering.
	  id:: 1040dd0c-7481-4d45-a1dc-708fb08fee9d
	- Measured on 2026-10-25 against the `gh-pages` build of garden commit `8bc3391` (served locally, light theme) and an `lsg` build of the whole garden from `581a7a4` (`mise run parity:build`). The OpenSpec change [match-logseq-inline-markup](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-25-match-logseq-inline-markup) then built the markup, and [published-inline-markup](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-inline-markup/spec.md) holds its requirements; [email-autolinks](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/email-autolinks/spec.md) covers email links.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb`, the file-graph continuation of 0.10 (`frontend` = `src/main/frontend`). The published site renders inline markup exactly as the app does: publishing never sets `:html-export?`, so the app-only branches for brackets and inline HTML apply. The inline renderer is `inline` in [`components/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1564-L1663). Measured live values are given first and win where the checkout's CSS differs.
		- ### Page references
			- ```html
			  <span class="page-reference" data-ref="tmux/Config">
			    <span class="text-gray-500 bracket">[[</span>
			    <a class="page-ref" data-ref="tmux/config">tmux/Config</a>
			    <span class="text-gray-500 bracket">]]</span>
			  </span>
			  ```
			- Built by `page-reference`, `page-cp` and `page-inner` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L533-L780)). The link has no `href`; a click handler routes it. On desktop the link is wrapped in a hover-preview trigger.
			- **Brackets** show unless `:ui/show-brackets?` is `false` ([`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L652-L654)). The encode garden leaves the key commented out, so brackets show. Live: grey `rgb(107,114,128)` at opacity 0.3, weight 400.
			- **Link:** live colour `rgb(16,107,163)`, weight 400, no underline, same size as the text. `.page-reference` has a 3px radius and a faint background on hover ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L580-L645)).
			- **Text:** the text as typed, so `[[Foo BAR]]` shows `Foo BAR`; an all-lower-case reference shows the page's original name. Namespaced references show the whole path, `tmux/Config`.
			- **Pages without files** render the same link. Every referenced page exists in Logseq's graph, so no reference is ever unlinked.
			- **Journal references** such as `[[2026-07-04 Sat]]` are ordinary page references; the text is not reformatted.
			- **Labelled references** `[label]([[Page]])` show the label inside the same brackets. Live, `LangSmith/26/09/29 Tue - Deep Agents/1045 Engine Workshop/Take-Aways` shows `[[LangSmith roadshow, Boston]]` linking to `LangSmith/26/09/29 Tue - Deep Agents`. The garden has 49.
		- ### Tags
			- ```html
			  <a class="tag" data-ref="keyshort">#Keyshort</a>
			  ```
			- `#tag` and `#[[multi word]]` render as one link with no brackets; `#[[multi word]]` shows `#multi word` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1575-L1578)).
			- Live on `1Password/Quick Access`: colour `rgb(52,81,178)`, 0.96em (15.36px in 16px text), opacity 0.8, padding `0 4px`, radius 4px. Hover sets opacity 1 and underlines ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L484-L499)).
			- A tag targets its page like a page reference. The garden has about 2,170 tags outside code blocks.
		- ### External links and email
			- `[text](https://…)` and bare URLs render `<a class="external-link" href="…" target="_blank">` with no `rel` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1226-L1234)).
			- Live: colour `rgb(0,65,116)`, weight 300 in an `h1`, no text decoration, `border-bottom: 1px solid` in the link colour ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L202-L209)).
			- Email autolinks render `<a href="mailto:…">` with no class ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1595-L1598)).
		- ### Emphasis and code
			- Bold `<b>` (live weight 600), italic `<i>`, strikethrough `<del>`, highlight `<mark>`, superscript and subscript `<sup>`/`<sub>` (`emphasis-cp`, [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1544-L1573)). `==text==` produces `<mark>`; the garden has 155, and no `^^text^^`.
			- **Highlight:** the build's stylesheet sets `mark { background: #fef3ac; color: #262626; padding: 2px 4px; border-radius: 3px }`. In the in-app browser the computed style was instead a 4px amber (`rgb(255,198,0)`) underline with no fill. Playwright's Chromium shows the same underline on the `gh-pages` build; the rule comes from `pengx17/logseq-dev-theme`, which the garden's `logseq/custom.css` imports.
			- **Inline code:** `<code>` with no class. Live: `"Fira Code", monospace`, 0.8em (12.8px in 16px text), background `rgb(241,243,245)`, text colour unchanged, padding `3px 5px`, radius 4px ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L597-L629)).
		- ### Images
			- ```html
			  <div class="resize image-resize" style="width: 231px">
			    <div class="asset-container">
			      <img loading="lazy" referrerpolicy="no-referrer" src="…" title="1Password connect diagram" height="265" width="231" class="rounded-sm relative">
			      …copy and maximize buttons, shown on hover…
			    </div>
			  </div>
			  ```
			- `resizable-image` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L265-L459)) copies the `{:height 265, :width 231}` map onto the `img` as attributes and the width onto the wrapper. Live on `1Password/Dev/Connect Server`.
			- The alt text goes into `title`; there is no `alt` attribute.
			- `max-width: 100%`, radius 3px, lazy loading. Relative `../assets/x.png` resolves to `assets/x.png` in the build. The garden has 1,191 images, 102 with a size map.
		- ### Macros
			- **Unknown macro:** `<div class="macro" data-macro-name="foo"><span class="warning" title="Unsupported macro name: foo">{{foo a, b}}</span></div>`, arguments joined by `, ` (`render-macro`, [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L855-L969)). The chip uses `--ls-warning-background-color` and `--ls-warning-text-color`, padding `.1em .4em` ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L434-L442)). `{{pdf}}` has no handler and falls into this case.
			- **Custom macros** from `:macros` in `config.edn` or a page's `macros::` property expand in place. The encode garden's `:macros` is `{}`.
			- **`{{renderer …}}`** renders nothing on the published site (it needs plugins).
			- `{{embed}}` is described in [[lsg/Ref/Logseq Compatibility/Transclusion]], `{{query}}` in [[lsg/Ref/Logseq Compatibility/Queries]], `{{video}}`, `{{youtube}}` and `{{youtube-timestamp}}` in [[lsg/Ref/Logseq Compatibility/Media Embeds]], and `{{cards}}` and `{{cloze}}` in [[lsg/Ref/Logseq Compatibility/Flashcards]]. The garden uses `{{namespace}}` 8 times and `{{tweet}}` 8 times; `{{tweet}}` loads Twitter's script.
	- ## What lsg publishes
		- Each page reference, plain or labelled, is a link to the referenced page inside `span.page-reference`. The link text is the reference as typed, or its label, in Logseq's page-reference colour for the active theme, at normal weight with no underline; the reference shows a faint background with a 3px radius on hover. On `tmux`, `[[tmux/Config]]` shows grey brackets at opacity 0.3 around a link to `tmux/Config`, and a labelled reference shows `[[LangSmith roadshow, Boston]]` with the label linked. [[lsg/Ref/Rendering and Assets]] describes the rendering for readers.
		- The stylesheet draws the `[[` and `]]` brackets on `span.page-reference`, so they add no bytes per reference. A garden with `:ui/show-brackets? false` gets the class `no-brackets` on each reference and no brackets; only the boolean `false` turns them off.
		- A reference that resolves to no published page, or to several, keeps its brackets and text without a link, and the build reports a diagnostic.
		- `#tag` and `#[[multi word]]` render as `a.tag` linking to the tag's page, with the text `#` and the tag as typed and no brackets: `#Keyshort` on `1Password/Quick Access` links to the `Keyshort` page. Tags use Logseq's tag colour for the active theme, 0.9em, opacity 0.8, `0 2px` padding and a 4px radius, with full opacity and an underline on hover. A tag whose page is unpublished or ambiguous renders as `span.tag` with the same look and no link, with a diagnostic. A tag inside a link label stays text.
		- Links to `http` and `https` addresses render as `a.external-link` with `target="_blank"` and `rel="noopener noreferrer"`, in Logseq's outside-link colour, with no text decoration and a 1px solid bottom border in the link colour. `mailto:` links and email autolinks keep plain markup, as Logseq's do.
		- Bold renders as `<strong>` at weight 600; italic, strikethrough and highlight stay `<em>`, `<del>` and `<mark>`. A highlight has Logseq's built-in fill, `#fef3ac`, with `#262626` text, `2px 4px` padding and a 3px radius.
		- Inline code uses `"Fira Code", monospace` at 0.8em, Logseq's code background for the active theme, the surrounding text colour, `3px 5px` padding and a 4px radius. The site ships no font file, so readers without Fira Code see their own monospace font.
		- An image with a `{:width …, :height …}` map gets each positive integer value as its `width` or `height` attribute; other keys and values are ignored. `1Password/Dev/Connect Server`'s diagram renders at 231 by 265. Every image loads lazily, keeps its `alt` text, carries the same text as `title`, has a 3px radius, and scales down to the width of its container, keeping its aspect ratio.
		- A macro with no handler renders `span.warning` titled `Unsupported macro name: <name>`, with the text `{{<name> <arguments joined by ", ">}}`, in Logseq's warning colours with `.1em .4em` padding, and the build reports a diagnostic. `{{renderer …}}` renders nothing and reports a diagnostic. `{{video}}`, `{{youtube}}`, `{{vimeo}}`, `{{bilibili}}` and `{{youtube-timestamp}}` render as media embeds, `{{query}}` as a query, `{{cards}}` and `{{cloze}}` as flashcards, and `{{embed}}` as an embed. `{{namespace}}` and `{{tweet}}` render as literal code with a diagnostic.
		- The same markup and styles apply wherever inline text renders, including page properties, the journals feed and Linked References.
		- Dark-theme colours are Logseq's built-in dark variables from the `gh-pages` build's stylesheet: `#8abbbb` for page references, `#849dff` for tags and outside links, `#202425` for inline code, and `#946800` and `#f9e68c` for warnings.
		- A garden's own theme reaches an `lsg` site through `--stylesheet logseq/custom.css`, and its selectors match because `lsg` writes Logseq's class names.
		- The SDK reads `:ui/show-brackets?` into `GardenSettings::show_brackets` and gives a tag's page name with `inline_references::tag_page_name`, which the publisher and the search index resolve with `PageNameIndex::candidates`, as they resolve page references. The publisher renders the markup, the styles in `site/inline.css`, the image attributes and the macro chip.
		- Inline markup adds no JavaScript and no graph data to the browser. The `[inline]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) holds the budgets that `parity:budgets` checks in CI: at most 1 KB gzip of inline styles on each page, and a limit on the growth of the largest fixture page. With the brackets drawn by the stylesheet, the change grew `tmux` by 1,465 bytes gzip.
	- ## Differences from Logseq
		- `lsg` draws the brackets with the stylesheet, so a reader copying text copies no brackets. Logseq writes them as `span.bracket` elements; in `lsg` those elements added 1,132 bytes gzip to `tmux`, which put the page over its 2 KB growth budget.
		- Page reference links carry an `href`, so they work without JavaScript, in hosts such as Gitpa, for crawlers and when opened in a new tab. Logseq's anchors have none and rely on its router.
		- References and tags to pages without a published file show their text without a link. In Logseq every referenced page exists, so no reference is unlinked. `lsg` does not publish pages that exist only because something references them.
		- Outside links carry `rel="noopener noreferrer"`, which stops the opened page from controlling the published one and changes nothing a reader sees. Logseq writes no `rel`.
		- Bold and italic use `<strong>` and `<em>`, which tell screen readers the text is emphasised; Logseq uses `<b>` and `<i>`. The look is the same.
		- Images keep `alt` and add `title`; Logseq writes only `title`. Images and macro chips have no wrapper `div`, since they sit inside paragraphs, where a `div` is invalid HTML.
		- Tags and highlights follow Logseq's own stylesheet. The encode garden's `gh-pages` build shows an amber underline for highlights, from the `pengx17/logseq-dev-theme` theme its `logseq/custom.css` imports, and the live tag measured 0.96em with `0 4px` padding.
		- `lsg` has no hover preview of a referenced page.
		- `{{namespace}}` and `{{tweet}}` render as literal code. Footnote references render as `[name]` followed by the footnote's text, and inline math as literal code, each with a diagnostic.
	- ## Before the change
		- The `lsg` build measured on 2026-10-25, from `581a7a4`, rendered inline markup in `crates/logseq-gardener-sdk/src/site/render.rs`.
		- **Page references:** `<a href="p-….html">tmux/Config</a>` with no brackets and no class, in `--lsg-link` colour. The text was as typed. A reference to a page with no file rendered `<span class="unresolved">1Password/Desktop</span>`; the build reported 18,628 such references, mostly reference-only pages.
		- **Labelled references:** `<a href="…">LangSmith roadshow, Boston</a>`, no brackets.
		- **Tags:** plain text. `Inline::Tag` wrote `#` and the tag's text with no link (`render.rs`, around line 807); `1Password/Quick Access` showed `#Keyshort #card` as text inside its heading.
		- **External links:** `<a href="https://…">` styled like page links, no class, no new tab, no underline.
		- **Emphasis:** `<strong>`, `<em>`, `<del>` and `<mark>` with browser defaults (bold 700, browser yellow highlight).
		- **Inline code:** `<code>` at 0.9em in the browser's monospace font, no background or padding.
		- **Images:** `<img src="…" alt="…">`; the size map was dropped, so `1Password/Dev/Connect Server`'s diagram showed at its natural size.
		- **Unknown and unsupported macros:** printed as `<code>{{name args}}</code>` with a diagnostic (`literal`, `render.rs` line 389). The build reported 4,220 macro diagnostics, almost all `youtube-timestamp` and `embed`. Footnote references, inline math and timestamps used the same fallback.
	- ## Fixture pages
		- | Page | Why |
		  | --- | --- |
		  | `tmux` | Page references, a journal reference, an external heading link; the size budget |
		  | `1Password/Quick Access` | Tags `#Keyshort` and `#card`, inline code, bold, images, a reference to a page without a file |
		  | `1Password/Dev/Connect Server` | Image with `{:height 265, :width 231}`, external links |
		  | `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` | `==highlight==`, tags, inline code, images |
		  | `LangSmith/26/09/29 Tue - Deep Agents/1045 Engine Workshop/Take-Aways` | Labelled page reference |
