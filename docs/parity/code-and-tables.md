# Fenced code and Markdown tables

Logseq shows each fenced code block in a read-only CodeMirror editor: the outline's indentation removed, line numbers in a grey gutter, the language name in the top-right corner, and token colours for the languages CodeMirror knows. Markdown tables span the content column with a 2px rule under the header and striped rows. `lsg` writes the same code as a plain `<pre><code>` that keeps the outline's indentation on every line (3,393 of 3,408 blocks in the encode garden), with no language label, colours or line numbers, and writes tables at their natural width with bold, centred headers and no striping.

The most visible gap is the indentation, which comes from how the SDK hands code text to the publisher. Highlighting must happen at build time: Logseq's highlighting comes from CodeMirror, part of the roughly 35 MB of JavaScript its site loads, and `lsg` should send coloured HTML and CSS instead of a script.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `d200706` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden from `e88e16b` (`mise run parity:build`).

## How often the garden uses them

The encode garden has 1,706 fenced code blocks (backtick and tilde fences) and 138 Markdown tables on 112 pages.

| Fence language | Blocks | Logseq highlights it |
| --- | --- | --- |
| (none) | 599 | no |
| `bash` | 284 | yes, Shell |
| `sh` | 157 | yes, Shell |
| `json` | 142 | yes |
| `toml` | 94 | yes |
| `python` | 93 | yes |
| `text` | 68 | no |
| `rust` | 57 | yes |
| `yaml` | 45 | yes |
| `typescript`, `ts` | 30 | yes |
| `markdown` | 17 | yes |
| `lua` | 10 | yes |
| `javascript` | 9 | yes |
| `zsh` | 8 | yes, Shell |
| `nix` | 8 | no |
| `mermaid` | 7 | no |
| `dockerfile` | 7 | yes |
| `yml` | 5 | yes, YAML |
| `clojure`, `xml`, `jsonc`, `calc`, `vim` | 4 each | `clojure` and `xml` yes; the others no |
| about 20 others | 1–3 each | `edn`, `css`, `c`, `go`, `haskell`, `ini`, `cpp`, `java` yes; `env`, `gitignore`, `kdl`, `tmux`, `md`, `config`, `bell` no |

"Logseq highlights it" is `CodeMirror.findModeByName` on the live build: `py`, `rs`, `md`, `text`, `nix`, `jsonc` and `mermaid` find no mode, so those blocks show plain text.

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where the checkout's CSS differs.

### Fenced code

```html
<div data-lang="rust" class="cp__fenced-code-block">
  <div class="ui-fenced-code-editor">
    <div class="extensions__code">
      <div class="extensions__code-lang">rust</div>
      <div class="code-editor …"><div class="CodeMirror cm-s-lsradix cm-s-light">…</div></div>
    </div>
  </div>
</div>
```

- `["Src" …]` renders `div.cp__fenced-code-block` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3342-L3350)), which runs `src-cp` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3152-L3183)). `edn`, `clj`, `cljc`, `cljs` and `clojurescript` become `clojure`.
- The published site loads a real CodeMirror 5 editor, made read-only by `config/publishing?` ([`code.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.cljs#L387-L419)). Line numbers are on; lines do not wrap and scroll sideways. `:editor/extra-codemirror-options` in `config.edn` can change both; the encode garden does not set it.
- **Text:** the block's indentation is gone: the first line of the `Rust/loop/label` example is `fn loop_labels_to_disambiguate_multiple_loops() {`. One trailing newline is dropped.
- **Language mode:** the lower-cased fence language goes to `CodeMirror.findModeByName`; a miss falls back to plain text ([`code.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.cljs#L359-L386)). Logseq bundles about 120 modes ([`code.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.cljs#L8-L129)).
- **Theme:** live, the editor uses `cm-s-lsradix cm-s-light` ([`codemirror.lsradix.css`](https://github.com/logseq/og/blob/6e7afa8eb/resources/css/codemirror.lsradix.css#L32-L141)). Light values:
  - Editor: `"Fira Code", Monaco, Menlo, Consolas, "COURIER NEW", monospace`, 12px, line height 16px, background `rgb(248,249,250)`, radius 2px, 18px space above the first line and 62px below the last.
  - Gutter: background `rgb(241,243,245)`, numbers `rgb(136,144,150)` at 12px with `0 5px` padding.
  - Tokens: keyword `rgb(189,75,0)`, string `rgb(41,124,59)`, number `rgb(205,29,141)`, definition `rgb(0,120,161)`, operator and type `rgb(121,58,175)`, variable `rgb(17,24,28)`. Comments are italic in the gutter-number grey family (`--lx-gray-10`). The dark theme uses the same classes with the dark Radix scale; it was not measured live.
- **Language label:** `div.extensions__code-lang` with the language as typed in lower case, absolutely positioned top right, 12.25px, `4px 6px` padding, opacity 0.8 (1 on hover), on the editor's background ([`code.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.css#L1-L25)). A block with no language has no label. `calc` blocks have no label and show computed results under the code ([`code.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.cljs#L420-L428), [L506-L519](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/code.cljs#L506-L519)).
- There is no copy button.
- **Mermaid:** no Mermaid rendering exists without a plugin. `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain` shows its diagram source as plain text labelled `mermaid`.

### Other code forms

- `#+BEGIN_SRC` parses to the same `Src` block and renders the same way.
- `["Example" …]` renders `pre.pre-wrap-white-space`, which wraps lines, on `--ls-secondary-background-color` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3251-L3253), [`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L77-L82)).
- Inline code is part of [Issue #112](https://github.com/codekiln/logseq-gardener/issues/112) and its [analysis](inline-markup.md).

### Tables

```html
<div class="table-wrapper">
  <table border="2" rules="groups" frame="hsides" class="table-auto">
    <colgroup><col class="org-left"><col class="org-left"></colgroup>
    <thead><tr><th scope="col" class="org-left">Context</th>…</tr></thead>
    <tbody><tr><td class="org-left">…</td>…</tr>…</tbody>
  </table>
</div>
```

- Built by `table` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3075-L3117)); `:logseq.table.version` (block property or config, default 1) selects this version ([`shui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/shui.cljs#L12-L23)). Every cell is `org-left`, so Markdown alignment markers such as `:--:` have no effect.
- Live on `Rust/loop/label` ([`table.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/table.css#L1-L97)):
  - The table fills the content column (718px there), `border-collapse: collapse`, `14px 0` margin; the wrapper scrolls sideways when the table is wider than the column.
  - Header cells: 14px, weight 400, colour `rgb(104,112,118)`, left aligned, `10px 8px` padding, a 2px `rgb(230,232,235)` bottom rule, no background.
  - Body cells: 16px, `6px 8px` padding, no rules between rows.
  - Rows alternate `rgb(251,252,253)` and `rgb(236,238,240)`.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/render.rs` (around line 582) and the whole-garden build:

- **Code:** `Block::Src` and `Block::Example` both render `<pre><code>` with the escaped text. The parser's `lang` is dropped, so there is no label and no colouring. The page stylesheet gives the `pre` a grey background, 16px padding and sideways scrolling, in the browser's `monospace` at 16px with a 25.6px line height.
- **Indentation:** the text keeps the bullet's continuation indentation. On `Rust/loop/label` every line starts with a tab and two spaces. 3,393 of the 3,408 `pre` blocks in the build have every line indented this way.
- **Tables:** `<table>` of `<tr>` rows, the header row in `<th>`, with no `thead`, `tbody` or wrapper. Natural width (348px on `Rust/loop/label`), no margin, header cells bold, centred and 16px in the text colour, 1px rules under every row, `6.4px` padding, no striping. A wide table pushes the page sideways instead of scrolling inside a wrapper.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Indentation.** Remove the block's continuation indentation from code text, as Logseq does, so the first line starts at column 0 when the author wrote it there. Check: on `Rust/loop/label` the first code line is `fn loop_labels_to_disambiguate_multiple_loops() {`; across the build, a line keeps leading whitespace only when the author indented it inside the fence.
- 2. **Language label.** Show the fence language in lower case at the top right with Logseq's size, padding, opacity and background; none for a block without a language. Check: `Rust/loop/label` shows `rust`.
- 3. **Token colours.** Colour tokens for the languages Logseq highlights in the table above, using Logseq's token classes and colours in both themes, computed at build time. Languages Logseq leaves plain (including `py`, `rs`, `text`, `mermaid` and unknown names) stay plain. Check: on `Rust/loop/label` the `loop`, `let` and `break` keywords have Logseq's keyword colour, string literals the string colour; on the batch-inference page the Python block is coloured and the Mermaid block is not.
- 4. **Editor box.** Logseq's monospace stack, 12px text, 16px line height, background, 2px radius and spacing; no wrapping, sideways scroll. Check: computed values on `Rust/loop/label` match the list above.
- 5. **Line numbers.** A gutter with Logseq's background and number colour, numbers not selected when a reader copies the code. Check: the `Rust/loop/label` example shows numbers 1 to 23 and copying the block yields only the code.
- 6. **Example blocks.** Wrapped `pre` on the secondary background, separate from fenced code. Check: an `#+BEGIN_EXAMPLE` block in a test garden wraps long lines.
- 7. **Table markup.** `div.table-wrapper` around a `table` with `thead` and `tbody`; alignment markers ignored. Check: `Rust/loop/label`'s table has a `thead` with two `th` cells and a wrapper.
- 8. **Table style.** Full column width, Logseq's header size, weight, colour, alignment and rule, cell padding and row striping, in both themes; the wrapper scrolls sideways on a phone. Check: computed values on `Rust/loop/label` and `Claude/Code/Plugin/Marketplace/Auto-Update` match the list above at 1280×800, and at 390×844 the widest table scrolls inside its wrapper.
- 9. **Performance.** Highlighting and line numbers add markup to pages with code, and none of this feature may add JavaScript. Check: `parity:budgets` passes; the largest code-heavy fixture page grows by a measured, budgeted amount; no script is added.

## Where the work belongs

- **SDK:** code block text with the outline's indentation removed and the fence language, as part of the block model, so the CLI and language server see the same code the publisher shows. The language-to-highlighter mapping (Logseq's name and alias rules) is a reusable lookup that also belongs in the SDK.
- **Publisher:** the build-time token spans and their CSS, the label, the gutter, and the table markup and styles.

## Fixture pages

| Page | Why |
| --- | --- |
| `Rust/loop/label` | Rust block with keywords, strings and labels; a two-column table with inline code in cells; the indentation gap |
| `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain` | Mermaid, Bash and Python blocks on one page |
| `Rust/script` | Several `~~~bash` blocks |
| `MCP/Code Mode` | `~~~python` |
| `Claude/Code/Plugin/Marketplace/Auto-Update` | Four tables on one page |

## Out of scope

- `calc` blocks: Logseq evaluates each line and shows the results. The garden has 4. Record under #106 if the comparison shows them on fixture pages.
- Rendering Mermaid diagrams, which Logseq does only with a plugin.
- Table version 2 (`:logseq.table.version 2`), which the encode garden does not use.
- Editing behaviour: focusing the editor, active-line highlight, bracket matching.
