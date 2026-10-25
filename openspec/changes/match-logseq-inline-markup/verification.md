# Verification: match-logseq-inline-markup

`lsg`'s inline markup now matches Logseq's built-in look on the five fixture pages in the light theme. Every difference that remains comes from `pengx17/logseq-dev-theme`, which the encode garden's `logseq/custom.css` imports from a CDN. `tmux` grows 1,465 bytes gzip, inside the 2 KB budget.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `8bc3391`, served locally and opened in Playwright's Chromium (`playwright==1.60.0`) at 1280×800 and 390×844 with `color_scheme` light and dark. The `lsg` build is `mise run parity:build` of the whole encode garden from this branch.

## Computed styles

Light theme, desktop and phone alike. "Same" means every compared property matched: colour, background, font size and weight, opacity, padding, radius and text decoration.

| Fixture | Element | Logseq | `lsg` |
| --- | --- | --- | --- |
| `tmux` | `[[tmux/Config]]` link | `rgb(16,107,163)`, weight 400, no underline | Same |
| `tmux` | `[[` bracket | `rgb(107,114,128)` at opacity 0.3, 12.8px | Same colour and opacity, 16px |
| `tmux` | Heading link to `github.com/tmux/tmux` | `rgb(0,65,116)`, 1px solid bottom border, `target="_blank"` | Same, plus `rel="noopener noreferrer"` |
| `1Password/Quick Access` | `Cmd+\` inline code | 12.8px, `rgb(241,243,245)` background, `3px 5px` padding, 4px radius | Same |
| `1Password/Quick Access` | `#Keyshort` tag | `rgb(52,81,178)`, 2px border, 0.8em, weight 600 (dev theme) | `rgb(52,81,178)`, 0.9em, opacity 0.8, `0 2px` padding (Logseq's own rule) |
| `1Password/Dev/Connect Server` | Connect diagram | `width=231 height=265`, 231×265 box | `width="231" height="265"`, `loading="lazy"`, 3px radius |
| `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` | Highlight | 4px amber underline, no fill (dev theme) | `#fef3ac` fill, `#262626` text, `2px 4px` padding, 3px radius (Logseq's own rule) |
| `LangSmith/…/1045 Engine Workshop/Take-Aways` | `[[LangSmith roadshow, Boston]]` | `rgb(16,107,163)` | Same |

The connect diagram is a remote image that did not load in the test browser for either build. Logseq's wrapper keeps the 231×265 box; `lsg`'s unloaded image shows its alt text. A loaded image takes its authored size from the attributes.

### Dark theme

Inline code (`rgb(32,36,37)`) and tags (`rgb(132,157,255)`) match. Page references, outside links and highlights differ, all from the dev theme's dark palette:

| Element | Logseq with the dev theme | `lsg` (Logseq's built-in dark values) |
| --- | --- | --- |
| Page reference | `#a3cef1` (`--ct-primary-color`) | `#8abbbb` (`--ls-link-text-color`) |
| Outside link | `#6096ba` (`--ct-secondary-color`) | `#849dff` (Radix indigo 11) |
| Highlight | `#d3a400` underline | `#fef3ac` fill |

## Where the remaining differences come from

`logseq/custom.css` in the encode garden starts with `@import url("https://cdn.jsdelivr.net/gh/pengx17/logseq-dev-theme@main/custom.css")`. That theme sets `mark` to a 4px underline in `--ct-highlight-color`, `a.tag` to a 2px border at 0.8em and weight 600, and the dark link colours above. None of these rules are in the `gh-pages` build's own `style.css`, whose `mark` rule is the `#fef3ac` fill and whose `a.tag` rule has no border.

`lsg`'s built-in styles follow Logseq's own stylesheet. Publishing with `--stylesheet logseq/custom.css` loads the garden's theme after them, and its selectors (`mark`, `a.tag`, `.page-reference`, `.external-link`) now find `lsg`'s elements. Loading the garden's `custom.css` by default, as Logseq does, is recorded on [Issue #106](https://github.com/codekiln/logseq-gardener/issues/106).

The 12.8px bracket size has no rule in either stylesheet that explains it; the brackets keep the text's size.

## Size

`tmux` is the largest fixture page. Sizes are first-paint bytes gzip, from `scripts/parity/budgets.py`'s `page_sizes`, before (`main` at `1cf4a53`) and after this change:

| Page | Before | After | Growth |
| --- | ---: | ---: | ---: |
| `tmux` | 95,156 | 96,621 | 1,465 |
| `1Password/Quick Access` | 9,115 | 9,659 | 544 |
| `1Password/Dev/Connect Server` | 8,418 | 8,918 | 500 |
| `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` | 11,150 | 11,681 | 531 |
| `LangSmith/…/1045 Engine Workshop/Take-Aways` | 11,342 | 12,044 | 702 |

The first build wrote brackets as `span.bracket` elements, as Logseq does; `tmux` grew 2,562 bytes, over the 2 KB budget. Drawing the brackets with `::before` and `::after` saved 1,132 bytes on that page (design, question 10). The inline styles are 467 bytes gzip against a 1 KB budget. No script was added: every page's script before the title stays 9,012 bytes in the fixture build.

`mise run parity:budgets` checks the inline styles and the largest fixture page's growth in CI.
