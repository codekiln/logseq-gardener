## Context

`lsdoc`, the parser, returns a fenced block's body as it appears in the file, with the outline's indentation on every line. It returns a `#+BEGIN_SRC` body and a `#+BEGIN_EXAMPLE` body with that indentation already removed. `code_text` in `code_blocks.rs` removed the indentation from every `Src` block, fenced or not, because the parser reports both as `Block::Src`. For a `#+BEGIN_SRC` body the second removal took leading characters that belong to the code whenever they matched the bullet's prefix: a tab under a tab-indented bullet, or two spaces under a top-level bullet.

Logseq 0.10.15 stores the block's content without the outline's indentation, and mldoc keeps every line of the source block as it is in that content. On a case graph, Logseq gives `\treturn 1` for the same line in a nested `#+BEGIN_SRC` block, a nested fenced block and a nested `#+BEGIN_EXAMPLE` block.

## Goals / Non-Goals

**Goals:**

- A `#+BEGIN_SRC` block gives the same code text as the same code in a fenced block or a `#+BEGIN_EXAMPLE` block.

**Non-Goals:**

None.

## Decisions

- `code_text` keeps a `Src` block's text as the parser gives it when the block's first line, after its indentation and an optional `- `, starts with `#+`, as for an example block. Every other `Src` block, a fenced block, loses the prefix as before. Alternatives considered: removing the prefix only when the first line starts with a fence marker, which would change fences inside a quote that the current rule handles; and changing the parser, which belongs to the `lsdoc` project.
- `mise run docs:parser-check` gets no fixture. The check compares page relationships, references and properties, not code text, so a fixture would pass before and after the fix.

## Risks / Trade-offs

- [A `#+BEGIN_SRC` line written with more indentation than its bullet's prefix] → The parser has already removed the prefix, so the extra indentation stays, as in Logseq.

## Resolved Questions

### 1 - Does the parser check cover this?

Decided by the agent without codekiln: no. The check compares references and properties, not code text. The SDK test and a direct comparison with mldoc 1.5.7 on a case graph record the evidence instead.

## Open Questions

None.
