## Context

Gardener already creates navigation-only destinations from visible namespace prefixes. The publisher does not record that distinction in generated HTML. It also skips page properties during body rendering, including authored prev and next links. Gitpa preserves a separate public graph and must retain those properties before invoking Gardener.

## Goals / Non-Goals

**Goals:** Help readers identify namespace destinations and follow explicit page sequences while preserving publication privacy.

**Non-Goals:** Infer episode order from dates.

## Decisions

### Label generated parent pages as Namespace

Track whether a hierarchy node came from a selected source route. A generated destination receives a visible Namespace notice and badge, and its links use a dotted underline and accessible Namespace label. Keep index anchor text equal to the full logical title so downstream packaging continues to resolve routes. Place status text outside anchors. An unselected stored ancestor has the same presentation as an absent ancestor; neither status asserts whether a source file exists.

### Link Previous and Next only to published source pages

A separate sequence module reads the leading Properties block and uses the existing title/alias candidate index. Only one direct page reference is accepted. The last matching property supplies the value, matching existing alias precedence. Resolve to a canonical selected route and title; omit unsafe or ambiguous neighbors. Render a Page sequence nav after the body and before Hierarchy. Rendered sequence links add no backlinks or source routes; authored property references retain the existing Page properties backlink context.

### Render navigation separately from page content

Use separate page_status and sequence modules and focused tests. Site assembly adds the helper output without changing body rendering. The publisher places Linked references after Hierarchy. Generated status is recorded in hierarchy nodes; source name and embed indexes stay unchanged.

## Risks / Trade-offs

- Status text could alter downstream route discovery → Keep index anchor text unchanged and place Namespace badges outside anchors.
- Source existence could become visible → Classify only selected source routes and use the structural Namespace label.
- Private sequence targets could leak → Resolve only through the selected visible source index and emit nothing for invalid targets.

## Migration Plan

The shared publisher adds the navigation automatically. Consumers retain explicit prev and next properties when preparing their graph. No source file rewrite or route migration is required.

## Resolved Questions

### 1 - Does Gardener infer chronology?

No. Authors supply prev and next properties; Gardener follows those links safely.

### 2 - How does an unselected stored ancestor appear?

It has the same Namespace status as an ancestor without a source file.

## Open Questions

None.
