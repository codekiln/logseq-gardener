## Context

`Renderer::invalid_properties` in `crates/logseq-gardener-sdk/src/site/render.rs` renders Logseq's "Invalid property names" warning. It asks `property_names::rejected_property_names` for the names Logseq rejects in a `Block::Properties`, and draws the warning when there are any. The same call serves a block's properties and a page's leading properties, and it runs again for every copy of a block in an embed, Linked References or query results. It records no diagnostic.

`Renderer::diagnostic` records a message for the page being rendered. In a copied block it puts the copied block's file in front of the message, and `site::page_diagnostics::PageDiagnostics` then reports it under that file, once, and not at all when that file reports it for its own page. `lsg publish` prints each diagnostic to stderr as `<file>: <message>`, and exits 0 when the site is complete.

The encode garden's `mise-tasks/site/build` writes those lines to a file, and its workflow fails when the file holds `shallow clone`. Gitpa's `scripts/build_site.py` runs `lsg publish` and fails only on a non-zero exit.

## Goals / Non-Goals

**Goals:**

- `lsg publish` reports each properties block that shows the warning, under the file that holds it, with a message a garden's CI can match.

**Non-Goals:**

- Let a garden's CI fail `lsg publish` on chosen kinds of diagnostic without matching message text, for example with an option that names the kinds that exit non-zero.

## Decisions

- `invalid_properties` records the diagnostic through `Renderer::diagnostic`, right where it draws the warning. The report then holds a diagnostic wherever a reader sees the warning, and copied blocks follow the rule that `PageDiagnostics` already applies to every other diagnostic.
- The message is `Invalid property names: ` followed by the names from `rejected_property_names`, in its order, separated by `, `. It opens with the warning's own words, so a reader of the report knows what they will see on the page, and a CI check can match the fixed prefix. Resolved Question 2 records the choice.
- `lsg publish` still exits 0. Logseq publishes such a block with its warning, and the CLI contract reports unsupported content as a diagnostic with success. Each garden decides in its own CI whether the diagnostic fails its build.

## Risks / Trade-offs

- [A garden's CI matches the message text, so rewording it would silently turn its check off] → The requirement fixes the `Invalid property names: ` prefix, and a test checks the whole message.
- [A page that `lsg` withholds reports nothing] → A withheld page reaches no site either. Gitpa publishes only public pages, so a rejected name on one of its private pages stays out of its report until the page becomes public.

## Resolved Questions

### 1 - Should `lsg publish` exit non-zero on a rejected property name?

Decided by the agent without codekiln: no. Logseq publishes the block and shows the warning, and the CLI contract treats unsupported content as a diagnostic with success. The encode garden and gitpa each get a CI step that fails on the diagnostic, as the encode garden's does for `shallow clone`: [codekiln/logseq-encode-garden#316 — Fail the site build on rejected property names](https://github.com/codekiln/logseq-encode-garden/issues/316) and [codekiln/gitpa#144 — Fail the site build on rejected property names](https://github.com/codekiln/gitpa/issues/144).

### 2 - What should the diagnostic say?

Decided by the agent without codekiln: `Invalid property names: ` and the names the warning shows, separated by `, `, such as ``Invalid property names: `logseq-entity``. The prefix is the warning's own heading, and the names are written as the warning's buttons write them, lower-cased with spaces and underscores turned into hyphens.

### 3 - Where does the diagnostic of a copied block go?

Decided by the agent without codekiln: under the copied block's file, once, as `report-each-diagnostic-once` decided for every diagnostic of a copied block. A page that embeds a block with a rejected name adds no diagnostic of its own.

## Open Questions

None.
