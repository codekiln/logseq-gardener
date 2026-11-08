## Context

Logseq's graph parser rejects some property names, such as `` `logseq-entity `` in a block that starts with the code `` `logseq-entity:: [[Logseq/Entity/Asset]]` ``. `extract-properties` in `deps/graph-parser/src/logseq/graph_parser/block.cljs` lower-cases each name, turns spaces and underscores into hyphens, and adds a rejected name to the block's `:block/invalid-properties`. A page's leading properties get the same treatment in `with-pre-block-if-exists`.

Logseq's `invalid-properties-cp` in `src/main/frontend/components/block.cljs` renders those names, after the block's title, deadline and scheduled date and before its properties and body:

```html
<div class="invalid-properties mb-2"><div title="Invalid properties" class="warning">Invalid property names: <button class="p-1 mr-2">`logseq-entity</button></div><code>Property name begins with a non-numeric character and can contain alphanumeric characters and . * + ! - _ ? $ % &amp; = &lt; &gt;. If -, + or . are the first character, the second character (if any) must be non-numeric.</code></div>
```

The button has no click handler. In Logseq's build of the encode garden, `mb-2` gives 7px, `p-1` 3.5px and `mr-2` 7px, because the root font size is 14px; the button has no border or background and inherits the warning's colour and font; the `code` element wraps with `white-space: pre-wrap`.

Since [PR #440 — drop the property names Logseq rejects](https://github.com/codekiln/logseq-gardener/pull/440), `garden::parse_document` removes these properties from each `Block::Properties`. The block keeps its source span, so the rejected lines are still in the document's text. `references.rs` already reads those lines to blank their values before it reads page names.

## Goals / Non-Goals

**Goals:**

- The SDK says which names it dropped from each properties block, and the publisher shows Logseq's warning for them with Logseq's markup, text and styles.
- On the encode garden, `lsg` shows the warning on the blocks where Logseq's build shows it, with the same names.

**Non-Goals:**

- Hide the bullet of a block with no text, as Logseq does. That rule covers every empty block, not only these; [Issue #445 — Hide the bullet of a block with no text, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/445) tracks it.

## Decisions

- The SDK reports the names through `property_names::rejected_property_names(source, block)`. It reads the lines of a `Block::Properties` span with the property-line reader that `references.rs` used, which moves into `property_names` as `rejected_property_line` so both callers apply one rule. The function writes each name as Logseq does and lists it once, in source order. Resolved Questions 2 and 3 record why.
- The publisher, `site/render.rs`, calls that function where it meets a `Block::Properties` in an outline and renders the warning there. A block's properties follow its first line, so the warning sits after the block's text and before its body and children, where Logseq puts it. A page's leading properties come before its first block, so their warning sits between the title and the outline. Embedded blocks render through the same code and show the warning too, as in Logseq.
- The styles go into `site/inline.css`, which every page already loads through the site's shared stylesheet. They reuse the existing `.warning` style, whose colours already match Logseq's theme tokens, and add the margin, button and code rules. No page loads a new script, and the work per page grows only with the number of properties blocks on that page.

## Risks / Trade-offs

- [A properties span holds a line that the Markdown parser did not read as a property] → `rejected_property_line` applies the parser's key rule: optional spaces, a key with no spaces or colons, then `::`. On the encode garden the reported names match Logseq's database for all 13 blocks.
- [Logseq accepts a few names that `valid_property_name` rejects, such as `a,b,c`] → Such a name would show a warning that Logseq does not show. Neither the encode garden nor the docs graph has one; PR #440 records the difference.

## Resolved Questions

### 1 - What should a name's button do on a static site?

Decided by the agent without codekiln: nothing, as in Logseq. `invalid-properties-cp` gives the button no click handler, so the button does nothing on Logseq's site either. Keeping the `button` element keeps Logseq's markup, look and keyboard focus.

### 2 - Should the loaded document carry the rejected names, or should the SDK read them on request?

Decided by the agent without codekiln: the SDK reads them on request with `rejected_property_names`. The loaded document's blocks are lsdoc's types, which have no place for them, and a new field on `GardenDocument` would touch every place that builds one. The rejected lines stay in the document's text inside the properties block's span, and `references.rs` already reads them there.

### 3 - In which order and form should the warning list the names?

Decided by the agent without codekiln: lower-cased, with spaces and underscores turned into hyphens, each name once, in the order the block first gives it. `extract-properties` stores the names that way, in a set it builds by adding each name in turn; ClojureScript keeps a set of up to eight members in that order. On the encode garden, every warning names one property.

### 4 - Should a page's leading properties show the warning?

Decided by the agent without codekiln: yes, between the page title and the outline. Logseq gives leading properties to the page's first block and renders the warning there with the same component, at the top of the page. The encode garden has no such page, so a test covers it.

## Open Questions

None.
