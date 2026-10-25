## Context

The [code and tables analysis](../../../docs/parity/code-and-tables.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `d200706`) with an `lsg` build from `e88e16b`. The garden has 1,706 fenced code blocks and 138 Markdown tables on 112 pages. In `crates/logseq-gardener-sdk/src/site/render.rs`, `Block::Src` and `Block::Example` both become `<pre><code>` with the parser's text, which still carries the bullet's continuation indentation, and the parser's `lang` is dropped. `Block::Table` becomes a bare `<table>` of rows.

Logseq highlights code with CodeMirror 5, part of the roughly 35 MB of JavaScript its site loads. `lsg` produces the same look as HTML and CSS during the build, so readers download coloured markup and one small stylesheet instead of an editor.

## Goals / Non-Goals

**Goals:**

- Readers of the encode garden see Logseq's code blocks and tables: code at column 0, the language label, line numbers, Logseq's token colours in both themes, and Logseq's table style.
- The CLI and the language server read the same code text and language mode as the publisher.
- First paint stays independent of garden size and code adds no script.

**Non-Goals:**

- Evaluating `calc` blocks and showing their results, as Logseq does. The garden has 4; they render as plain code for now.
- Wrapping code lines and changing line numbers through `:editor/extra-codemirror-options`, which the encode garden does not set.

## Decisions

**Where the work lives.** The SDK gains a `code_blocks` module:

- `code_text(block) -> String`: the code with the owning bullet's continuation prefix removed. The outline already knows each bullet's indentation; the prefix is that whitespace followed by two spaces. A line shorter than the prefix loses only the whitespace it has. One trailing newline is dropped.
- `code_language(fence: &str) -> CodeLanguage { label: Option<String>, mode: Option<LogseqMode> }`: the label is the fence language lower-cased; the mode comes from a table of the names and aliases `CodeMirror.findModeByName` accepts among the modes Logseq bundles (listed in `extensions/code.cljs` in `logseq/og`), after Logseq's Clojure aliases.

The publisher maps each `LogseqMode` to a grammar, highlights at build time, and renders the markup and styles.

**Highlighter.** `syntect` with its pure-Rust regex engine reads TextMate grammars; `two-face` supplies grammars that `syntect`'s defaults lack (TOML, TypeScript, Dockerfile and others in the analysis's table). The publisher maps TextMate scopes to CodeMirror token classes by prefix, for example `keyword` and `storage` to `cm-keyword`, `string` to `cm-string`, `constant.numeric` to `cm-number`, `entity.name.function` to `cm-def`, `keyword.operator` to `cm-operator`, `entity.name.type` and `support.type` to `cm-type`, `comment` to `cm-comment`, and `variable` to `cm-variable`. Adjacent tokens with the same class merge into one span. The mapping lives in `site/code.rs` with a test per language in the analysis's table.

Alternatives considered: tree-sitter grammars, which need a C compiler and one crate per language; and a hand-written tokenizer, which would cover fewer languages less accurately and need its own maintenance. `syntect` is widely used for static sites (`zola`, `bat`) and needs no build-time toolchain beyond Rust.

**Markup.** A fenced block renders as

```html
<div class="cp__fenced-code-block" data-lang="rust">
  <div class="extensions__code">
    <div class="extensions__code-lang">rust</div>
    <pre class="lsg-code"><code><span class="line"><span class="cm-keyword">fn</span> …</span>…</code></pre>
  </div>
</div>
```

Logseq's inner `ui-fenced-code-editor` and CodeMirror wrapper `div`s are left out because nothing styles them once there is no editor. Each code line is a `span.line`; the gutter number is CSS generated content (`counter-increment` and `::before`), which browsers leave out of copied text, so copying needs no script. A block with no language has no label `div` and no `data-lang`.

**Example blocks.** `Block::Example` renders `pre.pre-wrap-white-space` with the SDK's code text, as Logseq does.

**Tables.** `Block::Table` writes `div.table-wrapper`, `table.table-auto` with `border="2" rules="groups" frame="hsides"` dropped (CSS gives the look), a `colgroup` of `col.org-left`, `thead` with `th scope="col" class="org-left"` and `tbody` with `td class="org-left"`. Cells keep going through the inline renderer.

**Styles.** A new `site/code.css`, inlined into every page next to `outline.css`, styles the code box, label, gutter, token classes, example blocks and tables. Theme tokens join `site/theme.css`: `--lsg-code-editor-bg`, `--lsg-code-gutter-bg`, `--lsg-code-gutter-fg`, one per token class, `--lsg-table-header-fg`, `--lsg-table-rule`, `--lsg-table-stripe-a` and `--lsg-table-stripe-b`. Light values are the analysis's live measurements. Dark values come from the dark Radix scale in `codemirror.lsradix.css` and `table.css` in `logseq/og`, checked against the `gh-pages` build in dark mode during Apply. The build ships no font file; readers without Fira Code see the next font in the stack.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md):

| Measure | Budget |
| --- | --- |
| Code and table styles on every page, gzip | 1.5 KB |
| Largest code fixture page, gzip | within the existing 100 KB first-paint budget, at most 10 KB above the build without highlighting |
| Script added | none |
| Whole-garden build time | at most 20% above the build without highlighting |

`scripts/parity/budgets.toml` gains a `[code]` table with `styles_gzip_kb = 1.5`, `page_growth_gzip_kb = 10` and `build_slowdown_pct = 20`; `parity:budgets` checks the first two and `parity:perf` records the third.

## Risks / Trade-offs

- [Grammar and theme data make the `lsg` binary larger] → `two-face` loads only the syntaxes named in the mode table; Apply records the binary size before and after in `verification.md`.
- [TextMate grammars split tokens differently from CodeMirror modes, so some tokens get a different colour from Logseq's] → The scope mapping is tested per language, and Apply compares the fixture pages with the `gh-pages` build and adjusts the mapping where colours differ on common tokens.
- [Token spans make code-heavy pages larger] → Spans repeat a few class names and compress well; the page-growth budget fails CI before a page nears the first-paint limit.
- [Dark colours are read from source, not measured] → Apply checks them against the `gh-pages` build in dark mode before merging.

## Resolved Questions

### 1 - Is code highlighted in the browser or during the build?

Decided by the agent without codekiln: during the build. Logseq's highlighting comes from an editor bundled in its 35 MB of JavaScript; #106 asks for graph and rendering work at build time with no script where markup will do.

### 2 - Which highlighter does the build use?

Decided by the agent without codekiln: `syntect` with `two-face`, with TextMate scopes mapped to CodeMirror classes. It covers the garden's languages in pure Rust; tree-sitter needs a C toolchain and one crate per language, and a hand-written tokenizer would be less accurate.

### 3 - Which languages are coloured?

Decided by the agent without codekiln: exactly those Logseq's `CodeMirror.findModeByName` recognises, so `py`, `rs`, `text`, `nix`, `jsonc` and `mermaid` stay plain as on Logseq's site. Parity is the goal, and the lookup belongs in the SDK where the CLI can use it too.

### 4 - How much indentation is removed from code text?

Decided by the agent without codekiln: the owning bullet's continuation prefix, its leading whitespace plus two spaces, which matches what Logseq shows on the fixture pages and keeps indentation the author wrote inside the fence.

### 5 - How are line numbers shown without a script?

Decided by the agent without codekiln: one `span.line` per line with a CSS counter in `::before`. Generated content is left out of copied text, which matches CodeMirror's gutter behaviour.

### 6 - Does the table markup keep Logseq's `border`, `rules` and `frame` attributes?

Decided by the agent without codekiln: no. They are presentational attributes that the stylesheet overrides; the CSS gives the same look with less markup.

### 7 - Do Markdown alignment markers such as `:--:` align cells?

Decided by the agent without codekiln: no. Logseq marks every cell `org-left` and ignores them; parity is the goal.

### 8 - Are the code and table styles on every page or only pages with code or tables?

Decided by the agent without codekiln: every page, as with the outline and inline styles. One inlined stylesheet keeps rendering simple, and 1.5 KB gzip is within the per-page budgets.

## Open Questions

None.
