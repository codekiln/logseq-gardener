## Context

Logseq creates one page per alias name, and a later spelling replaces an earlier one with the same lower-cased name. `scripts/logseq-parser/check.py` shows `Homebrew` for `alias:: [[homebrew]], [[Homebrew]]`. `page_alias_spellings` in `page_names.rs` kept the first spelling per lookup key, and the publisher, the search index and the comparison harness all read spellings from it.

## Goals / Non-Goals

**Goals:**

- One alias value that repeats a name takes the last spelling.
- `check.py` reports no difference on `fixtures/alias-spelling-case`.

**Non-Goals:**

- The spelling an alias takes when a block references it in another case. [Issue #515](https://github.com/codekiln/logseq-gardener/issues/515) tracks it.

## Decisions

- Fix `page_alias_spellings` in place, so every caller gets Logseq's spelling. The alias keeps the position of its first occurrence because the check compares aliases as sets and authored order is the order readers see.
- Spellings across pages still keep the first, as before. The encode garden's remaining title differences, such as `developer`, come from that and from references, and the issue does not cover them.
- The parser check needs a fixture on which Logseq and the SDK still differ, to test that it reports a difference. `fixtures/alias-spelling-reference` is one, filed as #515.

## Risks / Trade-offs

- [The search data of a page that writes an alias twice changes spelling] → It is the spelling Logseq shows.

## Resolved Questions

### 1 - Should a repeated alias move to the position of its last spelling?

No. Logseq's order is not visible on a published page, and keeping the first position leaves the authored order readable.

## Open Questions

None.
