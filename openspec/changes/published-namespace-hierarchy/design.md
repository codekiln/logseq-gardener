## Context

The publisher generates routes and an index for selected visible source documents. A page's slash-separated namespace appears as its title, but readers cannot follow its ancestors or browse its children. Some intermediate namespaces exist only as prefixes, while others have source files outside publication selection. Gitpa reads full logical titles and routes from the generated page index to prepare its public URLs.

The inspected Auster leaf contains Notes. Its surrounding namespace needs navigation; the leaf's content does not need reconstruction.

## Goals / Non-Goals

**Goals:** Provide linked ancestors and child navigation; generate safe intermediate namespace destinations; preserve source rendering, visibility, routes, aliases, homepage selection, and source-page reporting; support narrow screens and keyboard navigation.

**Non-Goals:** None.

## Decisions

### Build the hierarchy from published titles

After applying selection and visibility checks, derive namespace prefixes from the selected visible original titles. Use the existing normalized title keys to share prefixes, prefer the spelling of an actual selected source page, and choose remaining display spellings deterministically. Aliases contribute no namespace nodes. Load visibility classification for source titles before selection so an explicitly private ancestor outside the include roots remains protected.

A missing or ordinarily unselected ancestor can become a navigation-only page using the prefix already present in a published title. Its source body, aliases, properties, and assets remain unavailable. A private or otherwise visibility-withheld stored ancestor receives no synthesized destination. A synthesized prefix that collides with a selected alias also receives no destination. These gaps retain plain breadcrumb text and connect descendants to the nearest available ancestor for child navigation.

### Keep navigation separate from source lookup

Use a hierarchy route map for breadcrumbs and child navigation. Keep the existing selected-source page index for ordinary source links, aliases, transclusions, and homepage selection. Adding generated namespaces to that index would change alias ambiguity and make a structural destination appear to have source content. Existing explicit source routes remain stable, and route/name collisions fail before output creation.

### Publish navigation destinations in the index

Render generated namespace pages with the normal HTML shell and child navigation. List source and generated destinations once each in the full page index, using their full logical title and ordinary local route shape so Gitpa can package them. Keep SiteReport.pages as the count of selected source documents; generated navigation pages and duplicate homepage output remain outside that count.

### Show ancestors and children

Place a labelled breadcrumb navigation before page content. Ancestors with destinations are links; the current page is identified as current and withheld gaps are plain text. Add a Hierarchy section when navigable children exist. Each child is linked once in deterministic order; when an unavailable ancestor creates a gap, show the remaining relative path so the destination is clear. Use escaped labels and semantic HTML, with wrapping that keeps long names and link focus visible on mobile screens.

## Risks / Trade-offs

- An unselected private ancestor could be recreated as a navigation page → classify stored title visibility before selection and suppress its generated destination.
- Generated prefixes could alter alias lookup or expose source content → keep navigation routes separate and test alias collisions and embed/homepage behavior.
- More output files could change existing page reports → preserve source-document counting and test index contents separately.
- Deep paths can overwhelm narrow screens → wrap breadcrumb and child labels and inspect a mobile viewport.

## Migration Plan

Add the hierarchy model and rendering to the SDK, then update publishing tests and documentation. Generate a Gitpa preview with the new renderer and inspect ancestor links, intermediate pages, and child navigation around the Auster leaf. Preserve the existing renderer pin until the preview and repository checks pass. After the implementation merges, synchronize every delta and archive this change in a separate cleanup PR.

## Resolved Questions

### 1 - Can an ancestor outside the include roots have a generated destination?

An ordinarily unselected ancestor can have a navigation-only destination derived from its selected visible descendant's title. A stored private or visibility-withheld ancestor has no generated destination, even outside the include roots.

### 2 - Do generated namespace pages change source lookup or page counts?

Generated destinations appear in the full page index and hierarchy navigation. Source links, aliases, transclusions, homepage lookup, and SiteReport.pages retain their selected-source behavior.

## Open Questions

None.
