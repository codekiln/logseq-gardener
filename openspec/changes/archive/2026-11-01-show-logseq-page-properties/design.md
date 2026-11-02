## Context

The page properties box decides which keys to leave out with `internal` in `crates/logseq-gardener-sdk/src/site/properties.rs`. That list started as Logseq's hidden keys plus keys `lsg` treated as control data: `title`, `alias`, `aliases`, `public`, `filters`, `search-exclude`, `file`, `file-path`, `file-name`, `custom_id`, `updated_at`, and every key that starts with `logseq.`, `logseq-proxy-` or `publishing/`. Query tables use the same list, minus `alias`.

Logseq 0.10.15 decides in `get-visible-ordered-properties` (`src/main/frontend/util/property.cljs`). It removes `hidden-properties`, which is `hidden-built-in-properties` from `deps/graph-parser/src/logseq/graph_parser/property.cljs`, the card keys that `frontend/extensions/srs.cljs` registers into it, and the graph's `:block-hidden-properties`. On a page's first block it also removes `hidden-editable-page-properties`: `title`, `filters` and `icon`. `property-cp` in `src/main/frontend/components/block.cljs` shows `alias`, `aliases` and `tags` values as one page link per name, separated by commas.

## Goals / Non-Goals

**Goals:**

- The properties box shows and hides the same keys as Logseq 0.10.15.
- An `alias` row shows the names the author wrote, each linking to the page.

**Non-Goals:**

- Query table columns follow Logseq's rule too. Logseq's query table removes the same hidden set, so `title` and `public` would become columns; that changes query output and is left for its own issue.
- The `alias` row also lists pages that name this page as their alias, as Logseq's `add-aliases-to-properties` does.
- Values that are not comma-separated keep Logseq's grey `[[` `]]` brackets.

## Decisions

- **One SDK function names the hidden keys.** `page_properties::is_hidden_page_property` holds Logseq's two sets and the card keys, so the rule sits in the SDK beside the other Logseq rules, and the publisher only renders. The publisher adds `:block-hidden-properties`, which it already reads.
- **The sets are copied exactly.** `custom_id` and `updated_at` show because Logseq lists only `custom-id` and `updated-at` for those two. Only the five `logseq.` keys Logseq hides on pages are hidden; `logseq.color` and the `logseq.table.*` keys are hidden on blocks only, so they show on a page.
- **`lsg`'s own keys follow Logseq.** `search-exclude`, `file`, `file-path`, `file-name`, `logseq-proxy-*` and `publishing/*` are not Logseq built-ins, so Logseq shows them. The existing value rules still drop values that are local paths, `file:` URLs or `logseq:` links, which covers what made these keys unsafe to show.
- **Alias names keep their spelling.** Other reference values show the canonical title of the page they reach, so `alias:: [[Logseq Gardener]]` would read `lsg`, the page's own title. Logseq shows the alias name, so the alias row labels each link with the name from `page_names::linkable_property_names`, the reader the alias index already uses.
- **An alias that names its own page adds no linked reference.** The alias row's targets are the page itself, and the reference index already leaves a page out of its own linked references.
- **Query tables keep the old list.** `is_internal` stays for query tables only, so this change touches the properties box alone.

## Risks / Trade-offs

- A garden that relied on `lsg` hiding `public`, `search-exclude` or `logseq-proxy-*` rows now shows them → add the key to `:block-hidden-properties`, as Logseq requires.
- An alias that two pages share is ambiguous, and its whole `alias` row is dropped, as for other ambiguous references → Logseq shows such an alias; this keeps the existing rule that an ambiguous reference never links.

## Resolved Questions

### 1 - Does `title::` show in the properties box?

No. The issue suggested showing it, but Logseq 0.10.15 removes `title`, `filters` and `icon` from page properties through `hidden-editable-page-properties`, so the box hides them.

### 2 - Does `search-exclude::` show?

Yes. Logseq does not know the key, so it shows it. Search still reads it from bullet properties, which the box never shows.

## Open Questions

None.
