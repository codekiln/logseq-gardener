# Linked-reference verification

## Automated coverage

`mise run ci` passed on 2026-10-23, including the complete Rust workspace, Clippy, formatting, the JavaScript suites, OpenSpec validation, RuleSync checks, documentation checks, release-impact fixtures, and distribution planning.

Eight SDK integration fixtures cover selected aliases and Unicode identity, duplicate references, literal code and block UUIDs, hidden-source sentinels across every output, property-only context, source anchors, repeated-copy IDs and media, safe namespace targets, ambiguous names, zero counts, threshold zero, reference-count thresholds, and copied/authored disclosure depth. Seven config unit fixtures cover top-level integers, misleading EDN forms, invalid input, and bounded reading. Eleven browser-script fixtures cover combined predicates, partial groups, search/reset, zero results, descendant paths, sibling isolation, disclosure preservation, replacement, and repeated initialization. Existing fragment-helper fixtures remain unchanged.

## Source comparison

The implementation follows the local Logseq 0.10.6 sources cited in the design: page-grouped authored references, current/ancestor include-all and exclude-any predicates, descendant-derived options, reference-count collapse thresholds, and integer disclosure defaults. Distinct linking-page counts are labelled separately from reference counts.

## Browser and independent review

The real Gitpa preview showed three linking pages and three references on Voidom. Excluding the October 22 source retained two; including it retained one; reset restored all three. Search narrowed related-page choices. Keyboard Enter toggled the native section while counts stayed visible. At 390px the controls and context wrapped without horizontal overflow. Static HTML retains counts, source context, and native disclosures without JavaScript; replacement and repeated initialization are covered by the script fixtures. Gitpa's host adapter receives separate integration checks when it consumes this renderer.

Independent privacy/security review at implementation commit `d711bf91500aaf786032de7db33d9470ce5f0bc5` found no actionable issues and reran all eight integration, seven config, and eleven filter-script fixtures successfully. The review checked selected-source boundaries, hidden identities and metadata, copied IDs and asset origin, escaped controls and trusted script, and bounded non-evaluating config reading.

## Follow-up

[Sync and archive published linked references after implementation](https://github.com/codekiln/logseq-gardener/issues/79) is a verified native child of the implementation issue. Cleanup begins after implementation merges into main.
