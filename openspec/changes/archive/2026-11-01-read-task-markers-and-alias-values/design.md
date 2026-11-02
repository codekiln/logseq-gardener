## Context

Logseq's file parser, at the revision `mise run docs:parser-check` pins, builds a block's references from its tags, its page references, its task marker and its priority (`with-page-refs` in `deps/graph-parser/src/logseq/graph_parser/block.cljs`). It reads the values of `alias::`, `aliases::` and `tags::` with one rule (`parse-property` in `text.cljs`): the references in the value's inline syntax, plus each comma-separated part of its plain text. A value with no names stays one string, which becomes a literal alias or tag and supplies no block reference. A quoted value stays one string too.

The SDK collected block references in `Contribution::from_document` from the block's text and each property value's references, read `tags::` with a private `tag_names`, and read `alias::` in `page_aliases` with direct page references and plain text only.

## Goals / Non-Goals

**Goals:**

- `mise run docs:parser-check` reports no difference for task markers, priorities, or `alias::`, `aliases::` and `tags::` values.
- One SDK operation reads those three properties, so page aliases, tagged pages, linked references and the page graph agree with each other.

**Non-Goals:**

- Read a nested reference in any other property value as Logseq does. [Issue #378](https://github.com/codekiln/logseq-gardener/issues/378) tracks it, and the parser check's fixture uses it as its deliberate mismatch.
- Read the property names that a graph's `:property/separated-by-commas` lists with the same rule.

## Decisions

- `page_names::linkable_property_names` reads an `alias::`, `aliases::` or `tags::` value. Page aliases, a block's references and a page's tags all call it, and `tag_names` is gone.
- A block's references add the marker and priority that the parser reports on the bullet. `lsdoc` reports the same markers as mldoc, including `STARTED`, and only at the start of the block, so the SDK keeps no list of its own.
- A `tags::` value with no names, quoted or not, stays the page's one literal tag and supplies no block reference, as an alias value does.

## Risks / Trade-offs

- [Linked references and graph edges grow for gardens that use task markers] → Logseq shows the same blocks under TODO and the other marker pages. The graph leaves these built-in pages out unless built-in pages are on, as before.
- [Plain text in a `tags::` value next to a reference now names pages, such as `foo` and `baz` in `foo [[bar]] baz`] → Logseq does the same; the parser check compares the result.

## Resolved Questions

### 1 - Does a priority name a page too?

Decided by the agent without codekiln: yes. Logseq adds the marker and the priority together, and the parser check reports `[#A]` as a missing reference to `A`.

### 2 - Do `tags::` values follow the new rule as well as `alias::`?

Decided by the agent without codekiln: yes. Logseq reads the three properties with one rule. On the encode garden the shared rule fixes `tags:: [[a]] [[b]]`, which `tag_names` read as one name, and `tags:: #[[Spotify Playlists]]`, which it read as `[[Spotify Playlists]]`.

### 3 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: a nested reference in a `type::` value, `[[Notes on [[Mldoc]]]]`, which gives the SDK an extra reference to `Mldoc`. It is the property-value difference closest to the alias fixture it replaces.

## Open Questions

None.
