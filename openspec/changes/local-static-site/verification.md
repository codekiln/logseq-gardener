# Local publishing demonstration

The initial local workflow generated and displayed readable HTML from the current [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden) working snapshot. The checkout was based on commit `e12f9568693662aee21c0c61b9787a767a56d4e9` with existing user changes; these results describe that snapshot rather than the commit alone.

## Namespace example

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example publish_site -- \
  /path/to/logseq-encode-garden /private/tmp/logseq-gardener-demo-20261007 \
  triple-lowbar --include My/AI --exclude My/AI/Agent
```

The run generated a Garden index and selected My/AI pages, with My/AI/Agent excluded. The index opened through a loopback-only HTTP preview in the in-app browser. Inspection covered readable headings and nested outlines in the Words and Phrases to Avoid page; its selected-page link opened Be like the holograms Data from ST:TNG conferred with. The Garden index navigation returned to the index.

```text
Published 30 pages and 0 assets; withheld 0 pages; skipped 488 journals.
77 publication diagnostics.
```

The selected namespace contains references beyond its selection; those labels remain visible without links to excluded output. No transclusion occurred. A snapshot of all visible ordinary files under pages, journals, and assets matched before and after generation. The real demonstration included no copied assets; asset behavior also has separate fixture and real-image checks below.

Browser inspection used HTTP on loopback. Direct `file:` navigation could not be tested because the automation browser blocks that protocol. Generated HTML uses relative routes and embedded styling; the guide also explains how a user can open the saved index themselves.

## Asset example

An existing Logseq documentation note referenced asset files that were absent from this garden. The publisher reported those missing assets and generated readable labels, which is the expected fallback. A separate public workshop page with an existing local image exercises copying and browser display:

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example publish_site -- \
  /path/to/logseq-encode-garden /private/tmp/logseq-gardener-image-demo-20261007 \
  triple-lowbar --include 'AI/ES/25/ws/1/Building Agents with Model Context Protocol'
```

The workshop run generated three pages and copied ten assets. Browser inspection confirmed all ten images loaded with nonzero natural widths. The publisher reported 490 diagnostics for references and unsupported constructs outside this small selection. The complete source snapshot remained unchanged after both demonstrations.

## Fixture verification

Integration tests generate synthetic gardens with excluded page text, excluded block text, excluded-only assets, selected private pages, and selected public content. Every generated file is inspected for excluded sentinel content. Selected image copying, navigation, deterministic routes, source preservation, existing-output refusal, source-descendant refusal, ambiguous titles, and unsafe asset paths are checked.

Renderer tests cover outlines, headings, code, selected and excluded links, raw HTML and unsafe URL escaping, unsupported macros and block references, nested-label asset suppression, depth limits, and non-image assets used with image syntax. Asset tests cover decoding, traversal, Unicode paths, unsupported extensions, and directory/leaf symlinks.

The initial whole-file private-property policy withholds more than a private child alone. Complete aliases, journal dates, precise block visibility, query execution, and embed expansion remain product work. The demonstrated capability is local selected-page HTML generation; complete replacement of Logseq publishing requires those additional decisions and feedback from trying the command.

The complete `mise run ci` workflow passed after the final rendering changes, including SDK tests, formatting, linting, CLI checks, and existing distribution consistency checks. No release or deployment was performed.
