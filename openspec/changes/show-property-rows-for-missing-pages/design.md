## Context

`value()` in `crates/logseq-gardener-sdk/src/site/properties.rs` renders one property value. For each `[[name]]` it asks the published-name index for candidates and returns nothing unless exactly one page matches, so `view()` skips the row. The published-name index holds only published pages, so it cannot tell three cases apart: a name with no file, a page the site withholds or leaves outside the selected namespaces, and a name two published pages share.

The page body renders every one of those names as `<span class="page-reference unresolved">` with the name as written. Logseq 0.10.15 shows the row and links the name to an empty page.

The unarchived change `show-logseq-page-properties` also modifies the requirement "Publish selected leading page properties safely". This change's delta starts from that change's text, so the two apply in order.

## Goals / Non-Goals

**Goals:**

- A property row whose references name pages with no file shows, as in Logseq.
- The row reads the same way the body shows the reference: the name as written, unlinked.

**Non-Goals:**

- A property row whose reference is ambiguous shows, as Logseq shows it. [Issue #395](https://github.com/codekiln/logseq-gardener/issues/395) tracks this for the `alias` row.
- A reference to a namespace parent with no file links to the page `lsg` generates for it. The body does not link it either, so both would change together.

## Decisions

- **The garden's names decide.** `select_with_urls` already reads every page's title to fill `garden_titles`. It now also records each page's aliases with `page_names::page_aliases` in a new `Publication::garden_names`. A name that has no published candidate and is not in `garden_names` names a page with no file. Building the set reads the leading properties each page already has in memory, and it happens once per publish, so a page's first paint does not depend on garden size.
- **Withheld and excluded pages keep the old rule.** The privacy rule dropped a row that names a private or excluded page so that its name does not appear in the box. Keeping that rule for every name the garden has changes only rows that name no page at all. Aliases of withheld pages count as garden names, so they stay hidden as well.
- **No brackets.** Resolved property links render as `<a class="page-ref">` without Logseq's grey brackets, so the unresolved span adds `no-brackets` to match the rest of the box.
- **No new references or diagnostics.** An unlinked name adds no linked reference. The box has never reported diagnostics for property values, and a garden that links pages it has not written yet, as the encode garden does on 775 pages, would gain hundreds of diagnostics that ask for no action.

## Risks / Trade-offs

- A journal referenced in a date format other than its title, whose file is withheld, is not in `garden_names`, so its date shows unlinked → the date is text the author wrote on a public page, as the body already shows it.
- Journals skipped because `--journals` is off are not in `garden_names`, so their dates show unlinked → journals are not private unless marked, and the body shows the same text.

## Resolved Questions

### 1 - Should a reference to a private or excluded page now show too?

No. The body shows such names, and Logseq would too, but the existing requirement omits these rows on purpose. The issue asks only about pages with no file, so this change keeps the privacy rule.

### 2 - Is issue #395 affected?

No. An ambiguous alias still drops the `alias` row, and the row still lists only the page's own aliases.

## Open Questions

None.
