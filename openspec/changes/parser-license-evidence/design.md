## Context

The comparison runner tested [lsdoc 0.5.8 at the pinned revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d). Gardener and that candidate both declare AGPL-3.0-only. Gardener already generates notices from its own Cargo lockfile; the candidate has a separate locked dependency graph to inspect before adoption.

## Goals / Non-Goals

**Goals:** Record source identities, license declarations, actual license and attribution files, and the actions required when adding the parser to the SDK.

**Non-Goals:** Parser adoption, the Pengx corpus comparison, and release activation remain subsequent work.

## Decisions

### Audit the tested source revision

Extract the pinned committed source from the existing ghq checkout into a system temporary directory outside Gardener's Cargo workspace. Run Cargo metadata with its committed lockfile, then record package sources, checksums, declarations, and license-file hashes. This preserves the upstream checkout and keeps evidence independent of later working-tree edits.

### Review actual license files

Save the upstream license and attribution texts alongside the inventory. Preserve all supplied alternative-license texts; record MIT as the intended option for permissive dual-licensed packages and retain Unicode terms where the declaration requires them. Actual SDK adoption must regenerate Gardener's inventory from its resulting lockfile because dependency resolution can differ.

### Keep the adoption decision tied to compatibility evidence

The guide will explain the source and notice work for adopting the candidate. The maintainer will decide whether to adopt lsdoc after reviewing the remaining Pengx corpus comparison under [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3). Runtime mldoc and Logseq implementations remain research references with their own documented licenses.

## Risks / Trade-offs

The upstream lockfile may resolve different versions from the SDK's future lockfile → repeat notice generation and review on adoption.

An empty Cargo cache can prevent an offline audit → provide an explicit online option to fetch locked registry dependencies; reuse the existing ghq source in either mode.

## Resolved Questions

### 1 - Which parser revision should the license evidence cover?

Use the same revision already tested in the syntax comparison, so compatibility results and license evidence refer to the same source.

## Open Questions

None.
