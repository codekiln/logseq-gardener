## Context

The [transclusion analysis](../../../../docs/parity/transclusion.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `7736d6f`) with an `lsg` build from `54a2b52`. The garden has 506 block references, 164 block embeds and 803 page embeds. In `crates/logseq-gardener-sdk/src/site/render.rs`, `outline_targets` builds the reference index and flattens each target's label to plain text; `link` writes that text inside `<a href="<page>#block-<uuid>">`; `embed` wraps the target blocks in an unstyled `<section class="transclusion">`. Unavailable targets show their source as code. No CSS styles references or embeds.

Logseq computes all of this in the browser from the whole graph. `lsg` resolves references and embeds during the build, so a page arrives with its references and embeds already in its HTML.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's block references and embeds: the first line with its markup and checkbox under a dashed underline, and shaded wells with bullets, collapse and page headers, in both themes.
- The CLI and the language server read the same reference content and embed outlines as the publisher.
- Pages that embed the most stay within the first-paint budget, and transclusion adds no script.

**Non-Goals:**

- Hover previews of referenced blocks and shift-click into the right sidebar, which belong with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).

## Decisions

**Where the work lives.** The SDK gains a `transclusion` module:

- `ReferenceTargets`: the uuid index over selected visible pages, moved out of `site/render.rs`. It registers outline bullets with an `id::`, `custom-id::` or `custom_id::` property, including bullets whose content is a quote, and drops a uuid with more than one owner.
- `reference_content(target) -> ReferenceContent { marker, heading, inline, body }`: the first line's inline nodes with properties removed. When the first line is empty, `body` holds the block's other blocks without properties, which Logseq shows instead; a quote bullet gives its quote.
- `embed_outline(request, chain) -> EmbedOutcome`: `Outline { title, route, blocks }`, `Unavailable`, `SelfEmbed { title, route }` or `TooDeep(kind)`, where `chain` is the stack of enclosing references and embeds. The nesting limit is 5 levels shared by references and embeds, as in Logseq's `block.cljs`.

The publisher renders each outcome and keeps its existing total-work and expansion budgets as a safety net behind the SDK's limit.

**Reference markup.**

```html
<a class="block-ref-link" href="<page>#block-<uuid>"><span class="block-ref">…first line…</span></a>
```

The whole first line goes through the same inline renderer as any block, so references carry the inline markup from [Issue #112](https://github.com/codekiln/logseq-gardener/issues/112). Inside the reference's link, page references and tags render as spans with their usual classes, because links cannot nest; other links show their label, and an embed in the first line is left out. A task marker renders the outline's checkbox markup. A heading renders as `span.block-ref-heading` with `role="heading"`, its `aria-level` and `font-size: 1rem`, because an `h1`–`h6` would close an enclosing paragraph. Logseq's wrapper `div`s are left out for the same reason. An invalid reference renders `<span class="warning" title="Block ref invalid">((uuid))</span>`; a reference past the nesting limit renders `<span class="warning">Block ref nesting is too deep</span>`.

**Destination.** A reference links to the block's anchor on its page, which already exists, and block zoom from #111 gives Logseq's focused view. Logseq's `#/page/<uuid>` route has no static equivalent.

**Embed markup.**

```html
<div class="embed-page" data-level="1">
  <section class="embed-header"><svg class="page-icon">…</svg><a href="…">AI/Coding/History/Phase/1 - Primitive</a></section>
  <div class="blocks-container">…outline…</div>
</div>
```

A block embed uses `div.embed-block` with an inner `div` and no header. `data-level` carries the nesting level; the stylesheet maps levels 2 to 5 to Logseq's shades with `[data-level]` selectors. The nesting limit stops at level 5, so Logseq's sixth shade and its repeats past the sixth level never show. Embedded outlines go through the existing outline renderer with `embedded` set, which leaves out block IDs, and with bullets, so the outline script handles collapse inside wells with no change. The page icon is Logseq's page glyph as one inline SVG of about 330 bytes.

**Styles.** A new `site/transclusion.css`, inlined into every page after the code styles, styles references, wells, headers and shades; the existing `.warning` rule from the inline styles covers the warnings. Theme tokens `--lsg-embed-1` through `--lsg-embed-5` join `site/theme.css` for both themes, measured on the `gh-pages` build with Logseq's own theme toggle. The reference underline and the page embed link use the existing `--lsg-ref` token: in the light theme it equals Logseq's `rgb(16,107,163)`; in the dark theme Logseq's `rgb(163,206,241)` comes from the theme the garden's `logseq/custom.css` imports, which `lsg` loads only through `--stylesheet`.

**Budgets.** Measured by the [comparison harness](../../../specs/parity-comparison-harness/spec.md):

| Measure | Budget |
| --- | --- |
| Transclusion styles on every page, gzip | 1 KB |
| Largest embedding fixture page, gzip | within the existing 100 KB first-paint budget |
| Script added | none |
| Peak phone heap on the embedding fixtures | within the existing 10 MB |
| Search latency per keystroke | unchanged; the search corpus keeps one entry per block, and embedded copies add none |

`scripts/parity/budgets.toml` gains a `[transclusion]` table with `styles_gzip_kb = 1`. `parity:budgets` runs in CI on the fixture garden, which has no copy of the encode garden, so it checks the style size and that a fixture page (`Tools/Embeds`) holds embed wells under the existing first-paint budget. `AI/Coding/History` and `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` join the `parity:perf` fixture list, which measures them on the whole encode garden.

## Risks / Trade-offs

- [Page embeds copy whole outlines, so the pages that embed the most grow] → The first-paint budget on the embedding fixtures fails CI before a page exceeds it; Apply records the largest pages' sizes before and after in `verification.md`.
- [A limit of 5 is lower than the current 16 and could cut off deep embed chains that render today] → Logseq stops at the same level, so the published site shows the same content as Logseq's; Apply lists any page in the garden that reaches the limit.
- [Dark shades are read from source, not measured] → Apply checks them against the `gh-pages` build in dark mode before merging.

## Resolved Questions

### 1 - Where does a block reference link to?

Decided by the agent without codekiln: the block's anchor on its page, as today. Logseq opens `#/page/<uuid>`, which needs the client-side router; the anchor plus block zoom shows the same block without loading the graph.

### 2 - What does an unavailable embed show?

Decided by the agent without codekiln: what Logseq shows. Nothing for a block embed, the well and header with no body for a page embed, and only the header for a self embed, each with its build diagnostic. The diagnostic still tells the author; the reader sees Logseq's site.

### 3 - Does a page embed of an unpublished page link to it in the header?

Decided by the agent without codekiln: no. The header shows the name as text. Logseq links to an empty page, which a static site does not have, and a link would lead to a missing page.

### 4 - Which nesting limit applies?

Decided by the agent without codekiln: Logseq's 5 levels shared by references and embeds, with its two warning messages. The publisher keeps its total-work and expansion budgets as a safety net for repeated embeds.

### 5 - Are quote bullets with `id::` reference targets?

Decided by the agent without codekiln: yes, with empty display content, because Logseq resolves them and shows them empty. This removes 6 fallbacks from the encode garden build.

### 6 - Does the reference keep Logseq's wrapper `div`s?

Decided by the agent without codekiln: no. A `div` inside an inline link is invalid HTML; `span.block-ref` with the same styles gives the same look.

### 7 - Are the transclusion styles on every page or only pages with references or embeds?

Decided by the agent without codekiln: every page, as with the outline, inline, and code styles. One inlined stylesheet keeps rendering simple, and 1 KB gzip is within the per-page budgets.

### 8 - Does a heading in a block reference use an `h1`–`h6` element?

Decided by the agent without codekiln: no. It renders as `span.block-ref-heading` with `role="heading"` and the heading's `aria-level`, at `1rem`. A reference can sit inside a paragraph, and an `h1`–`h6` there closes the paragraph and breaks the line apart; the span keeps Logseq's look and still announces a heading.

### 9 - Which element shows `Block ref nesting is too deep`?

Decided by the agent without codekiln: `span.warning`. A reference past the limit sits inside the enclosing reference's link, where Logseq's `p.warning` would close the surrounding paragraph. Embeds keep `p.warning` for `Embed depth is too deep`, because embeds already render as flow content.

### 10 - How long is a reference's text?

Decided by the agent without codekiln: the whole first line, as Logseq shows it. The old 160-character cut-off with an ellipsis is gone.

### 11 - What happens to an embed inside a referenced block's first line?

Decided by the agent without codekiln: it is left out, with a diagnostic. Logseq draws the embed inside the reference, but an outline cannot sit inside the reference's link. The garden has no such case.

### 12 - Where do the embedding pages' first-paint budgets run?

Decided by the agent without codekiln: CI runs `parity:budgets` on the fixture garden, so it checks a new fixture page with embeds. The encode garden's embedding pages are measured by `parity:perf` and recorded in `verification.md` and `docs/parity/performance.md`. CI has no copy of the encode garden.

### 13 - Does the stylesheet carry Logseq's sixth shade and its repeats?

Decided by the agent without codekiln: no. The shared nesting limit of 5 means a well never reaches level 6, so those rules would never apply.

### 14 - What does a reference to a quote bullet show? (revisits 5)

Decided by the agent without codekiln: the quote. Answer 5 said Logseq shows these references empty; on the `gh-pages` build both references on `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` show the quoted text, because `block-content-inner` in `block.cljs` draws a referenced block's body when its title is empty. Quote bullets stay reference targets.

## Open Questions

None.
