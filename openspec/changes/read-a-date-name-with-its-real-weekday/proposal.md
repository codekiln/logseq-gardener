## Why

Logseq reads a page reference that spells a date in the journal title format as that date, and it spells the page with the weekday the date has. With `:journal/page-title-format "yyyy-MM-dd EEE"`, `[[1967-09-03 Fri]]` is the page `1967-09-03 Sun`, because 1967-09-03 was a Sunday. The SDK keeps the name as written, `1967-09-03 Fri` ([Issue #503 — Read a date name with a wrong weekday as Logseq does](https://github.com/codekiln/logseq-gardener/issues/503)). The encode garden has four such references, for example `date-created:: [[1967-09-03 Fri]]` in `pages/Person___Daron Acemoglu.md`.

Two causes:

- `DateFormat::read` rejects a date whose weekday name differs from the date's weekday. Logseq's cljs-time parser reads the weekday name and ignores it, so Logseq reads the date and the SDK does not.
- A date name that matches no journal file becomes a reference-only page titled as written. Logseq's `convert-page-if-journal` instead rewrites the name to the date in the journal title format, whatever the written spelling.

## What Changes

- A weekday name in a date is read but not checked against the date. `2026-10-23 Mon` reads as 2026-10-23, so the reference finds the journal for that date when one exists.
- A reference-only page that spells a date takes the title the journal title format gives that date. `[[1967-09-03 Fri]]` is the page `1967-09-03 Sun`, and no page is titled `1967-09-03 Fri`.
- The parser check's deliberate mismatch moves from `fixtures/wrong-weekday-date-name`, which now agrees, to `fixtures/alias-spelling-case`: `alias:: [[homebrew]], [[Homebrew]]`, which Logseq spells `Homebrew` and the SDK spells `homebrew`. [Issue #509](https://github.com/codekiln/logseq-gardener/issues/509) tracks that difference. `wrong-weekday-date-name` stays as a fixture that must agree.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `journal-identity`: the requirement "Recognise journal references in Logseq's date formats" no longer rejects a date with a wrong weekday.
- `garden-graph`: adds the requirement "Title a page that names a date with the date's own weekday".

## Impact

- `crates/logseq-gardener-sdk/src/journals.rs`: `DateFormat::read` does not compare the weekday with the date.
- `crates/logseq-gardener-sdk/src/graph.rs`: `GraphSource::is_journal_name` becomes `GraphSource::journal_title`, which returns the title Logseq gives a date name. This is a public field, so a caller that builds a `GraphSource` changes its closure.
- `crates/logseq-gardener-sdk/src/site/mod.rs`: the publisher passes the date's title in the journal title format. The publisher gains no relationship logic.
- `scripts/logseq-parser/`: the fixture `alias-spelling-case` and its test.
- `crates/logseq-gardener-sdk/tests/`: the fixture `wrong-weekday-date-name` and a test in `og_relationships.rs`; the comparison harness maps date names the same way.
- A published page that links a date with a wrong weekday shows the real weekday in its graph.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
