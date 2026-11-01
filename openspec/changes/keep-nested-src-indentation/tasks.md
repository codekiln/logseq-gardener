## 1. Reproduce

- [x] 1.1 Add SDK tests for a nested `#+BEGIN_SRC` block with tab-indented code, compared with the same code in a fenced block and a `#+BEGIN_EXAMPLE` block, and for a `#+BEGIN_SRC` block under a top-level bullet; see them fail
- [x] 1.2 Record what Logseq 0.10.15 and mldoc give for the same case graph

## 2. Fix

- [x] 2.1 Remove the outline prefix in `code_text` only for a fenced block

## 3. Verification

- [x] 3.1 Publish the case graph before and after, and record the code text in `verification.md`
- [x] 3.2 Run `mise run ci`
