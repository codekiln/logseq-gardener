## 1. Hierarchy model

- [x] 1.1 Classify stored source-title visibility before selection and derive deterministic namespace prefixes from selected visible original titles, reusing selected source routes.
- [x] 1.2 Generate safe missing or ordinarily unselected ancestor destinations, suppress private/withheld titles and selected-alias collisions, and keep hierarchy routes separate from source lookup.
- [x] 1.3 Include full-title synthetic destinations in the complete page index while preserving source-page counts, route collision checks, homepage selection, transclusions, and media.

## 2. Navigation

- [x] 2.1 Render labelled breadcrumbs, identified current pages, and Hierarchy child links, connecting across unavailable ancestor gaps with clear relative labels.
- [x] 2.2 Add semantic markup and wrapping styles that retain keyboard access and visible focus for long names on mobile screens; update local publishing documentation.

## 3. Verification

- [x] 3.1 Test missing and ordinarily unselected ancestors, private ancestors outside include roots, nested privacy, excluded branches, alias collisions, normalized prefix sharing, deterministic output, title overrides, counts, and preserved source lookup/rendering.
- [x] 3.2 Run repository checks and inspect a Gitpa preview around the Auster leaf at desktop and mobile widths, recording parent links, intermediate namespace destinations, child links, and retained Notes content.
- [x] 3.3 Review the artifacts and implementation against requirements and writing preferences; create the cleanup child and link the implementation PR before merge.
