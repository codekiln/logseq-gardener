## Context

Logseq 0.10.6 preserves visible property order, renders page-reference values and hides its internal property keys plus configured `:block-hidden-properties`. Gardener currently skips properties in the body and offers a generic authored sequence region. This change adds a conservative opt-in publication surface.

## Goals / Non-Goals

Goals: useful linked metadata, native collapse, correct property source navigation and generic sequence presentation.

Non-goals: backlink card redesign and search, tracked separately.

## Decisions

Read top-level boolean `:publishing/page-properties?` (default false), `:publishing/page-properties-collapsed?` (default true), and `:publishing/page-sequence?` (default true) with the existing bounded config reader. Honor keyword collections in `:block-hidden-properties`. Existing sites retain their current display until explicitly enabled.

Render only the leading property block in authored order, keeping the last duplicate key. Omit Logseq control/internal keys, publication controls, file/source metadata and source-sync bookkeeping. Document the exact exclusions. Ordinary scalar values are escaped public metadata after opt-in. Plain text, direct page references and standalone public HTTPS values are supported. HTTPS links require a nonempty authority without credentials, whitespace or unsafe delimiters. Unsafe expressions, local/asset paths, unsafe schemes, block references and unavailable/ambiguous page targets omit the entire row, including its key. Mixed values require every reference to resolve to a selected visible source page. Preserve canonical target labels and routes.

Compute property views before backlink indexing. When enabled, property backlinks derive only from these safe visible rows. A property source link targets `#page-properties` only if the relevant visible row is present. Otherwise the existing source-page link remains honest about its destination. The Properties summary has that stable ID inside native details, so the existing ancestor reveal helper opens the section for direct and host navigation.

Prev/next rows use the existing generic direct-target validation. Visible property rows replace duplicate bottom sequence links; omitted rows leave valid enabled sequence navigation available. The graph can disable bottom sequence navigation entirely. Book and preset fixtures prove that shared behavior has no episode dependency.

## Risks / Trade-offs

- Newly displayed scalars can contain author metadata. Explicit opt-in, configured hidden keys and fixed internal exclusions bound that publication change.
- A mixed property loses useful public pieces if one target is unavailable. Whole-row omission prevents hidden identity leaks and misleading partial values.
- Configured hidden property references could still influence backlinks. The same safe property view supplies both display and enabled property indexing.

## Resolved Questions

### 1 - How do existing sites adopt properties?

Enable the graph setting explicitly. The section starts collapsed by default, and the existing sequence region remains the fallback for valid neighbors absent from visible rows.

### 2 - How does a property source link reveal its destination?

Use the fixed summary fragment inside details. Existing native/host fragment reveal opens its ancestor disclosure, and hosts can focus the returned summary.

## Open Questions

None.
