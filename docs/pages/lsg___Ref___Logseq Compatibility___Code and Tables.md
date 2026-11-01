tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Outline and Blocks]], [[lsg/Ref/Rendering and Assets]]

- # Code and Tables Compatibility
	- `lsg` publishes fenced code as Logseq's site shows it: the outline's indentation removed, the fence language as a label at the top right, a line-number gutter, and Logseq's `lsradix` token colours for exactly the languages Logseq's CodeMirror recognises, in Logseq's 12px editor box that scrolls sideways. Example blocks wrap, and Markdown tables fill the content column with Logseq's header rule, cell padding and striped rows, ignoring alignment markers as Logseq does. It colours code during the build and adds no script, so there is no editor behind the code, and it leaves `calc` blocks unevaluated and Mermaid diagrams as source text.
	  id:: 16eeb7a4-dcaa-450a-8d66-31c311ff7c17
	- Measured on 2026-10-25 against the `gh-pages` build of garden commit `d200706` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden from `e88e16b` (`mise run parity:build`). The OpenSpec change [match-logseq-code-and-tables](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-26-match-logseq-code-and-tables) then built the code and table rendering, and [published-code-and-tables](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-code-and-tables/spec.md) and [code-blocks](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/code-blocks/spec.md) hold its requirements.
	- ## How often the garden uses them
		- The encode garden has 1,706 fenced code blocks (backtick and tilde fences) and 138 Markdown tables on 112 pages.
		- | Fence language | Blocks | Logseq highlights it |
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
		- "Logseq highlights it" is `CodeMirror.findModeByName` on the live build: `py`, `rs`, `md`, `text`, `nix`, `jsonc` and `mermaid` find no mode, so those blocks show plain text.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where the checkout's CSS differs.
		- ### Fenced code
			- ```html
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
		- ### Other code forms
			- `#+BEGIN_SRC` parses to the same `Src` block and renders the same way.
			- `["Example" …]` renders `pre.pre-wrap-white-space`, which wraps lines, on `--ls-secondary-background-color` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3251-L3253), [`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L77-L82)).
			- [[lsg/Ref/Logseq Compatibility/Inline Markup]] covers inline code.
		- ### Tables
			- ```html
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
	- ## What lsg publishes
		- ### Fenced code
			- Each fenced code block renders as `div.cp__fenced-code-block` with the fence language in `data-lang`. The code text starts at column 0 when the author wrote it there: the SDK removes the owning bullet's continuation indentation, its leading whitespace plus two spaces, and keeps indentation the author wrote inside the fence. On `Rust/loop/label` the first line is `fn loop_labels_to_disambiguate_multiple_loops() {`.
			- A block with a language shows `div.extensions__code-lang` with the language in lower case at the top right, 12.25px, `4px 6px` padding, opacity 0.8 rising to 1 on hover, on the editor's background. A block without a language has no label and no `data-lang`.
			- The code uses `"Fira Code", Monaco, Menlo, Consolas, "COURIER NEW", monospace` at 12px with 16px lines, Logseq's editor background for the active theme, a 2px radius, 18px above the first line and 62px below the last, and scrolls sideways at any width while the page stays put.
			- The build colours tokens for exactly the languages Logseq's `CodeMirror.findModeByName` recognises, with Logseq's CodeMirror token classes (`cm-keyword`, `cm-string`, `cm-number`, `cm-def`, `cm-operator`, `cm-type`, `cm-variable`, `cm-comment` and the others) and the `lsradix` colours for the active theme, comments in italic. On `Rust/loop/label`, `loop`, `let` and `break` are `rgb(189,75,0)` in the light theme and string literals `rgb(41,124,59)`. `py`, `rs`, `text`, `nix`, `jsonc`, `mermaid` and unknown names stay plain, as on Logseq's site.
			- Each code line is a `span.line`, and a CSS counter numbers it in a gutter with Logseq's gutter background and number colour, 12px with `0 5px` padding. Copying a block copies only the code, because browsers leave generated content out of copied text.
			- `#+BEGIN_EXAMPLE` blocks render as `pre.pre-wrap-white-space` on the theme's secondary background, wrapping long lines, without a label, gutter or colours.
		- ### Tables
			- Each Markdown table renders as `div.table-wrapper` around `table.table-auto`, with a `colgroup` of `col.org-left`, the header row in `thead` as `th scope="col"` cells and the other rows in `tbody`. Every cell carries `org-left`, and alignment markers have no effect.
			- The table fills the content column with collapsed borders and a `14px 0` margin. Header cells are 14px, weight 400, left aligned, in Logseq's header colour, with `10px 8px` padding, a 2px bottom rule and no background. Body cells are 16px with `6px 8px` padding and no rules between rows, and rows alternate Logseq's two stripe colours, in both themes.
			- A table wider than the column scrolls inside its wrapper, and the page does not scroll sideways at 390×844. Inline markup in cells renders as elsewhere on the page.
		- ### How it is built
			- The SDK's `code_blocks` module gives each code and example block its text without the outline's indentation and looks up the fence language through Logseq's names and aliases, so the CLI, the language server and the publisher read the same code.
			- The publisher highlights with `syntect` and the grammars from `two-face`, maps TextMate scopes to CodeMirror token classes in `site/code.rs`, highlights each distinct block once across the machine's cores, and reuses the markup wherever the block appears again. `site/code.css`, inlined into every page, styles the code box, label, gutter, tokens, example blocks and tables.
		- ### Budgets and measurements
			- The `[code]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) limits the code and table styles on every page to 1.5 KB gzip and the growth of the largest code fixture page to 10 KB gzip over its size without highlighting, within the 100 KB first-paint budget. A whole-garden build may take at most 20% longer than without highlighting. `parity:budgets` checks the sizes in CI, and code and tables add no script.
			- On 2026-10-26 (`lsg` `b490d68`), the `ff` fixture page, which holds fenced code, showed its title in 27.9 ms on desktop and 238 ms on the phone profile, with 14.5 KB of data before it. [[lsg/Ref/Logseq Compatibility/Performance]] holds every row.
	- ## Differences from Logseq
		- `lsg` writes coloured HTML and CSS during the build. Logseq's colours come from a read-only CodeMirror editor that is part of the roughly 35 MB of JavaScript its site loads; `lsg` adds no script for code.
		- Logseq's inner `ui-fenced-code-editor` and CodeMirror wrapper elements are left out, since nothing styles them without an editor. Focusing the editor, the active-line highlight and bracket matching have no counterpart.
		- TextMate grammars split some tokens differently from CodeMirror modes, so an occasional token can take a different colour from Logseq's. Where CodeMirror leaves a token plain that a grammar marks, the mapping follows CodeMirror: Shell command names stay plain.
		- The build ships no font file; readers without Fira Code see the next font in the stack.
		- Tables drop Logseq's `border`, `rules` and `frame` attributes; the stylesheet gives the same look.
		- `calc` blocks show their source without results. Logseq evaluates each line and shows the results; the encode garden has 4.
		- Mermaid blocks show their source, as on Logseq's site, which renders diagrams only with a plugin.
		- Table version 2 (`:logseq.table.version 2`) is not rendered; the encode garden does not use it.
		- The dark editor and table colours come from Logseq's dark Radix scale in `codemirror.lsradix.css` and `table.css`; Logseq's dark theme was not measured live.
	- ## Before the change
		- The `lsg` build measured on 2026-10-25, from `e88e16b`, rendered code and tables from `crates/logseq-gardener-sdk/src/site/render.rs` (around line 582).
		- **Code:** `Block::Src` and `Block::Example` both rendered `<pre><code>` with the escaped text. The parser's `lang` was dropped, so there was no label and no colouring. The page stylesheet gave the `pre` a grey background, 16px padding and sideways scrolling, in the browser's `monospace` at 16px with a 25.6px line height.
		- **Indentation:** the text kept the bullet's continuation indentation. On `Rust/loop/label` every line started with a tab and two spaces. 3,393 of the 3,408 `pre` blocks in the build had every line indented this way.
		- **Tables:** `<table>` of `<tr>` rows, the header row in `<th>`, with no `thead`, `tbody` or wrapper. Natural width (348px on `Rust/loop/label`), no margin, header cells bold, centred and 16px in the text colour, 1px rules under every row, `6.4px` padding, no striping. A wide table pushed the page sideways instead of scrolling inside a wrapper.
	- ## Fixture pages
		- | Page | Why |
		  | --- | --- |
		  | `Rust/loop/label` | Rust block with keywords, strings and labels; a two-column table with inline code in cells; the indentation the build removes |
		  | `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain` | Mermaid, Bash and Python blocks on one page |
		  | `Rust/script` | Several `~~~bash` blocks |
		  | `MCP/Code Mode` | `~~~python` |
		  | `Claude/Code/Plugin/Marketplace/Auto-Update` | Four tables on one page |
