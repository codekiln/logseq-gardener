# Linked-reference verification

## Automated coverage

`mise run ci` passed on 2026-10-23, including the complete Rust workspace, Clippy, formatting, the JavaScript suites, OpenSpec validation, RuleSync checks, documentation checks, release-impact fixtures, and distribution planning.

Eight SDK integration fixtures cover selected aliases and Unicode identity, duplicate references, literal code and block UUIDs, hidden-source sentinels across every output, property-only context, source anchors, repeated-copy IDs and media, safe namespace targets, ambiguous names, zero counts, threshold zero, reference-count thresholds, and copied/authored disclosure depth. Seven config unit fixtures cover top-level integers, misleading EDN forms, invalid input, and bounded reading. Eleven browser-script fixtures cover combined predicates, partial groups, search/reset, zero results, descendant paths, sibling isolation, disclosure preservation, replacement, and repeated initialization. Existing fragment-helper fixtures remain unchanged.

## Source comparison

The implementation follows the local Logseq 0.10.6 sources cited in the design: page-grouped authored references, current/ancestor include-all and exclude-any predicates, descendant-derived options, reference-count collapse thresholds, and integer disclosure defaults. Distinct linking-page counts are labelled separately from reference counts.

## Follow-up

[Sync and archive published linked references after implementation](https://github.com/codekiln/logseq-gardener/issues/79) is a verified native child of the implementation issue. Cleanup begins after implementation merges into main.
