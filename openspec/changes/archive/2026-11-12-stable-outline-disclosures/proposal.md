## Why

The disclosure arrow moves between blocks as the pointer crosses nested gutters, making some children unreachable. Its small hover-only target is also difficult to use on headings, keyboards and touch screens.

## What Changes

- Give each expandable block its own stable, focusable disclosure control.
- Use a target at least 24px square, centered on the owning bullet for plain blocks and headings.
- Keep closed, focused and touch controls visible before hover; retain native authored disclosures without JavaScript.
- Initialize replacement outlines without changing zoom, fragment navigation or source content.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-outline`: Stable disclosure ownership, accessible targets and initialization of replacement content.

## Impact

The shared outline controller, styles, tests and publication guide change. Existing source markup and host navigation contracts remain in place. No dependencies are added.

## Citations

- [Make the block disclosure arrow larger, visible on collapsed blocks and aligned with its bullet](https://github.com/codekiln/logseq-gardener/issues/167)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
