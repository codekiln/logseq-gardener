## Why

Logseq reads `[[a/b/]]` as the page `a/b` with the namespace parent `a`. The SDK keeps the trailing slash: its page is titled `a/b/` and has no namespace ([Issue #495 — Drop the trailing slash of a page name, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/495)). A leading slash is the same case: the encode garden's `pages/Ableton___Live___UG___29 Audio Effect Reference___39 Tuner___04 Note Spellings.md` writes `/D♭)`, which Logseq reads as `D♭)` and the SDK reads as `/D♭)`.

The cause is the spelling of a written name. Logseq's `remove-boundary-slashes` drops one slash from each end of a page name before it makes the page, the title and the namespace. The SDK removed those slashes only from lookup keys, so the title and the namespace parents of a reference-only page kept them.

## What Changes

- A page name a block writes loses one slash at its start and one at its end, as in Logseq. `[[a/b/]]` is the reference-only page `a/b` with the namespace parent `a`, and `[[/D♭)]]` is `D♭)`.
- The parser check's deliberate mismatch moves from `fixtures/trailing-slash-name`, which now agrees, to `fixtures/wrong-weekday-date-name`: `[[1967-09-03 Fri]]`, which Logseq reads as the date `1967-09-03 sun` and the SDK keeps as written. [Issue #503](https://github.com/codekiln/logseq-gardener/issues/503) tracks that difference. `trailing-slash-name` stays as a fixture that must agree.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-graph`: adds the requirement "Name a page without its boundary slashes".

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: `remove_boundary_slashes` is new.
- `crates/logseq-gardener-sdk/src/references.rs`: `Contribution::from_document` applies it to every written name before it finds namespace parents or resolves the name.
- The publisher's code is unchanged. A page that links `a/b/` now shows `a/b` in its graph and namespace list, and no page changes otherwise.
- `crates/logseq-gardener-sdk/tests/`: the fixture `trailing-slash-name` and a test in `og_relationships.rs`.
- `scripts/logseq-parser/`: the fixture `wrong-weekday-date-name` and its test.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
