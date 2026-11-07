## Context

Logseq's file parser takes a page reference's name from mldoc's `Page_ref` value in `get-page-reference`, and from `page-ref-un-brackets!` for a tag or an embed. `with-page-refs` adds the namespace parents with `split-namespace-pages`, and `page-name->map` names each page with `page-name-sanity-lc`, which lower-cases the name, removes one slash at each end and normalizes Unicode. None of these functions trims spaces, so `[[Release Notes ]]` refers to `release notes `. They are in `deps/graph-parser/src/logseq/graph_parser/block.cljs`, `text.cljs` and `util.cljs` at the revision `scripts/logseq-parser/source.json` pins, `6efedb75588763af256bc7dfd0ed5526dc91fe7c` of `logseq/logseq`.

Logseq's link does trim. The `page-reference` component in `src/main/frontend/components/block.cljs` (`version/file` branch) calls `string/trim` on the name before it renders the link, so `[[Release Notes ]]` shows `Release Notes` and opens the page `release notes`. The block's reference still names `release notes `, so the page `Release Notes` lists the block in neither its linked references nor its unlinked references: the unlinked-reference pattern skips a name that follows `[[`.

The SDK trims written names in `Contribution::from_document`, and the change `name-namespace-parent-with-space` (Issue #412) already keeps the space before a namespace parent's slash with `namespace_parent_key` and `ends_in_space`.

## Goals / Non-Goals

**Goals:**

- `mise run docs:parser-check` reports no difference for a page reference, tag or embed with spaces inside its brackets.
- A garden page named `Release Notes` lists no block that links only `[[Release Notes ]]`, and the garden graph gives `Release Notes ` its own node.

**Non-Goals:**

- Publish a page for `Release Notes `, a name that only references have. That needs its own address, which is the work of [Issue #351](https://github.com/codekiln/logseq-gardener/issues/351).

## Decisions

- `Contribution::from_document` keeps each written name as lsdoc reads it, spaces included, and drops a name that is only spaces, as Logseq drops a blank name.
- `page_names::reference_key` replaces `namespace_parent_key`. It is `lookup_key` without trimming the spaces at either end, so `Release Notes `, ` Release Notes` and `Release Notes` get three keys. For a name without such spaces it equals `lookup_key`, so every other page keeps its key. The reference index, the garden graph, the `(namespace …)` query and the publisher's resolver use it where they used `namespace_parent_key`.
- `page_names::has_outer_space` replaces `ends_in_space`. A resolver answers a name with a space at either end from no garden page file, because a page title has no such space. The publisher's resolver can still answer it from a generated namespace page spelled the same, such as the parent `MacOS ` of `MacOS /Applications Folder`.
- The garden graph keys every name with `reference_key` and no longer trims written names, so it needs one builder function for a name instead of one for a written name and one for a namespace parent.
- The publisher's page link trims the name before it resolves and shows it, as Logseq's `page-reference` component does. The link for `[[Release Notes ]]` shows `Release Notes` and opens that page.
- The parser check's deliberate mismatch is a block that starts with inline code holding a property, `` `status:: [[Done]]` marks a finished task. ``, in `scripts/logseq-parser/fixtures/code-span-property`. Resolved Question 1 records why.

## Risks / Trade-offs

- [A garden page file whose title ends in a space, such as `Release Notes .md`] → Its title would not answer to `[[Release Notes ]]`, which Logseq may match. The encode garden and the docs graph have no such file.
- [Page names in `alias::`, `tags::` and other linkable property values] → `linkable_property_names` still trims each name. The parser check agrees on `type:: [[ Bar ]]` and `source:: [[ Quoted Prop]], [[Other ]]` before and after this change, so the change leaves them alone.

## Resolved Questions

### 1 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: a block that starts with inline code holding a property, `` `status:: [[Done]]` marks a finished task. ``. Logseq reads code; the SDK reads the property `` `status `` and a reference to `done`. Every line the check reports names the one file and one of those two names. The difference is genuine and current: the encode garden has 13 such blocks in 11 files, which the check reports before and after this change. Its cause is unrelated to spaces in names, so it cannot disappear with this fix by accident. It is filed as [Issue #431](https://github.com/codekiln/logseq-gardener/issues/431). The other candidates were journal references that Logseq and the SDK spell with different weekdays, such as `2026-06-28 sun` and `2026-06-28 sat`, which depend on the garden's journal format, and the namespace parent of a page that no block names, which comes from how the check lists Logseq's pages rather than from Logseq.

### 2 - Does `[[Release Notes ]]` name a page apart from `Release Notes`?

Decided by the agent without codekiln: yes, for references, and no, for the link. Logseq's database holds `release notes ` as a page of its own, with the block as its linked reference, while the link the block shows trims the name and opens `release notes`. The SDK follows both: the reference index and the graph use the name with its spaces, and the publisher's link trims it.

### 3 - Do spaces inside a property value's page reference count too?

Decided by the agent without codekiln: no change. The parser check agrees on property values with spaces inside their brackets before and after this change, so the SDK already reads them as Logseq does.

## Open Questions

None.
