## Context

Logseq's file parser names a block's namespace parents in `with-page-refs`, which calls `split-namespace-pages` on each referenced name, and gives a page its namespace with `split-last` in `page-name->map`. Both functions are in `deps/graph-parser/src/logseq/graph_parser/block.cljs`, and the helpers they call are in `deps/graph-parser/src/logseq/graph_parser/util.cljs`: `split-namespace-pages` splits on `/` and joins the parts back, and `split-last` cuts at the last `/`. It names each page with `page-name-sanity-lc`, which lower-cases the name, removes one slash at each end and normalizes Unicode. Nothing on that path trims spaces, so `MacOS /Applications Folder` has the parent `MacOS `. These are the files at the revision `scripts/logseq-parser/source.json` pins, `6efedb75588763af256bc7dfd0ed5526dc91fe7c` of `logseq/logseq`.

The SDK splits names the same way, then looks each parent up with `page_names::lookup_key`, which trims, so `MacOS ` becomes `MacOS`. `lookup_key` has about 75 callers, and most of them read written names, which the SDK trims on purpose today.

## Goals / Non-Goals

**Goals:**

- `mise run docs:parser-check` reports no difference for a namespace parent with a space before its slash.
- A garden page named `MacOS` lists no reference to `MacOS /Applications Folder` in its linked references, and the garden graph gives the parent `MacOS ` its own node.

**Non-Goals:**

- Keep the spaces inside a written reference's brackets, so that `[[Release Notes ]]` names `Release Notes ` as in Logseq. [Issue #421](https://github.com/codekiln/logseq-gardener/issues/421) tracks it, and the parser check's fixture uses it as its deliberate mismatch.
- Publish a separate generated namespace page for `MacOS ` when the garden has a page `MacOS`, and show it in the namespace hierarchy list. Both need their own address, which is the work of [Issue #351](https://github.com/codekiln/logseq-gardener/issues/351).

## Decisions

- `page_names::namespace_parent` returns the text before the last `/`, untrimmed. Every place that derives a parent from a name uses it: the reference index, the garden graph and the `(namespace …)` query.
- `page_names::namespace_parent_key` is `lookup_key` that keeps a space at the end of a name. The graph keys its nodes with it, and the reference index keys page titles with it when it finds a page's namespace parents. For a name without such a space it equals `lookup_key`, so every other node and page keeps its key.
- A reference's written names are trimmed before the reference index resolves them, as `lookup_key` already trimmed them. After that, only a namespace parent can end in a space, so a resolver can tell one apart with `page_names::ends_in_space`.
- The publisher's resolver answers a name that ends in a space only from its generated namespace pages, keyed with `namespace_parent_key`, and never from a garden page file. A generated namespace page whose title is `Claude/Code/Q/If I rename a CLI session with ` keeps the references it has today.
- `built_in_page_title` keys a name with `namespace_parent_key`, so `A ` from `[[A / B]]` is not the built-in page `A`.

## Risks / Trade-offs

- [A garden page file whose name ends in a space, such as `MacOS .md`, would have the title `MacOS `, which Logseq may match with the parent and the SDK does not] → The encode garden and the docs graph have no such file. A later change can key page titles with `namespace_parent_key` if one appears.
- [The garden has both `MacOS/Tools` and `MacOS /Applications Folder` but no page `MacOS`] → The publisher generates one namespace page, spelled `MacOS`, and the parent `MacOS ` names no page there. The garden graph still gives `MacOS ` its own node.

## Resolved Questions

### 1 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: a written reference with a space inside its brackets, `[[Release Notes ]]`. Logseq names `release notes ` and the SDK `release notes`; every line the check reports names the one file and that page. It shares this fix's cause, Logseq keeping spaces that the SDK trims, but changing it changes how every written link resolves, so it is filed separately as Issue #421.

### 2 - Does the check keep one deliberate-mismatch fixture?

Decided by the agent without codekiln: yes. The fixture shows that the check reports a difference with its file and line, which any genuine difference shows. The SDK tests added here keep the `MacOS /Applications Folder` case from coming back, and they run in seconds, while each extra fixture run costs a Logseq parser start and a test build.

### 3 - Where does the rule that a parent ending in a space names no page file live?

Decided by the agent without codekiln: in the resolvers, through `page_names::ends_in_space`, rather than in the reference index skipping such parents. A generated namespace page spelled with the trailing space must still answer to the parent, and only the resolver knows which pages are generated. The parser check's resolver keys every name with `namespace_parent_key`, so it reports the parent `macos ` as Logseq does.

### 4 - Do the namespace query and the graph follow the parent too?

Decided by the agent without codekiln: yes. `(namespace MacOS)` lists no `MacOS /Applications Folder` in Logseq, because that page's namespace is `macos `, and the graph draws Logseq's namespace edges. Both are one comparison each. The namespace hierarchy list keeps its trimmed keys, because it shares them with the publisher's page addresses.

## Open Questions

None.
