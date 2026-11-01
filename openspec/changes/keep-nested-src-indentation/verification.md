# Verification: keep-nested-src-indentation

A `#+BEGIN_SRC` block now keeps the indentation of its code lines, as Logseq 0.10.15 does and as fenced and `#+BEGIN_EXAMPLE` blocks already did. Checked on 2026-11-01 against the SDK built without the fix (before) and with it (after).

## Case graph

One page, `code.md`, holds the same two-line Python function in four blocks: a `#+BEGIN_SRC`, a fenced block and a `#+BEGIN_EXAMPLE` under a bullet indented one tab, with the second line written as the bullet's prefix followed by a tab; and a `#+BEGIN_SRC` under a top-level bullet, with the second line written as the bullet's two spaces followed by two more.

| Block | Logseq | Before | After |
| --- | --- | --- | --- |
| Nested `#+BEGIN_SRC` | `\treturn 1` | `return 1` | `\treturn 1` |
| Nested fenced block | `\treturn 1` | `\treturn 1` | `\treturn 1` |
| Nested `#+BEGIN_EXAMPLE` | `\treturn 1` | `\treturn 1` | `\treturn 1` |
| `#+BEGIN_SRC` under a top-level bullet | `  return 1` | `return 1` | `  return 1` |

The Logseq column is the block content from Logseq's file parser at the revision `mise run docs:parser-check` pins, parsed with mldoc 1.5.7, the version `scripts/logseq-parser/package-lock.json` pins: each `Src` block's lines keep the leading tab or the two spaces. The before and after columns are the `<pre>` text `lsg publish` wrote for each block.

## Published gardens

| Garden | Pages | Files that differ |
| --- | --- | --- |
| Encode garden, `--journals --filename-format triple-lowbar` with one `--include` per top-level name | 8730 pages, 512 journals, 35674 diagnostics with both builds | none |
| `docs`, `--include lsg --include Logseq` | 83 pages, 2 diagnostics with both builds | none |

Neither garden has a `#+BEGIN_SRC` line that starts with its bullet's indentation characters, so neither changes.
