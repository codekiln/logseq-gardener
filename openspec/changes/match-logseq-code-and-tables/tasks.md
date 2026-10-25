## 1. SDK

- [ ] 1.1 Add the `code_blocks` module with `code_text`; unit-test nested, top-level, short and blank lines and the trailing newline
- [ ] 1.2 Add `code_language` with Logseq's mode table and Clojure aliases; unit-test every language in the analysis's table, including those Logseq leaves plain
- [ ] 1.3 Read code text through `code_text` wherever the SDK exposes code blocks

## 2. Highlighting

- [ ] 2.1 Add `syntect` (pure-Rust regex engine) and `two-face`, loading only the syntaxes the mode table names
- [ ] 2.2 Add `site/code.rs`: highlight a block for its mode, map TextMate scopes to CodeMirror classes, merge adjacent same-class tokens; test one sample per mapped language

## 3. Markup

- [ ] 3.1 Render fenced code as `div.cp__fenced-code-block` with `data-lang`, the label and `span.line` lines; extend the render tests for labelled, unlabelled and plain-mode blocks
- [ ] 3.2 Render example blocks as `pre.pre-wrap-white-space`; add a render test
- [ ] 3.3 Render tables with `div.table-wrapper`, `colgroup`, `thead`, `tbody` and `org-left` cells, ignoring alignment; extend the render tests

## 4. Styles

- [ ] 4.1 Add the code and table theme tokens for both themes to `site/theme.css`, light values from the analysis and dark values from Logseq's source
- [ ] 4.2 Add `site/code.css` and inline it into every page next to the outline styles

## 5. Harness and budgets

- [ ] 5.1 Add a `[code]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check the style size and the largest code page's growth, and `parity:perf` record whole-garden build time

## 6. Verification

- [ ] 6.1 Build the encode garden with `parity:build` and compare `Rust/loop/label`, `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain`, `Rust/script`, `MCP/Code Mode` and `Claude/Code/Plugin/Marketplace/Auto-Update` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record computed styles, screenshots and the binary size before and after in `verification.md`
- [ ] 6.2 Run `parity:perf --record` and commit the updated rows in `docs/parity/performance.md`
- [ ] 6.3 Run `mise run ci`
