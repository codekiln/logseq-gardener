## Why

`lsg publish` shows Logseq's "Invalid property names" warning on a block whose property names Logseq rejects, but its report lists no diagnostic for that block ([Issue #474 — Report rejected property names as a publish diagnostic](https://github.com/codekiln/logseq-gardener/issues/474)). A garden's build therefore passes while its site shows the warning. The encode garden published 14 such blocks on 12 pages, and gitpa 3 of them, until [codekiln/logseq-encode-garden#315 — fix: start property examples with prose](https://github.com/codekiln/logseq-encode-garden/pull/315) and [codekiln/gitpa#143 — feat: re-sync entity pages with prose-led property examples](https://github.com/codekiln/gitpa/pull/143) rewrote them.

Both gardens publish with `lsg` in CI, and the encode garden already fails its build on one diagnostic, `shallow clone`. With a diagnostic for rejected property names, each garden can fail its build on those blocks too.

## What Changes

- Each properties block that shows the warning adds one diagnostic, `Invalid property names: <names>`, with the names that the warning's buttons show, separated by `, `.
- A block shown in an embed, Linked References or query results reports under its own file, once, as other diagnostics of copied blocks do.
- The exit status and the published files stay the same.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-outline`: a new requirement, "Report rejected property names as a diagnostic", says when the publisher reports the diagnostic and what it says.

## Impact

- `crates/logseq-gardener-sdk/src/site/render.rs`: `Renderer::invalid_properties` records the diagnostic where it renders the warning.
- `SiteReport::diagnostics` and the `lsg publish` report: one more entry for each properties block with rejected names. The encode garden at `cbab08523e` gains 14.
- `docs/pages/lsg___Ref___Logseq Compatibility___Page Outline.md`: lists the diagnostic under "Differences from Logseq".
- Published files: unchanged.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don%27t%20Repeat%20Yourself%20DRY.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
