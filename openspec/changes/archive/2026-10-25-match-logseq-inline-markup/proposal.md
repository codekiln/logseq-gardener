## Why

Logseq's published site marks up inline text so readers can tell a page reference from a tag, an outside link, code and an unsupported macro at a glance. `lsg` publishes the same text, but tags and unknown macros come out as plain text or code, page references lose their `[[` `]]` brackets, outside links look like page links and open in the same tab, and images ignore their authored size ([inline markup analysis](../../../../docs/parity/inline-markup.md), [Issue #112 — Match Logseq's inline markup: links, tags, emphasis, images and macros](https://github.com/codekiln/logseq-gardener/issues/112)).

## What Changes

- The SDK reads `:ui/show-brackets?` from `logseq/config.edn` (Logseq's default is to show brackets) and gives the page name a tag points at, for `#tag` and `#[[multi word]]`, so the CLI, the language server and the publisher resolve tags the same way as page references, including aliases and case.
- Page references, including labelled ones such as `[label]([[Page]])`, render inside `span.page-reference` with faint `[[` and `]]` brackets, unless the garden turns brackets off. The link keeps its `href`, uses Logseq's link colour and weight, and has no underline.
- Tags render as `a.tag` links to the tag's page, showing `#` and the tag text, at Logseq's size, colour, opacity and padding.
- Outside links render `a.external-link`, open in a new tab with `rel="noopener noreferrer"`, and get Logseq's colour and 1px bottom border.
- Bold, italic, strikethrough and highlight keep their elements and get Logseq's weight and highlight colours; inline code gets Logseq's font, size, background, padding and radius.
- Images with a `{:height …, :width …}` map get those `width` and `height` attributes, load lazily, keep `alt`, add a `title`, and never grow wider than the content column.
- A macro with no handler renders Logseq's warning chip, `span.warning`, titled `Unsupported macro name: <name>`. Macros that other parity issues own keep their current fallback until those issues land.
- All of this is markup and one small inline stylesheet; no inline feature adds JavaScript or graph data to the browser. New budgets cap the stylesheet's size and the growth of the largest fixture page.

## Capabilities

### New Capabilities

- `inline-references`: the SDK operations that read the garden's bracket setting and name the page a tag points at.
- `published-inline-markup`: the published markup and styles for page references, tags, outside links, emphasis, inline code, sized images and unknown macros, and their budgets.

### Modified Capabilities

None. The publishing requirement that unsupported macros keep visible fallback content and diagnostics still holds; the chip is that fallback.

## Impact

- SDK: `garden_settings.rs` gains `show_brackets`; a new `inline_references` module gives a tag's page name; `site/search.rs` uses it so search text and links agree.
- Publisher: `site/render.rs` (page references, tags, outside links, images, macro chip), a new `site/inline.css` inlined with the other page styles in `site/mod.rs`, and colour tokens in `site/theme.css` for both themes.
- Parity harness: `scripts/parity/budgets.toml` gains an `[inline]` table checked by `parity:budgets`.
- Gitpa receives the same markup in swapped `<main>` content; nothing it runs changes.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
