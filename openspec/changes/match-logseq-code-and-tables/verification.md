# Verification: match-logseq-code-and-tables

`lsg`'s fenced code and Markdown tables now match Logseq's published site on the five fixture pages: the same text at column 0, label, line count, font, background, token colours, table header style, width and striping, in the light theme at 1280×800 and 390×844. Dark values come from Logseq's live build. The code and table styles are 896 bytes gzip on every page, the largest code page in the fixture garden grows 1,077 bytes gzip, no script was added, and a whole-garden build takes 13–16% longer. The encode garden's `tmux` page, already at 98.5 KB, now measures 101 KB before its title, over the 100 KB first-paint budget; see [Size and build time](#size-and-build-time).

Measured on 2026-10-25 against the `gh-pages` build of garden commit `d200706`, served locally and opened in Playwright's Chromium (`playwright==1.60.0`) with `color_scheme` light and dark. The `lsg` build is `mise run parity:build` of the whole encode garden from this branch.

## Fixture pages

Light theme, at 1280×800 and 390×844 alike. "Same" means the two builds gave the same value.

| Fixture | Check | Logseq | `lsg` |
| --- | --- | --- | --- |
| `Rust/loop/label` | First code line | `fn loop_labels_to_disambiguate_multiple_loops() {` | Same |
| `Rust/loop/label` | Label, lines | `rust`, 23 | Same |
| `Rust/loop/label` | Code font and box | 12px/16px, `rgb(248,249,250)`, 2px radius | Same; 18px above the first line, 62px below the last |
| `Rust/loop/label` | Keyword, string | `rgb(189,75,0)`, `rgb(41,124,59)` | Same |
| `Rust/loop/label` | Gutter | `rgb(241,243,245)`, numbers `rgb(136,144,150)` at 12px, `0 5px` | Same |
| `Rust/loop/label` | Table | `thead`, header 14px 400 `rgb(104,112,118)` left, `10px 8px`, 2px `rgb(230,232,235)` rule; rows `rgb(251,252,253)` and `rgb(236,238,240)` | Same |
| `AI/Batch Inference/…/AWS vs OpenAI vs LangChain` | Mermaid, Bash, Python | `mermaid` plain; `bash` and `python` coloured | Same |
| `Rust/script` | Four blocks | `bash` one-line blocks plain, `rust` coloured | Same |
| `MCP/Code Mode` | `python` | coloured | Same |
| `Claude/Code/Plugin/Marketplace/Auto-Update` | Three `json` blocks, tables | strings coloured; header, rule and stripes as above | Same |

Tables fill the content column: 718px in Logseq and 728px in `lsg` at 1280×800, the width of each build's block content. On a 390px phone the two wide `Auto-Update` tables (315px and 644px) scroll inside their 276px wrappers.

Selecting the whole `Rust/loop/label` block and reading the selection gives the code lines without numbers: the gutter numbers are CSS generated content.

### Dark theme

`lsg`'s dark values were read from Logseq's live build with its dark theme on, and match it: editor `rgb(26,29,30)`, gutter `rgb(32,36,37)` with numbers `rgb(105,113,119)`, keyword `rgb(255,139,62)`, string `rgb(99,193,116)`, table header `rgb(155,161,166)` with a `rgb(43,47,49)` rule, rows `rgb(21,23,24)` and `rgb(38,41,43)`.

### Differences that remain

- **Token boundaries.** `lsg` reads TextMate grammars, Logseq runs CodeMirror modes, and the two split some tokens differently. On `Rust/loop/label` the loop label `'counting_up` is a keyword in `lsg` and plain in Logseq; whitespace after a keyword shares the keyword's span, which changes no colour.
- **Shell command names.** CodeMirror's shell mode leaves command names plain; the Bash grammar marks them as function calls, so `lsg` drops that class for Shell blocks (design: highlighter).
- **Page width on a phone.** The `Auto-Update` page scrolls sideways at 390px in both builds. In `lsg` the cause is long inline code in paragraphs, `CLAUDE_CODE_PLUGIN_KEEP_MARKETPLACE_ON_FAILURE=1`, which belongs to inline markup ([Issue #112](https://github.com/codekiln/logseq-gardener/issues/112)); code blocks and tables scroll inside their own boxes.
- **Languages without a grammar.** A few modes Logseq bundles, such as APL and Cobol, have no grammar in the build and show plain text. None appears in the encode garden's language table.

## Size and build time

| Measure | Before | After | Budget |
| --- | ---: | ---: | --- |
| Code and table styles on every page, gzip | — | 896 | 1,500 |
| `Tools/Code` fixture page, first paint gzip | 8,010 | 9,087 | 18,010 |
| Largest fixture page (`Tools/Code`), first paint gzip, inline budget | — | 9,087 | 9,479 |
| Encode garden `tmux`, first paint gzip (`page_sizes`) | 96,618 | 98,535 | 100,000 |
| Encode garden `Rust/loop/label`, first paint gzip | 8,487 | 9,439 | 100,000 |
| Encode garden `Claude/Code/Plugin/Marketplace/Auto-Update`, first paint gzip | 13,586 | 14,532 | 100,000 |
| Script before the title, every fixture page | 9,012 | 9,012 | no growth |
| `lsg` binary (macOS arm64, release) | 3,075,088 | 6,407,024 | recorded |

"Before" is `main` at `78d6a56`. The grammars account for the binary growth.

Whole-garden build time, three runs each of `lsg publish` with the parity options: `main` 3.71, 3.83 and 3.85 seconds; this branch 4.81, 4.38 and 4.44 seconds. The median grows 16%, inside the 20% budget. `mise run parity:perf --builds lsg --loads 1 --baseline-lsg <main lsg>` measured 3.9 seconds for `main` and 4.4 seconds for this branch (+13%). Highlighting every block while rendering made the build 43% slower; highlighting each distinct block once, in parallel, before pages render brought it down (design: risks).

### The `tmux` page

`parity:perf` measured `tmux` at 101 KB of data before its title on desktop and phone, against 98.5 KB in the recorded table and a 100 KB budget. The page is 707 KB of HTML, almost all of it linked references; its 30 code blocks and 11 tables add 1,265 bytes gzip of markup and the shared styles add 815. Removing every Logseq structure element that no style uses (`colgroup`, `scope`, `org-left` on cells, the `extensions__code` wrapper) would save about 220 bytes, and dropping token colours would give up parity, so neither brings the page back under 100 KB.

Decided by the agent without codekiln: keep Logseq's markup and colours and record the overrun. The page's weight comes from linked references, which [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115) covers; loading references after the title would bring `tmux` far under the budget. `parity:budgets` in CI checks the fixture garden, which stays within every budget.

`docs/parity/performance.md` was not rewritten: a run that also loads Logseq's build did not finish in two and a half hours, and an `lsg`-only run would drop Logseq's rows from the table. The `lsg`-only results above come from that run's `perf.md`.
