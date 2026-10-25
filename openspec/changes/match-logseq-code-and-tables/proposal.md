## Why

Logseq's published site shows fenced code with the outline's indentation removed, a language label, line numbers and token colours, and shows Markdown tables across the content column with striped rows. `lsg` publishes the same code as a plain block that keeps the outline's indentation on every line (3,393 of 3,408 code blocks in the encode garden), with no label, colours or line numbers, and publishes tables at their natural width with no striping ([code and tables analysis](../../../docs/parity/code-and-tables.md), [Issue #113 — Highlight fenced code like Logseq and render Markdown tables](https://github.com/codekiln/logseq-gardener/issues/113)).

## What Changes

- The SDK gives each code block its text with the owning bullet's indentation removed, and looks up the fence language the way Logseq does: a language Logseq highlights gets Logseq's mode name, and any other language, including `py`, `rs`, `text`, `mermaid` and unknown names, gets none. The CLI, the language server and the publisher see the same code.
- Fenced code renders in Logseq's structure: `div.cp__fenced-code-block` with the language in `data-lang`, a lower-case language label at the top right, a line-number gutter, and token spans with Logseq's CodeMirror class names and colours in both themes. The tokens are computed at build time; no script is added.
- Code blocks use Logseq's monospace stack, 12px text, 16px lines, background, 2px radius and spacing, and scroll sideways instead of wrapping. Copying a block copies only the code, without line numbers.
- `#+BEGIN_EXAMPLE` blocks render as a wrapping `pre.pre-wrap-white-space` on the secondary background, separate from fenced code.
- Markdown tables render inside `div.table-wrapper` with `thead` and `tbody`, fill the content column, and use Logseq's header style, cell padding and row striping in both themes. The wrapper scrolls sideways when a table is wider than the column. Alignment markers have no effect, as in Logseq.
- New budgets cap the code and table styles every page carries, the growth of the largest code page and the whole-garden build time.

## Capabilities

### New Capabilities

- `code-blocks`: the SDK operations that give a code block's text without the outline's indentation and map a fence language to the mode Logseq highlights it with.
- `published-code-and-tables`: the published markup and styles for fenced code, example blocks and Markdown tables, and their budgets.

### Modified Capabilities

None.

## Impact

- SDK: a new `code_blocks` module; `outline.rs` gives code blocks their de-indented text.
- Publisher: `site/render.rs` (code, example and table markup), a new `site/code.rs` for build-time highlighting, a new `site/code.css` inlined with the other page styles in `site/mod.rs`, and colour tokens in `site/theme.css` for both themes.
- Dependencies: the SDK gains `syntect` (pure-Rust regex engine) and `two-face` for the grammars of the languages the garden uses.
- Parity harness: `scripts/parity/budgets.toml` gains a `[code]` table checked by `parity:budgets`; `parity:perf` gains a build-time row.
- Gitpa receives the same markup in swapped `<main>` content; nothing it runs changes.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
