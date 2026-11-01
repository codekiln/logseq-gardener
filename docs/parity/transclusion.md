# Block references and embeds

Logseq shows a block reference as the referenced block's first line, with its inline markup and task checkbox, above a dashed blue underline. It shows embeds as shaded, rounded wells that hold a full outline with bullets; a page embed adds a header with a page icon and a bold link to the page, and each nested embed gets a darker shade. `lsg` already resolves the same targets and keeps withheld pages out, but it shows a block reference as a plain link with the block's text flattened (checkbox and markup lost), and an embed as an unstyled list with no bullets, no header and no shading.

Almost all of the gap is markup and CSS in the publisher. Resolving a block reference's display content (first line, markers, markup, with properties removed) is graph work that belongs in the SDK, so the CLI and the language server can show the same text. Embeds copy whole outlines into a page, so the page-size budgets need a check on the pages that embed the most.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `7736d6f` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden from `54a2b52` (`mise run parity:build`).

## How often the garden uses them

| Form | Uses | Files |
| --- | --- | --- |
| Block reference `((uuid))` | 506 | 164 |
| Labelled block reference `[label](((uuid)))` | 0 | 0 |
| Block embed `{{embed ((uuid))}}` | 164 | 73 |
| Page embed `{{embed [[Page]]}}` | 803 | 256 |

In the `lsg` build, 74 block references and 7 embeds fall back with a diagnostic. 13 of the 19 distinct unresolved references point at an `id::` that exists nowhere in the garden. The other 6 point at bullets whose content is a quote (`- > …` with `id::` on the next line); `lsg` registers no target for them. Logseq resolves those references and shows the quote, because a reference shows a block's body when its title is empty.

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where the checkout's CSS differs; the embed wells in particular are styled differently live than in the checkout's `block.css`.

### Block references

```html
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

### Block embeds

```html
<div class="color-level embed-block bg-base-2" style="z-index:2">
  <div class="px-3 pt-1 pb-2">
    <div class="blocks-container flex-1">…the block and its children, with bullets…</div>
  </div>
</div>
```

- Built by `block-embed` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L798-L814)). The embedded block and all its children render as a normal outline with bullets and collapse arrows; each block's own `collapsed::` sets its initial state ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2829-L2834)). The embedded root hides its reference count.
- **Style, live** on `Spotify/Tool/TUI`: background `rgb(248,249,250)`, 8px radius, 8px margin, `12px 8px` padding; the inner `div` adds `4px 12px 8px`. There is no header.
- An unknown uuid or an argument that is not a reference renders nothing ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1271-L1289)).

### Page embeds

```html
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
- **Nested embeds** are wells inside wells. Live, the second level is `rgb(233,236,239)`. The checkout's CSS steps through `--color-level-2` to `--color-level-6` and then repeats levels 4, 5 and 6 ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L481-L527)); `AI/Coding/History` nests only two levels, so the deeper shades were not measured. Dark values come from `--color-level-*` in [`vars-classic.css`](https://github.com/logseq/og/blob/6e7afa8eb/packages/ui/src/vars-classic.css#L106-L173) and were not measured.
- **Missing page:** the well and header render with an empty body. **The page itself or the enclosing embed:** only the header renders ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L826-L830)).

### Depth and cycles

References and embeds share one depth counter with a limit of 5 ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L96)). Past it, Logseq shows `p.warning` reading `Embed depth is too deep` or `Block ref nesting is too deep` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1142-L1143), [L1274-L1275](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1274-L1275)). The only cycle check is the page-embed self check above; other cycles stop at the depth limit.

### Where the reference boards differ

[lsg/Ref/Logseq Published Site/Transclusion](../pages/lsg___Ref___Logseq%20Published%20Site___Transclusion.md) gives the reference text the link colour, mixes block and page embed styles into one `.embed` rule with a 4px radius, and draws only a header for any embed already in the chain. The live build shows body-coloured reference text, 8px radius on both embed kinds, and the narrower cycle check above. This analysis follows the live build.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/render.rs` (`link` around line 1011, `embed` around line 874) and the whole-garden build:

- **Block references** render `<a href="<page>#block-<uuid>">` with the target block's text as plain text: markup, brackets and task checkbox are gone (`Claude/Code` shows `figure out Claude/Code/Bug/Failed to save memory: … #`). Links take the default link style; there is no underline style, no hover preview. An unavailable target shows `((uuid))` as plain text with a diagnostic.
- **Embeds** render `<section class="transclusion">` around a bare `ul` of `li` items: no bullets, no collapse, no header for page embeds, no background or shading. The page has no CSS for `.transclusion`.
- **Unavailable targets** show the macro source, `{{embed …}}`, as code with a diagnostic, where Logseq shows nothing (block) or an empty well (page).
- **Limits:** a cycle check over the whole embed chain, a depth limit, 16 active embeds and a total-work budget per page, as the [selected content transclusion spec](../../openspec/specs/selected-content-transclusion/spec.md) requires. Withheld pages never appear.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Reference text.** A block reference shows the target's first line with its inline markup and task checkbox, properties removed, body and children left out. Check: on `Claude/Code` the first reference shows a checked checkbox and `[[Claude/Code/Bug/Failed to save memory: …]]` as a page reference with brackets.
- 2. **Reference style.** `span.block-ref` in the body colour with a `0.5px dashed` bottom border in the link colour, `2px 0` padding and `cursor: alias`, in both themes. Check: computed values on `Claude/Code` match the list above.
- 3. **Reference destination.** A click goes to the referenced block. Logseq opens the block as its own page; `lsg` links to the block's anchor on its page, which already exists. Check: the link reaches the block on `Claude/Code`.
- 4. **Invalid reference.** An unknown uuid shows `span.warning` titled `Block ref invalid` with `((uuid))`, and keeps its diagnostic. Check: a test garden reference to a missing uuid.
- 5. **Quote targets.** A bullet whose content is a quote and that carries `id::` is a reference target; a reference to it shows the quote, as Logseq does. Check: the two references on `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` resolve with no diagnostic.
- 6. **Embed wells.** Block and page embeds render Logseq's well: background, 8px radius, 8px margin, `12px 8px` padding, in both themes. Check: computed values on `Spotify/Tool/TUI` and `AI/Coding/History`.
- 7. **Embedded outlines.** Embedded blocks render as a normal outline with bullets and collapse arrows, honouring each block's `collapsed::`, with unique ids. Check: the `Spotify/Tool/TUI` embed shows bullets; collapsing an embedded block works.
- 8. **Page embed header.** A header with the page icon and a bold 19.2px link to the embedded page. Check: `AI/Coding/History` shows one header per embed, linking to each phase page.
- 9. **Nested shading.** Each nested level takes Logseq's next shade, repeating levels 4 to 6 past the sixth. Check: the second level on `AI/Coding/History` is `rgb(233,236,239)`.
- 10. **Unavailable embeds.** An unknown block uuid renders nothing; a page with no published file renders the well and header with no body; a self-embed renders only the header. Each keeps its diagnostic. Check: test garden pages for each case, plus the 7 fallbacks in the whole-garden build.
- 11. **Depth warnings.** Past the depth limit, `p.warning` with Logseq's two messages. Check: a test garden chain of seven nested embeds.
- 12. **Performance.** Embeds copy outlines into the page, and the wells and bullets add markup. Check: `parity:budgets` passes, the largest embedding page in the garden stays within the first-paint budget, and no script is added beyond the existing outline script.

## Where the work belongs

- **SDK:** resolving a block reference to its display content (first line, task marker, inline nodes, properties removed), and resolving an embed to its target outline with the depth and cycle rules, so the CLI and the language server show and follow the same references. The existing target index in the publisher moves behind an SDK operation.
- **Publisher:** the reference, well, header and warning markup, the shades and the styles, and reuse of the outline renderer and its collapse script inside embeds.

## Fixture pages

| Page | Why |
| --- | --- |
| `Claude/Code` | Block references to `DONE` tasks with page references inside |
| `Spotify/Tool/TUI` | A block embed of a heading block with links and children |
| `AI/Coding/History` | Page embeds of pages that embed pages: two levels of shading |
| `Py/Zen of Python` | Many page embeds on one page |
| `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` | References to quote blocks, which Logseq shows as the quote |
| `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` | 60 block references and 6 block embeds on one page |

## Out of scope

- Hover previews of referenced blocks and shift-click into the right sidebar, which belong with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).
- Opening a referenced block as its own page (`#/page/<uuid>`); `lsg` links to the block's anchor, and block zoom from #111 gives the same focused view.
