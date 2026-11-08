## Context

Logseq drops one slash from each end of a page name in `gp-util/remove-boundary-slashes` (`deps/graph-parser/src/logseq/graph_parser/util.cljs`), which `block/page-name->map` calls before it spells the page's original name and finds its namespace. The SDK's `lookup_key` already drops those slashes, so a written name found its page, but `Contribution::from_document` in `references.rs` kept the name as written. A name with no page then became a reference-only node titled `a/b/`, and `namespace_parent("a/b/")` gave `a/b` instead of `a`.

## Goals / Non-Goals

**Goals:**

- A written name has no slash at its start or end, so its title, namespace and namespace parents match Logseq.
- `scripts/logseq-parser/check.py` reports no difference on `fixtures/trailing-slash-name`.

**Non-Goals:**

- Slashes inside a name, an empty namespace such as `a//b`, and names that are not page references, such as property values the SDK reads as literal text.
- Changes to the publisher.

## Decisions

- `page_names::remove_boundary_slashes` drops one slash from each end and keeps spaces, as Logseq does. `Contribution::from_document` applies it to every written name right after it removes blank names, so `unresolved`, `unresolved_parents` and the namespace walk all see the same spelling.
- A name that becomes empty, such as `[[/]]`, stays in the list and is skipped by the existing empty-name checks.
- The parser check's deliberate mismatch moves to `fixtures/wrong-weekday-date-name`, a real remaining difference that is filed as [Issue #503](https://github.com/codekiln/logseq-gardener/issues/503). `fixtures/trailing-slash-name` stays and the check must pass on it.

## Risks / Trade-offs

- [A name that differed only by a boundary slash now merges into one node] → This is Logseq's behavior: both spellings are one page.

## Open Questions

None.
