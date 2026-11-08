## Context

Logseq's `block/page-name->map` calls `convert-page-if-journal` (`deps/graph-parser/src/logseq/graph_parser/block.cljs`). That function reads the name with `date-time-util/journal-title->int` over the journal title format, `MMM do, yyyy`, `yyyy-MM-dd` and `yyyy_MM_dd`. When the name reads as a date, it replaces the page's original name with `int->journal-title`, the date written in the journal title format. cljs-time reads a weekday name in a pattern such as `EEE` and does not compare it with the date, so `1967-09-03 Fri` reads as 1967-09-03 and becomes `1967-09-03 Sun`.

The SDK's `DateFormat::read` compared the weekday with the date and returned `None` on a difference. `GraphIndex` also named a page without a file by the text a block wrote, so a date name kept its spelling.

## Goals / Non-Goals

**Goals:**

- A date name reads as its date, whatever weekday it names.
- A reference-only page that spells a date takes the title the journal title format gives the date, so the graph, the namespace list and the parser check match Logseq.
- `scripts/logseq-parser/check.py` reports no difference on `fixtures/wrong-weekday-date-name`.

**Non-Goals:**

- Names of pages that have a file. A journal file's title keeps coming from its file name or `title::` property.

## Decisions

- `DateFormat::read` still requires a weekday token to match a weekday name but no longer compares it with the date. The unused comparison goes; nothing else called for it, and a name that is no date still fails on the other tokens.
- `GraphSource::is_journal_name` becomes `journal_title: &dyn Fn(&str) -> Option<String>`. `Some(title)` says the name spells a date and gives the title for it. `Builder::name` in `graph.rs` swaps the name for that title before it builds the lookup key and the node, and marks the node as a journal. The SDK holds the relationship logic, and the publisher only supplies the date reader and the title format that it already has.
- The change applies to every date spelling, not only a wrong weekday, because Logseq converts every spelling. With the default title format the result is the same for `[[Oct 23rd, 2026]]`, and under `yyyy-MM-dd EEE` the spellings `Sep 3rd, 1967` and `1967-09-03 Fri` meet in one node.
- The comparison harness in `tests/relationships/mod.rs` maps a written date name to the same title before it records the names a block refers to, as the published resolver does through its page lookup.
- The parser check keeps `fixtures/wrong-weekday-date-name` as a fixture that must agree. The check's own test needs a fixture where Logseq and the SDK still differ. It moves to `fixtures/alias-spelling-case`, a page whose `alias:: [[homebrew]], [[Homebrew]]` Logseq spells `Homebrew` and the SDK spells `homebrew`. [Issue #509](https://github.com/codekiln/logseq-gardener/issues/509) tracks that difference; the encode garden shows it for `homebrew`, `codespace` and other aliases.

## Risks / Trade-offs

- [A reference-only page that was titled `Feb 1st, 2025` under another title format now has the configured format's title] → This is Logseq's behavior, and `[[Feb 1st, 2025]]` with a file still keeps the file's title.
- [`GraphSource` is a public struct] → The SDK is below 1.0, and the field has three callers in this repository.

## Resolved Questions

### 1 - Should the SDK keep rejecting a wrong weekday and only retitle the page?

No. The SDK rejected a wrong weekday because an earlier requirement assumed Logseq did. The parser check shows Logseq reads the date, so keeping the rejection would leave `[[2026-10-23 Mon]]` unresolved when the journal `2026-10-23 Fri` exists, which Logseq resolves.

## Open Questions

None.
