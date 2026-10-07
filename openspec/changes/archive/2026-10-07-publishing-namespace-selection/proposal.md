## Why

The user wants to publish selected Logseq namespaces through a Rust static site workflow by the October 10–11 weekend. A shared namespace-selection policy gives the future publisher a testable foundation for deciding which pages can contribute content.

## What Changes

- Add an SDK namespace policy with explicit inclusion, descendant matching, and exclusion precedence.
- Validate selectors and reject malformed page identities during matching.
- Demonstrate the policy in an SDK example and record the publishing milestone and follow-ups.

## Capabilities

### New Capabilities

- `publishing-namespace-selection`: Select logical page identities through explicit namespace roots and exclusions.

### Modified Capabilities

None.

## Impact

Adds a dependency-free SDK module and example. The policy supports [Issue #12 — Namespace selection](https://github.com/codekiln/logseq-gardener/issues/12), under [Issue #11 — Rust static site publishing](https://github.com/codekiln/logseq-gardener/issues/11). Garden loading and rendering will consume the policy in subsequent changes.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md): keep selection independently testable while the parser decision proceeds.
