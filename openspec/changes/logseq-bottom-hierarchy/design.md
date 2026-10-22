## Context

The current publisher emits top breadcrumbs and an immediate-child Hierarchy list. Logseq's inspected hierarchy component renders a flat list of complete namespace paths, with each segment linking to its cumulative prefix. Its hierarchy data includes recursive descendants, and the whole section is foldable and expanded by default. The page component places it below source content and references.

The requested behavior also includes the current full path as a row, including on a namespaced leaf. This provides ancestor links at the bottom of the leaf page. [The source reference](references/logseq-hierarchy.md) records the inspected implementation and this explicit current-row behavior.

## Goals / Non-Goals

**Goals:** Render the bottom hierarchy as full paths across every descendant level; link each available prefix segment; retain expanded-by-default disclosure, mobile readability, and existing publication protections.

**Non-Goals:** None.

## Decisions

### Render complete paths in a flat bottom list

Replace the top breadcrumb output with one Hierarchy section after rendered source content. Emit the current page's complete path first, then every navigable descendant destination in normalized logical-title order. Keep rows flat and visible when the section is open. The example at A/B/C contains A/B/C, A/B/C/D, A/B/C/E, and A/B/C/E/F; E/F remains visible alongside the other rows.

Use the existing safe hierarchy destination model, including generated intermediate destinations. A namespace-prefix boundary determines ancestry, so A/B/Cousin is outside A/B/C. Aliases supply no additional descendants.

### Link every available segment to its prefix

Render each path segment as a separate ordinary link to its cumulative destination, separated by slash text. The current row's final segment remains a link and can carry aria-current. Existing unavailable private or alias-conflicting destinations remain plain segment text. A descendant beyond a protected gap still has its full row and available links; its hidden ancestor gains no standalone destination.

### Make disclosure native and accessible

Use an expanded-by-default section disclosure with a labelled Hierarchy summary. Use native `<details open>` and `<summary>Hierarchy</summary>` within the labelled Hierarchy section to provide keyboard operation without new JavaScript. Keep the section's flat path list readable at narrow widths with wrapping and visible link focus. Each row is a path, so depth does not require nested list indentation or an individual collapse control.

### Preserve publication boundaries

Retain the selected-visible destination model, private-title suppression before selection, alias collision protection, deterministic routes/indexing, source-only lookup and homepage validation, and source-page counts. Change the presentation and descendant traversal; reuse existing source rendering and assets.

## Risks / Trade-offs

- A descendant traversal can omit deeper levels → test the explicit E/F row and a larger branch at multiple depths.
- Prefix matching can include similarly named siblings → require slash boundaries and test C versus Cousin.
- More path links can accidentally recreate protected ancestor routes → test private and alias-conflicting segments as plain text across descendant rows.
- Long full paths can overflow on mobile → inspect a narrow viewport and test wrap/focus styles.

## Migration Plan

Update the SDK renderer, tests, and documentation, then generate a Gitpa preview with matching presentation styles. Inspect the explicit path example, a namespaced leaf, a multilevel session collection, and mobile navigation. Run the full repository checks before merging and deploying the renderer pin update. After implementation merges, synchronize the delta and archive this change in a separate cleanup PR.

## Resolved Questions

### 1 - Does recursive hierarchy data require a nested tree display?

The Logseq component renders a flat list of full paths. Every descendant level is visible when the section is open.

### 2 - Which row supplies ancestor navigation on the current page?

The current full path is the first row, including on namespaced leaves. Each available segment links to its cumulative namespace destination.

## Open Questions

None.
