## Context

One reusable disclosure button currently moves to the nearest block on mouseover. Nested gutter hit-testing can move it back to an ancestor before the reader reaches the button. Heading bullets also use different vertical offsets while the arrow keeps one offset.

## Goals / Non-Goals

**Goals:** Stable ownership, larger aligned targets, visible closed controls, keyboard/touch activation, and safe replacement initialization.

**Non-Goals:** None.

## Decisions

### Give every expandable block its own button

Create one button per owning outline block during initialization. Keep delegated activation but resolve the clicked button's own parent rather than a shared movable button. Native buttons supply keyboard and touch activation. Leaf blocks receive none. Existing native authored disclosures retain their markup and behavior.

### Share the bullet's vertical offset

Use the same CSS offset for an owning bullet and its arrow, with a 24px target centered beside the 14px bullet column. Closed, focused and touch controls remain visible; expanded mouse controls can appear on their own row's hover. Each target stays attached to its block throughout pointer movement.

### Initialize replacement content explicitly

Expose idempotent window.logseqGardenerInitOutline(root=document). Initialize existing outlines before applying the current zoom route. Hosts can initialize an imported main before focus and route hooks. A narrow child-list observer covers later journal batches and hosts that insert content without the explicit hook; button insertion does not trigger another scan. Native toggle events refresh arrow state, including when fragment reveal opens an authored disclosure. Zoom and unzoom refresh states without recreating controls.

## Risks / Trade-offs

- One control per expandable block adds DOM nodes; retain the existing 8KB controller and page-performance budgets and avoid repeated whole-document scans for inserted content.
- Heading bullets and disclosure buttons can become misaligned. Use their shared vertical offset and check their positions at desktop and phone widths.
- Native summary activation and fragment reveal change state independently; synchronize accessible state through toggle events.

## Resolved Questions

### 1 - Should the arrow move between hovered rows?

Each expandable block keeps its own control. Hover affects appearance only.

### 2 - How do hosts initialize replaced content before focusing it?

Call window.logseqGardenerInitOutline(importedMain). Repeated initialization preserves the existing buttons and collapse state.

## Open Questions

None.
