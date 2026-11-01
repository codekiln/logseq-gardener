## Why

A `#+BEGIN_SRC` block under a nested bullet loses the leading tabs of its code lines in the published site ([Issue #360 — Keep indentation inside nested #+BEGIN_SRC blocks](https://github.com/codekiln/logseq-gardener/issues/360)). Logseq keeps them, and so does the SDK for a fenced block or a `#+BEGIN_EXAMPLE` block. The parser already removes the outline's indentation from a `#+BEGIN_SRC` body, and the SDK removes it a second time, so a code line that starts with the same characters as that indentation loses them.

## What Changes

- The SDK removes the outline's indentation only from a fenced code block. A `#+BEGIN_SRC` block keeps the text the parser gives it, as a `#+BEGIN_EXAMPLE` block already does.
- Code that starts with a tab under a tab-indented bullet, or with two spaces under a top-level bullet, keeps that indentation in a `#+BEGIN_SRC` block, as in Logseq.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `code-blocks`: the requirement "Give code text without the outline's indentation" names which blocks lose the prefix and adds a scenario for a nested `#+BEGIN_SRC` block.

## Impact

- `crates/logseq-gardener-sdk/src/code_blocks.rs`: `code_text` removes the prefix only for a fenced block.
- Published sites: the code in a `#+BEGIN_SRC` block under a bullet keeps its indentation where it starts with the bullet's own indentation characters. Fenced and example blocks do not change.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
