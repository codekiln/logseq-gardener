## Why

Logseq accepts the weekday token `E` in `:journal/page-title-format`, and its settings screen offers `E, MM/dd/yyyy`, `E, dd-MM-yyyy`, `E, dd.MM.yyyy` and `E, yyyy/MM/dd`. The SDK rejects `E`, falls back to `MMM do, yyyy`, and so gives a garden that uses one of these formats the wrong journal titles and leaves links to its journals unresolved ([Issue #269](https://github.com/codekiln/logseq-gardener/issues/269)).

## What Changes

- Accept `E` in journal date formats. It writes the short weekday name, as `EEE` does.
- When reading a date, accept the full or short weekday name for `E`, `EEE` and `EEEE`, as Logseq does.
- Keep `EE` and five or more `E`s unsupported: Logseq's date library has no such pattern.

## Capabilities

### New Capabilities

### Modified Capabilities

- `journal-identity`: The supported tokens include `E`, and a weekday token reads either name length.

## Impact

The SDK's `journals` module (`DateFormat`) and the journal-format check in garden settings. Gardens that set `:journal/page-title-format` to a pattern with `E` get Logseq's journal titles and resolved journal links. Gardens using other formats, including the encode garden's `yyyy-MM-dd EEE`, publish the same titles as before. The limitation note in `docs/parser-adoption.md` is removed.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
