## Context

[Issue #371 Reconcile specs and code where they disagree](https://github.com/codekiln/logseq-gardener/issues/371) lists three pairs of specs that disagree, five places where a spec and the code disagree, and, in codekiln's comment of 2026-11-09, two more older requirements that `include-all-pages` left behind. [PR #365 correct publishing pages that contradict the specs](https://github.com/codekiln/logseq-gardener/pull/365) gives the evidence for each.

Where two specs disagree, the newer requirement describes the shipped code, so the older one changes. Where a spec and the code disagree, the code, its tests and Logseq's behaviour decide which side changes.

## Goals / Non-Goals

**Goals:**

- Each older requirement named in #371 agrees with the newer requirement and the code.
- The two small code bugs are fixed with tests that fail on `origin/main`.

**Non-Goals:**

- The parity harness's wait for `data-lsg-search-drawn`, which #371 also lists; another change is editing `parity-comparison-harness` now.
- Browser checks of the page initializers, tracked in [Issue #576 Check page behaviour after a plugin navigation and align the page initializers spec](https://github.com/codekiln/logseq-gardener/issues/576).

## Decisions

- Each spec change is a MODIFIED delta of the older requirement. The text names the newer requirement it defers to, so a reader finds the details in one place.
- `Contribution::from_document` in `references.rs` uses the shown property rows' pages for a leading property block and no longer adds the namespace parents of every written name. `ReferenceIndex::expand` already adds the parents of the shown pages from the page titles, so a shown row still counts for its namespace parent.
- The site `revision` digest reads each page's plugin values and the plugin host script, alongside the generated files it read before. Those two are the parts of the output the revision cannot read directly, because the page metadata holds the revision and the host's file name is its own digest. `plugins::page_head` is split into `page_plugins`, which builds the values, and `page_head`, which writes the tags.
- The bottom Page sequence region keeps leaving out a neighbour whose key is hidden. The spec changes, the code stays.

## Risks / Trade-offs

- [A namespace parent loses Linked References that came only from hidden properties] → Hidden keys are those the author asked not to show; the referenced page itself already left them out.
- [The graph view's layout files are still outside the revision digest, as the Site Manifest reference page says] → The layout follows the published links, which the page files hold, so a layout change without a page change is not expected. This change leaves that as it is.

## Resolved Questions

### 1 - Which requirement wins where `local-static-publishing` disagrees with the later query, block reference and macro requirements?

Decided by the agent without codekiln: the later requirements. published-queries, published-transclusion, selected-block-navigation and published-inline-markup describe the shipped behaviour, and the code follows them. The older requirement now defers to them for queries, unlabelled block references and `{{renderer …}}`.

### 2 - May a page other than `graph.html` fetch the graph files?

Decided by the agent without codekiln: yes, through an open Page graph card only. published-page-graph "Load neighbourhoods per page" has the card fetch `_graph/default.bin` or `journals.bin` for a page whose neighbourhood file is too large, and `graph.js` does so. A page with no open card still fetches no graph file and no graph script, which keeps first paint independent of graph size.

### 3 - Do macros add search text?

Decided by the agent without codekiln: only `{{youtube-timestamp}}`, which adds its displayed time. `plain` in `search.rs` does this, published-media-embeds requires it, and a reader who sees `10:20` on a page can search for it. A new test checks that a timestamp adds `10:20` and a `tweet` macro adds nothing.

### 4 - How should "Explicit namespace inclusion" and "Explicit SDK-backed publishing" treat `--include-all`?

Decided by the agent without codekiln: as codekiln's comment on #371 says, both defer to the newer requirements. An empty include list selects nothing unless the policy selects every page, and the publish flags include `--include-all`.

### 5 - Should the `lsg-site.json` revision change when only plugin metadata changes?

Decided by the agent without codekiln: yes, the code changes. The spec says the revision changes whenever any generated file changes, and a plugin that caches by revision would otherwise keep stale metadata. The fix is small and stays in the SDK. A new test publishes twice with different contributed chapter titles and fails on `origin/main` with equal revisions.

### 6 - Should a page reference in a hidden property count for the referenced page's namespace parent?

Decided by the agent without codekiln: no, the code changes. published-page-properties leaves hidden keys out of property backlinks, and the referenced page already left the reference out; only its namespace parent counted it. A new unit test in `references.rs` fails on `origin/main` and passes with the fix.

### 7 - Should a hidden `prev` or `next` key hide the bottom sequence link?

Decided by the agent without codekiln: yes, the spec changes. The code, the test `hidden_mixed_and_unsafe_values_supply_no_rows_or_property_backlinks` in `site_properties.rs`, and the Published Pages reference page all leave the link out. An author who hides the key has asked not to show that neighbour, and `:publishing/page-sequence? false` remains the switch for the whole region. Logseq has no sequence region to compare with.

### 8 - Should the plugin host register the page initializers its spec lists?

Decided by the agent without codekiln: not in this change. The scripts for media, slides, flashcards, the Page graph card and the sidebar use document-level listeners that may keep working after navigation, but only a browser check can show which ones break. [Issue #576 Check page behaviour after a plugin navigation and align the page initializers spec](https://github.com/codekiln/logseq-gardener/issues/576) holds that work.

## Open Questions

None.
