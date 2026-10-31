## Context

`lsg publish` chooses pages in two steps. Namespace selection (`publishing::NamespaceSelection`) keeps pages under the `--include` roots and outside the `--exclude` roots. The site builder then withholds any page that contains `public:: false` anywhere in its outline. Journals skip namespace selection and are published when the caller asks for them.

Logseq's publishing export decides differently. `build-html` in [`deps/publishing/src/logseq/publishing/html.cljs`](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/publishing/src/logseq/publishing/html.cljs#L135-L150) reads `:publishing/all-pages-public?` and falls back to `:all-pages-public?` when the newer key is not true. When the result is true it calls `clean-export!`, which drops pages whose properties set `public:: false`. Otherwise it calls `filter-only-public-pages-and-blocks`, which keeps pages whose properties set `public:: true` and the pages aliased to them ([`db.cljs` lines 111–149](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/publishing/src/logseq/publishing/db.cljs#L111-L149)). Page properties are the file's first block of properties ([`extract.cljc` line 228](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/graph-parser/src/logseq/graph_parser/extract.cljc#L228)), and only the exact text `true` becomes the boolean ([`text.cljs` line 91](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/graph-parser/src/logseq/graph_parser/text.cljs#L87-L97)). Files matched by `:hidden` never enter Logseq's database ([`cli.cljs` lines 18–35](https://github.com/logseq/og/blob/6e7afa8eb040686ff057156ee877193b581dd369/deps/graph-parser/src/logseq/graph_parser/cli.cljs#L18-L35)), and the SDK's garden loader already leaves them out, so `:hidden` applies before either rule.

## Goals / Non-Goals

**Goals:**

- Publish the pages Logseq's export would publish, for both values of the setting, within the namespaces the caller selects.

**Non-Goals:**

- None.

## Decisions

`GardenSettings` gains `all_pages_public`. Either `:publishing/all-pages-public? true` or `:all-pages-public? true` turns it on. `false` and `nil` leave it off. Any other value leaves it off and adds a diagnostic. `GardenSettings::default()` has it off, as Logseq does. `GardenSettings::read` turns it on for a garden without `logseq/config.edn`.

`publishing::PublicPages` holds the public-page rule. `PublicPages::all()` makes every page public. `PublicPages::marked(pages)` takes each page's title and document, and makes public the pages whose leading properties set `public:: true` (the last `public` key wins, the key is case-insensitive and the value must be exactly `true` after trimming), plus pages that name such a page with `alias::` or that such a page names. `publishing::marked_public` exposes the property check.

The site builder builds `PublicPages` once from every loaded page and journal, before selection. A page is published when namespace selection includes it (journals skip this step, as before), the public-page rule includes it, and it has no `public:: false`. A page that fails only the public-page rule is counted in `withheld_pages`, gets its own diagnostic, and joins the withheld titles that namespace navigation leaves out.

## Risks / Trade-offs

[Gardens that publish with the setting off lose pages] → The proposal marks the change as breaking. The diagnostic on each withheld page names the setting to turn on.

[Alias groups of more than one step] → Logseq's alias rule follows up to two alias steps; `PublicPages` follows one step in each direction. A page two alias steps from a marked page stays private, which withholds more than Logseq does.

## Resolved Questions

### 1 - How does the setting interact with `--include` and `--exclude`?

Both apply. Namespace selection chooses which pages the caller wants, and the public-page rule chooses which of those Logseq would publish. A `public:: true` page outside the included namespaces stays out of the site. This keeps the existing namespace contract and adds Logseq's rule as a second filter.

### 2 - What happens when the two keys disagree?

The site is all-public when either key is `true`. Logseq's export reads `:all-pages-public?` whenever `:publishing/all-pages-public?` is not true, so `:publishing/all-pages-public? false` with `:all-pages-public? true` publishes every page in Logseq's export. Logseq's settings screen (`frontend.state/all-pages-public?`) gives the newer key precedence instead; the SDK follows the export because the export decides what Logseq publishes.

### 3 - What does a garden without `logseq/config.edn` publish?

Every selected page without `public:: false`, as before. Logseq writes `config.edn` when it opens a folder, so a folder without one holds no publishing choice. Many SDK callers and tests publish plain Markdown folders, and this keeps their output unchanged.

### 4 - What does a value other than a boolean mean?

It counts as `false` and produces a diagnostic. Logseq's export treats any truthy value, such as a string, as on. Treating it as off keeps pages private when the setting is unclear, and the diagnostic tells the author to fix the value.

### 5 - Does a page with `public:: false` stay withheld when it is aliased to a public page?

Yes. Logseq's public-only export includes alias pages without checking `public:: false`. The SDK keeps its existing rule that `public:: false` anywhere in a page withholds the whole page.

### 6 - Do journals follow the rule?

Yes. Logseq stores journals as pages, and its export applies the same property check to them.

## Open Questions

None.
