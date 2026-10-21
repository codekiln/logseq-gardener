## Context

The publisher already selects visible pages, indexes aliases and explicit outline UUIDs, and copies referenced assets. Its renderer currently treats embeds and queries as unsupported syntax. The published root is a generated page index. Gitpa's existing homepage uses embeds, media, and Logseq query results, so replacing its renderer requires both reusable publisher features and Gitpa's application-specific query bridge.

## Goals / Non-Goals

**Goals:** Render selected page and block embeds safely; expose a configurable homepage through the SDK and CLI; preserve ordinary page navigation, media playback, and source gardens; verify Gitpa's homepage against its existing published appearance.

**Non-Goals:** A general Logseq Datalog engine inside the Rust publisher. Gitpa evaluates its actual queries with nbb-logseq before invoking the publisher.

## Decisions

### Visibility and lookup

Build transclusion lookup from the same selected visible documents used for links. Resolve page titles and aliases with the existing page-name index. Resolve block embeds through unique valid outline UUIDs and retain the target outline's descendants. Missing, ambiguous, excluded, and private targets keep escaped source fallbacks with local diagnostics. Lookup over all loaded documents would expose withheld content and is unsuitable for publication.

### Expansion and assets

Render embedded content with its original document as the source for asset paths. Track the active expansion chain and impose a fixed expansion budget so cycles and large repeated embeds terminate. Embedded copies omit source target IDs or assign unique copy IDs; ordinary block references continue to navigate to the target's original generated page. Copying raw rendered HTML would confuse relative assets and duplicate IDs, so reuse rendering with explicit source context.

### Homepage

Add an optional logical page selector to SiteOptions and `--homepage` to publish. Resolve it against selected visible titles and aliases before writing output. A missing or ambiguous selector fails publication. Keep the selected page's ordinary route, render its content at index.html, and expose the complete page list at pages.html. All generated pages retain a link to the page list. Without a selector, retain the existing index homepage.

### Query ownership

The Gitpa bridge evaluates existing Logseq Datalog queries with nbb-logseq and supplies the rendered source to the SDK. The generic publisher retains escaped fallbacks and diagnostics for arbitrary queries it receives. A query-specific Rust implementation would duplicate Logseq semantics and increase the migration's scope.

## Risks / Trade-offs

- Recursive or repeated embeds can exhaust rendering resources → bound expansion and test cycles, repetition, and fallback diagnostics.
- Embedded assets can resolve against the wrong page → retain original source context through every expansion.
- Repeated content can duplicate HTML IDs → test unique IDs and links to original target anchors.
- A visually similar homepage can still lose queries or media → compare the generated Gitpa homepage in a browser and exercise its audio, images, navigation, query results, and embeds before switching deployment.

## Migration Plan

Release the SDK and CLI changes with focused rendering and homepage tests. Update Gitpa's bridge, theme, and build workflow to consume that release and generate a candidate site. Compare the homepage with the current published site, then switch the deployment after the comparison succeeds. The previous workflow and renderer revision remain available for rollback.

## Resolved Questions

### 1 - Where are Logseq queries evaluated?

Gitpa evaluates its existing Datalog queries through nbb-logseq before publication. The Rust publisher provides generic selected-content transclusion and homepage selection.

### 2 - How does a selected homepage affect page navigation?

The selected page appears at index.html and its ordinary page route. The complete page list remains available at pages.html.

## Open Questions

None.
