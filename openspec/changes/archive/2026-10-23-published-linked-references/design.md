## Context

The publisher already plans selected visible source documents and safe namespace routes before rendering. Its parser provides page and block reference identities; its renderer preserves source-relative assets, bounded embeds, authored disclosures, and a host-callable fragment helper. Linked references currently have no index or output.

Logseq 0.10.6 groups references by originating page, applies all include filters and any exclude filter to block path references, and uses a block-count threshold for initial collapse. Its parser excludes block UUID references from page path references. Gardener will label distinct-page and reference counts separately to meet the requested page-count behavior.

## Goals / Non-Goals

**Goals:** Show useful published-source context and visible counts, support related-page filters, honor graph disclosure settings, and preserve privacy and existing navigation.

**Non-Goals:** Editing the garden through reference controls; persisting per-page filter preferences back into source files.

## Decisions

### Build the index after visibility selection

Use only selected original documents, the existing source-name index, and already-approved namespace destinations. Index authored reference contexts before generating any navigation markup. This prevents generated links or unavailable source names from affecting results. Resolve aliases once to destination routes; routes also supply safe filter identities.

Each outline context includes its own line and attached properties/continuations. Page properties form a separately labelled context. References inside code/comments are omitted through parser reference extraction. Descendant content supplies useful copied context, while ancestor references supply filter identity. Pure block UUID references remain ordinary block navigation rather than page backlinks.

### Reuse source rendering for copied context

Add a reference rendering mode that suppresses original IDs, bounds work, and applies configured initial disclosure depth to copied outlines. Original blocks receive deterministic source-context anchors for navigation. Keep source-relative assets and existing embed safeguards. The original page rendering and authored collapse remain unchanged.

### Keep static content readable and enhance native controls

Render counts and all reference context in native details elements. Related-page selects offer Any, Include, and Exclude; all includes must match and any exclude rejects a context. Search narrows option labels only. Hide controls until the trusted script initializes them, so a no-JavaScript reader sees working disclosures and all results. The browser API `logseqGardenerInitBacklinks` initializes the initial document and supports explicit host initialization after content replacement, independently of fragment handling.

### Read Logseq’s reference collapse threshold and outline depth settings

Use a bounded EDN reader for top-level integer values, honoring comments, strings, nested forms, and discarded forms. Keep Logseq's defaults of threshold 100 and open depth 2. Treat unusable settings as defaults with diagnostics rather than changing public API options or adding a parser dependency.

## Risks / Trade-offs

- Private names could enter filter metadata → build options from resolved published identities only and test sentinels across every output file.
- Copies could duplicate IDs or lose asset context → reuse source rendering with explicit copy mode and original source paths.
- Deep references could expand output excessively → bound rendered context work and retain visible truncation diagnostics; preserve full index counts.
- Host page replacement can bypass script execution → expose idempotent initialization for the host to call after replacing content.

## Migration Plan

Regenerating a site adds the shared section, styles, and trusted initialization script. Existing source files and publisher call signatures remain valid. Reverting the feature restores prior output without source migration.

## Resolved Questions

### 1 - How do counts differ from Logseq's heading?

[Logseq's reference component](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/reference.cljs) counts direct referring blocks. Gardener shows separately labelled linking-page and reference counts, including filtered versus total values.

### 2 - Which references participate in filters?

[Logseq's filter predicates](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/handler/block.cljs#L253-L270) use current and ancestor page references: all included pages must occur and any excluded page rejects the block. Its related options also consider descendants. Gardener preserves those predicates for published identities.

### 3 - Which settings control initial collapse?

[Logseq's config readers](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/state.cljs#L538-L553) default to threshold 100 and open depth 2. [Its editor](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/handler/editor.cljs#L3774-L3787) combines depth-based collapse with authored collapse. Zero remains a meaningful integer.

## Open Questions

None.
