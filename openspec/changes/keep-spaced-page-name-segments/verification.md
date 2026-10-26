# Verification: keep-spaced-page-name-segments

The three pages from Issue #129 now publish under Logseq's titles, and the whole-garden comparison reports no page that `lsg` cannot select and no page found only in `lsg`.

Measured on 2026-10-26 against the `gh-pages` build `8132c95` of garden commit `083106d`, with `mise run parity:build` and `mise run parity:compare` from this branch.

| Group | Before (`main` 8d44fd4, per Issue #129) | After |
| --- | --- | --- |
| Titles lsg cannot select | 3 | 0 |
| Only in lsg | not reported | 0 |
| Other pages | 3 | 3 (unchanged: an underscore-named report page, `contents`, `PARAMETERS`) |

Published titles, read from each page's `<title>`:

- `Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?`
- `Claude/Code/Q/Is there a Claude Code CLI slash command to see a diff of changed files, and why doesn't /diff show the diff content?`
- `Obsidian/Q/In Obsidian, is there a command that inserts a callout, and what are those colored Info / Warning / Danger blocks called?`

Logseq's database also holds four generated namespace parents that end in a space, such as `claude/code/q/if i rename a cli session with `. `lsg` lists the same four in `pages.html` with the trailing space. The harness had trimmed link text, which made these four look like `lsg`-only pages; it now reads titles as written.
