## Why

Logseq's published site shows a block reference as the referenced block's first line, with its markup and task checkbox, above a dashed underline, and shows embeds as shaded, rounded wells holding a full outline with bullets; a page embed adds a header that links to the page. `lsg` resolves the same targets but shows a reference as a plain link with the text flattened, and an embed as an unstyled list with no bullets, header or shading ([transclusion analysis](../../../docs/parity/transclusion.md), [Issue #114 — Match Logseq's block references and block and page embeds](https://github.com/codekiln/logseq-gardener/issues/114)).

## What Changes

- The SDK resolves a block reference to its display content: the target's first line as inline nodes, its task marker and heading level, with properties, body and children left out. It also gives an embed's target outline and applies Logseq's shared nesting limit of 5 for references and embeds. The CLI, the language server and the publisher read references the same way.
- A bullet whose content is a quote and that carries `id::` becomes a reference target, as in Logseq. A reference to it shows the quote, because Logseq shows a block's body when its first line is empty.
- A block reference renders as `span.block-ref` inside its link: the first line with its markup and checkbox, the body text colour, a `0.5px` dashed underline in the link colour and `cursor: alias`. Clicking it still opens the block's anchor on its page.
- An unknown or unavailable block reference renders as `span.warning` titled `Block ref invalid`, showing `((uuid))`, and keeps its diagnostic.
- Block and page embeds render as Logseq's wells: background, 8px radius, 8px margin and padding, holding the target as a normal outline with bullets and collapse arrows that honour each block's `collapsed::`. A page embed adds a header with the page icon and a bold link to the page. Each nested well takes Logseq's next shade, in both themes.
- An unavailable block embed renders nothing; an unavailable page embed renders the well and header with no body; a page embedding itself renders only the header. Each keeps its diagnostic. Past the nesting limit, Logseq's two warnings appear: `Embed depth is too deep` and `Block ref nesting is too deep`.
- New budgets cap the transclusion styles every page carries and the size of the pages that embed the most.
- **BREAKING** for readers of `lsg` output only: an unavailable embed no longer shows its `{{embed …}}` source as code.

## Capabilities

### New Capabilities

- `transclusion-content`: the SDK operations that resolve a block reference to its display content, resolve an embed to its target outline, and apply the shared nesting limit.
- `published-transclusion`: the published markup, styles and budgets for block references, block embeds and page embeds.

### Modified Capabilities

- `selected-block-navigation`: an unlabelled reference shows the target's rendered first line instead of flattened text; quote bullets with an `id::` are targets; an unavailable reference renders as Logseq's warning span.
- `selected-content-transclusion`: unavailable embeds render as Logseq does instead of showing the macro source, and the nesting limit becomes Logseq's 5 with its warnings.

## Impact

- SDK: a new `transclusion` module; the reference target index moves out of `site/render.rs` and includes quote bullets.
- Publisher: `site/render.rs` (reference, well, header and warning markup; embedded outlines through the outline renderer), a new `site/transclusion.css` inlined with the other page styles in `site/mod.rs`, shade and reference tokens in `site/theme.css` for both themes, and the page icon as inline SVG.
- Parity harness: `scripts/parity/budgets.toml` gains a `[transclusion]` table checked by `parity:budgets`.
- Gitpa receives the same markup in swapped `<main>` content; the outline script it already loads handles collapse inside wells.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
