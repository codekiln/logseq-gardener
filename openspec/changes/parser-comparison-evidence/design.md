## Context

The merged mldoc baseline contains synthetic fixtures and raw syntax output. The candidate lsdoc repository supplies a normalized differential harness. graph-validator's default branch now targets database graphs; its file-compatible predecessor pins Logseq 0.9.8.

## Goals / Non-Goals

**Goals:** Reproduce syntax and relationship comparisons, record classified disagreements and performance, and explain the implications for the SDK.

**Non-Goals:** Parser adoption, source-editing commands, and the first read-only garden command.

## Decisions

Use upstream comparison tooling at pinned revisions and build lsdoc from source. Record its normalization exclusions alongside results so syntax agreement has a stated meaning. Retain the existing shared fixtures and add focused cases when a missing semantic question warrants one.

Run Logseq's file parser as a separate relationship reference. Use graph-validator's last file-compatible first-parent revision and its declared parser revision. Record graph identities and relationships with generated UUIDs and machine paths removed from the comparison.

Run available public gardens locally. Save reproducible minimized disagreements and corpus hashes; keep timings separate from deterministic snapshots. Measure peak process memory separately from parser-reported timings.

## Risks / Trade-offs

- Normalization can hide source-location errors → exercise source ranges separately and qualify source-preservation claims.
- Public gardens change → record source revisions and input hashes.
- Historical graph-validator differs from newer Logseq file behavior → identify the tested revision and retain this limitation in the recommendation.

## Resolved Questions

### 1 - Which validator supports Markdown gardens?

Use the first parent of the database migration merge, then check its pinned file parser. The default branch validates database graphs.

### 2 - What does syntax agreement establish?

Agreement covers the upstream normalized syntax and reference projection. Alias resolution, file identity, hierarchy, and source preservation need separate evidence.

## Open Questions

None for running the experiment. Parser adoption depends on the recorded results.
