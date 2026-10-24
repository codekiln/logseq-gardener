## Context

[Logseq 0.10.6 reference rendering](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/reference.cljs) groups blocks by source page and requests breadcrumbs. The supplied published Voidom screenshot shows shaded source groups, short ancestor paths and referring content. The current Gitpa examples repeat a source-page heading, an “Open source” line and target links; leading properties display a generic label rather than the actual safe fields.

## Goals / Non-Goals

Goals: compact source groups, useful original-source navigation and one displayed copy of overlapping outlines with unchanged authored context accounting.

Non-goals: search and timestamp seeking.

## Decisions

Reuse validated property rows from the property renderer. Include only rows whose selected target set contains the viewed page. A property disclosure link uses the existing interior fragment target.

Render the outermost referring subtree once. Mark each nested genuine reference with its own entry attributes and source control. The controller includes a nested entry's own node when evaluating its subtree, retaining the same ancestor/descendant matching predicates. Distinct source-page and genuine context totals remain unchanged.

Ancestor labels link to original source offsets. Every source control has a keyboard-readable label. Cards use existing theme variables and wrap long paths.

## Risks / Trade-offs

Nested entries share DOM descendants. Focused controller fixtures check child-only matches, ancestor retention, exclusion and reset. Shared property validation supplies the privacy boundary for snippets.

## Resolved Questions

### 1 - How are overlapping references displayed?

Each source subtree appears once, with nested authored references retaining their own context identity and source link.

## Open Questions

None.
