## Context

Issue94 publishes a selected-source manifest/corpus/evaluator with expected revision metadata in each page. The existing shell supplies a shared header, delegated controls and theme variables. Gitpa replaces main content while keeping its audio player alive.

## Goals / Non-Goals

**Goals:** accessible shared search, bounded lazy loading, clear failure states, stable source links and idempotent persistent-navigation integration.

**Non-Goals:** representative performance measurements and worker/chunking selection (issue96).

## Decisions

Add a hidden labelled search button to the existing header. A small fingerprinted module reveals it only when native dialog support and valid publication metadata are available. Create one native dialog outside main on first use. Native modal focus containment handles Tab; arrow keys move through real result links and Enter activates the selected result. Escape/close restores the invoking control. Results use text nodes and retain ordinary modified/new-tab link behavior.

Keep the loader independently testable through injected fetch/import functions. On first open, fetch the manifest then corpus, verify same-origin artifact paths, expected revision, response status and serialized UTF-8 length, and use the existing pure evaluator validation. Import its fingerprinted module only after manifest validation. Cache a prepared publication by identity. Abort active requests and discard stale generations on close, retry or metadata changes; synchronous query callbacks also check current generation.

Expose `window.logseqGardenerInitSearch(root=document, {resolvePage}={})`. Read expected metadata from the current document head on every call. A changed/missing identity invalidates pending/cached data and controls. Persist the body dialog and one set of document listeners across main replacement. Default result URLs resolve beside the manifest, including nested pages; a host resolver supplies readable routes using canonical titles. Close before an ordinary result click bubbles to the host navigator.

Errors offer explicit retry and reload controls. Publication mismatch never automatically reloads the host or interrupts its player. Dialog styles use shared appearance variables and a bounded mobile viewport.

## Risks / Trade-offs

- Static artifact deployment can be incomplete → show retry/reload while ordinary page links remain usable.
- A host can forget to update revision metadata → document and test the reinitialization sequence.
- Synchronous whole-corpus evaluation may exceed a measured phone budget → issue96 measures the existing parity harness before choosing workers or chunks.

## Migration Plan

Search-enabled publication adds the controller and its hidden trigger. Hosts call the initializer after updating page metadata and replacing content. Existing opt-out publication retains ordinary navigation. Rebuild without search to remove controller artifacts.

## Resolved Questions

### 1 - Where does the dialog live?

One body-level native dialog persists outside main. Hosts replace main normally and update search metadata before reinitializing.

### 2 - How do results navigate?

Result anchors keep browser defaults and host delegated navigation. The resolver adapts only the page route; existing source fragments and reveal behavior remain intact.

## Open Questions

None.
