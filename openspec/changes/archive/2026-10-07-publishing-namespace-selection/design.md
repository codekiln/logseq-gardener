## Context

The SDK currently exposes version information. Parser comparisons favor lsdoc, while broader compatibility work remains open. Namespace selection needs only logical page identities and can be tested independently of garden loading.

## Goals / Non-Goals

**Goals:** Provide a reusable selection policy that includes namespace roots and descendants, gives exclusions precedence, and selects nothing by default.

**Non-Goals:** Garden loading, publication-property evaluation, reference expansion, and static site rendering follow under [Issue #11 — Rust static site publishing](https://github.com/codekiln/logseq-gardener/issues/11).

## Decisions

### Match logical identities at namespace boundaries

`NamespaceSelection` accepts included and excluded logical namespace roots. `includes(page_name)` matches the root itself or descendants whose next character is `/`. This prevents `My/AI` from accidentally selecting `My/AIM`. Filename decoding belongs to the loader. Exact, case-sensitive matching keeps this layer independent of graph identity normalization; the future loader must resolve selectors and page identities consistently before calling it. Prefix globs would make accidental widening harder to detect and are deferred.

### Require explicit inclusion and validate input

An empty include list selects nothing. Exclusion wins even when an included root is more specific. Construction rejects empty selectors, empty slash-separated segments, surrounding segment whitespace, and control characters. Matching malformed page names returns false. Duplicate selectors are harmless. This policy never interprets names as filesystem paths.

### Keep publication decisions in the shared SDK

The module has no filesystem access or external dependencies. CLI and Rust publishers can share it. The constructor reports which selector failed without echoing its contents; future public diagnostics must not expose excluded names. Page publication properties and all referenced content still require separate filtering.

## Risks / Trade-offs

- Selection alone does not prevent leaks through embeds or assets → the publisher must enforce visibility before rendering or collecting referenced content.
- Callers can supply inconsistent identity casing → document exact matching and test case differences; reconcile selectors through the garden loader during integration.
- This conservative syntax omits identities containing empty namespace segments → reject them explicitly until compatibility evidence establishes a safe interpretation.

## Resolved Questions

### 1 - Can selection proceed before parser adoption?

Yes. Selection consumes logical page identities supplied by a caller and needs no parser dependency.

### 2 - What should an empty inclusion list publish?

Nothing. A caller must name the namespaces to include.

## Open Questions

None for this policy. The publishing workflow still needs to settle direct HTML rendering versus Markdown export.
