# Verification

Measured on 2026-10-27 with an `lsg` release build of this change (`parity:build`, garden commit cc1bb2eb, 8,697 pages), served from localhost and driven in a browser at 1280×800 and 390×844. Logseq's values come from the [Right sidebar analysis](../../../docs/parity/right-sidebar.md), measured on the `gh-pages` build 8132c95; they were not captured again for this change.

## Behaviour

| Check | Logseq (analysis) | lsg |
| --- | --- | --- |
| Header order on `tmux` | Toggle left sidebar, Home, Search, More, Toggle right sidebar | Toggle left sidebar, Home, Search garden, More, Toggle right sidebar |
| `t r` at 1280px | opens a 512px column, main 768px | 512px column, main column and header 768px wide, page 1,280px wide (no sideways scroll); `aria-expanded="true"`; `t r` again closes it |
| Empty sidebar | Contents card | Contents card with one empty bullet from `pages/contents.md`, no fetch |
| Shift-click `tmux/Idea/Pomodoro with status line and silent flash`, then the second bullet | block card above page card, URL unchanged | block card titled `tmux › tmux` above the page card; URL unchanged; the placeholder Contents card was replaced |
| Page card content | no title, outline, Linked References closed | no `h1`; first heading `Pomodoro in tmux: status-line countdown and silent visual cue`; Linked References closed; no Unlinked References |
| Linked References opened in a card | loads references | three groups loaded from `_refs/p-b9735a668a69fe2f-linked-1.html` |
| Block reference on `VSCode/Proj/24t3/LearnVSCode/01 Terminal` | Block references card | `Block references` card, breadcrumb `VSCode/Proj/24t3/LearnVSCode/01 Terminal › #Todos`, then the referenced TODO block |
| Card menu with three cards | Close, Close others, Close all, Collapse, Collapse others, Collapse all, Expand all | the same seven items in two groups; focus on Close; arrows move; Escape closes and returns focus to Card menu |
| Resize | 3px separator, `max(320px, 10%)` to 70% | drag to 600px saved; after reload the width is `600px` before the cards load |
| Saved state | survives reloads | two cards restored in order after a reload and after following a link |
| 390×844 | no toggle; `t r` opens a 156px column | no toggle (`display: none`), `t r` ignored, Shift-click not cancelled by the page, `lsg-sidebar.js` never requested, page 390px wide |

The script tests in `tests/sidebar.test.cjs` and `tests/shell.test.cjs` cover Shift-click targets, card order and moving to the top, page and block content, menu items by card type and count, the keyboard menu, the saved state round trip, start cards from `:default-home`, the Help card, unpublished pages, phone width and loading on first use.

## Sizes and times

| Measure | Budget | lsg |
| --- | --- | --- |
| `lsg-shell.js` | 6,500 bytes | 6,463 |
| `lsg-sidebar.js`, gzip | 6,000 bytes | 5,744 (fixture build); 18,256 bytes uncompressed |
| `lsg-sidebar.css`, gzip | none | 1,131 |
| Fixture first paint and frame | `[desktop]`, `[phone]`, `[shell]` | `parity:budgets` passes for all 14 fixture pages |
| Shift-click to card outline, p95, desktop | 300 ms | 27 ms over ten page cards (first includes loading the script) |
| `Person` card transfer, gzip | 60 KB | 14.2 KB |
| JS heap after ten cards, desktop | 20 MB (phone) | 3.1 MB |

Two pages are larger than the card budget: `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton` (81.9 KB gzip) and `Ableton/Live/UG/29 Audio Effect Reference` (62.1 KB gzip). The page content makes up the size, so content-only fragments would save only the 7 KB shell and were not added.

## Differences from the plan

- Before first paint, a saved open sidebar reserves the column's width; the card frames appear after the `load` event with their content, not before paint. The page does not move when they appear.
- `pages/contents.md` was missing from the parity build because `scripts/parity/build.py` selected the root `contents` while the page's title is `Contents`. The fix is in the harness; the SDK already published the page.
- The EDN reader keeps one more level of nesting so that `:default-home {:sidebar [..]}` can be read.
- `lsg-sidebar.js` is published without indentation or comment lines, which keeps it within its gzip budget.
- `parity:perf` and `parity:screenshots` do not measure the sidebar yet; the times above were measured by hand. [Issue #219](https://github.com/codekiln/logseq-gardener/issues/219) tracks the harness work.
