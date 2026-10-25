## Context

The publisher already selects visible source documents, resolves names and original block identities, and produces shared fragment reveal helpers. Logseq 0.10.6 separates page-name search from block-content search. Static publication can reuse the selected-source boundary while exposing a smaller portable contract.

## Goals / Non-Goals

**Goals:** deterministic opt-in search data, original-source destinations, safe text projection, pure bounded evaluation and host integration seams.

**Non-Goals:** accessible dialog (issue95), browser loading/performance validation (issue96), property-content search, generated namespace title search.

## Decisions

Use selected stored pages and unique aliases. Alias claims that resolve ambiguously contribute no match keys. Preserve authored display spelling. Index original AST text after privacy selection; references contribute text only when their destination resolves uniquely to a selected page. Macros, property values, raw HTML, copied embed text, asset metadata and unsupported constructs contribute no search text.

Keep a record builder, JSON artifact writer and dependency-free ESM evaluator separate. Emit a versioned external manifest, immutable revision-specific corpus and evaluator. Include evaluator source in the publication fingerprint. Pages advertise a manifest link but fetch neither corpus nor evaluator. A host keeps the expected revision from its loaded page, validates manifest and corpus, and caches by revision.

Request source-offset anchors for searchable original bullets. Prefer selected globally unique authored UUID destinations. Filter content records against anchors actually emitted by the renderer so rendering limits cannot create broken results. Version1 indexes bullet inline text only; non-bullet constructs remain outside the corpus.

The SDK exposes canonical-title body exclusions while retaining page-title results. This supplies a reusable boundary for host-generated content slots. Hosts that rewrite routes map results using page title and fragment at navigation time; the search corpus keeps its published revision and routes.

Normalize query and match text with Unicode lowercasing, NFC and whitespace collapse. Rank exact title, exact alias, title/alias prefix, all-term title/alias, then all-term content. Ties use normalized title, route and source position. Snippets are bounded plain text; display code uses text nodes.

## Risks / Trade-offs

- Source-offset anchors change after earlier source edits → revision-specific cache validation prevents mixing publications.
- One corpus can be large → keep it off ordinary page loading; issue96 measures before choosing chunking or worker execution.
- Conservative projection omits some rendered text → document supported searchable constructs and test privacy at serialized output.

## Migration Plan

Enable search explicitly. Existing publication defaults retain their output behavior. Deploy manifest and corpus together with pages; retain revision-specific assets during rolling static deployments. Rebuild without the option to remove the artifacts.

## Resolved Questions

### 1 - Which page kinds participate?

Stored selected pages and their unique aliases participate. Generated namespace titles can be added in a later version with an explicit result kind.

### 2 - How does a host exclude generated content?

A canonical-title body exclusion retains the page title and aliases while withholding all content records for that page. The `search-exclude:: true` control on an original bullet excludes its whole subtree.

## Open Questions

None.
