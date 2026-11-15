## Context

[PR #645 — start lsg lsp](https://github.com/codekiln/logseq-gardener/pull/645) added `lsg lsp` and the SDK's `GardenIndex`, which holds every page and journal with its title, aliases, block ids and property lines. The server answers no editor request yet apart from its own `lsg/status` and `lsg/occurrence`. [Issue #641 — Complete page links, tags, block references and properties in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/641) adds `textDocument/completion`.

These facts shape the design:

- The cursor usually sits in text the parser reads as plain text. `[[Fru` is an unclosed bracket to lsdoc, so the index holds no occurrence there.
- The repository owner's Neovim uses `blink.cmp`, which filters and sorts items in the editor, and `mini.pairs`, which writes `]]` after a typed `[[` and `))` after `((`.
- The encode garden holds about 9,300 pages and 525 journals. [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) sets 100 ms at the 95th percentile as the target for a completion answer.

## Goals / Non-Goals

**Goals:**

- Completion after `[[`, `#`, `((`, at a property key and after `key::`, including inside `{{embed [[` and `{{embed ((`.
- Items for page titles, aliases, namespace pages, journal titles, blocks with an `id::`, property keys and property values.
- A text edit that leaves one closing pair when the editor already wrote `]]` or `))`.
- The context and the ranked candidates come from the SDK, so a later `lsg` command can list the same candidates.
- An answer within the epic's 100 ms target on the encode garden.

**Non-Goals:**

- Writing an `id::` on a block that has none when a person picks it after `((`. Issue #638 lists it under Later as "Mint block ids".
- Completing `today`, `tomorrow` and `yesterday` to a journal title. Issue #638 lists it under Later as "Journal helpers".
- A `completionItem/resolve` answer with a preview of the page or block. Hover, in [Issue #640 — Follow links, block references and journal dates, and list references, in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/640), shows the same text.

## Decisions

### The context comes from the line's text

`index::completion_context(text, offset)` reads the cursor's line and, for a property key, the lines above it. It returns the kind (page link, tag, block reference, property key, property value, or a page name inside `tags::` and `alias::`), the typed text, and the source range a chosen item replaces. It needs no index, so a test can call it with a string.

An open `[[` or `((` is the last one before the cursor on the line with no closing pair between it and the cursor. When both are open the later one wins, which makes `((` inside a link label and `[[` after a finished block reference read as a person expects. A tag starts at a `#` that begins the line or follows white space or `(`, and holds no white space up to the cursor; `# Heading` and the `#top` of a web address give no context.

A property key is offered only where Logseq reads properties: at the top of the page, or beneath a bullet with only property lines between the bullet and the cursor. The bullet's own first line gives no key context, because a word there is the block's text far more often than a key.

### The replaced range covers a closing pair after the cursor

For `[[` and `((` the range runs from the end of the opening pair over the typed text, the rest of the name after the cursor, and the closing pair when it follows on the same line with no other bracket of its kind between. Every page and block candidate inserts its text followed by the closing pair. With `- [[Fru|]]` the edit replaces `Fru]]` with `Fruit/Apple]]`; with `- [[Fru|` it replaces `Fru` with the same text. Both end with one closing pair, whether or not a pairing plugin ran.

A property key's range covers the rest of the key, its `::` and the spaces after them, and a key candidate inserts `key:: `. A value directly after `key::` gets a leading space.

### Candidates are a query on `GardenIndex`

`GardenIndex::completions(&context, limit)` returns `Completions { candidates, partial }`. Each `Candidate` has a kind, a label, the text to insert, the text the typed characters match, the page it belongs to (for an alias and a block) and a use count (for a key and a value). The module holds no protocol type.

Page names are every document title, every alias as its page spells it, and every namespace parent of a title, each name once with a title taking a name before an alias or a parent does. The list is built from the index's documents for each request. The measurement below shows that this costs about 20 ms on the encode garden, so the index keeps no second list to update on each edit.

### Ranking

Page names, keys and values are ranked in groups, as `search::SearchIndex::query` ranks page titles: a name equal to the typed text, then a name that starts with it, then another fuzzy match. `search::fuzzy_score` orders each group, comparing in lower case, and a name that scores zero is left out. Keys and values with equal scores are ordered by how many lines write them.

A journal for a typed date and the pages of a typed namespace come ahead of that order. When the typed text spells a date in a format `journals::parse_journal_reference` accepts, that day's journal title in `:journal/page-title-format` comes first, whether or not the journal has a file. When the typed text holds a `/`, the pages one level beneath the namespace before the last `/` come after the journal, matched on the segment after the `/`.

A block matches when its first line holds every typed word, compared after `search::normalize`.

### A tag is written as Logseq reads it

A tag candidate inserts the bare name when lsdoc reads `#name` as one tag that names the same page, and `[[name]]` otherwise, giving `#[[multi word]]`. Asking the parser keeps the rule equal to Logseq's for spaces, commas and other characters that end a tag. Only the candidates that are returned are parsed, at most 100.

### One hundred items, and `isIncomplete`

The server asks for 100 candidates, the number Logseq's own search returns for each kind. `isIncomplete` is true when the limit cut the list, so the editor asks again and a longer typed text reaches names the cut left out. It is also true for every list of page names, because a journal offered for a typed date appears only once the typed text spells the date, and the editor would otherwise filter the earlier list and never ask.

### Fields for a client that filters and sorts

`blink.cmp` matches the characters before the cursor against `filterText`, and sorts by its own score and then by `sortText`.

- `filterText` is the candidate's name, the block's text for a block, and the typed text for a journal offered because of a typed date, so `2026-01-04` keeps the item `Jan 4th, 2026` in the menu.
- `sortText` is the rank as four digits, so items with equal client scores keep the server's order.
- `label` is the name, alias, block text (at most 120 characters), key or value. `labelDetails.description` names the page of an alias and of a block, and `detail` reads `alias of Fruit/Apple`, the block's page, `journal`, `namespace` or the use count.
- `kind` is File for a page, Event for a journal, Folder for a namespace page without a file, Reference for an alias, Text for a block, Property for a key and Value for a value.

### The handler reads the buffer's text

`features/completion.rs` takes the text of the open buffer, or the indexed file's text for a document that is not open, converts the position with `positions::Encoding`, asks the SDK for the context and the candidates, and converts the replaced range back. A buffer the garden has no file for still completes, because the context needs only the text. A document with no text known to the server gets an empty list.

### Shutdown waits for `exit` itself

`lsp-server`'s `Connection::handle_shutdown` answers `shutdown` and then fails unless the next message is `exit`. A client that sends `shutdown` soon after `initialized` still has its answers to the server's `client/registerCapability` and progress token requests on the way, so the server exited with status 1. The main loop now answers `shutdown` and reads messages until `exit`: it ignores responses and notifications and answers a request with `InvalidRequest`, as the protocol describes.

### Measured response time

On the encode garden at commit `fcfefe0b524`, 9,855 documents, a release build on an Apple M2 Pro answered 1,050 completion requests with a median of 20.2 ms and a 95th percentile of 23.1 ms; the slowest took 67.7 ms. A script started `target/release/lsg lsp`, declared `utf-8` positions and file watching by the client as Neovim does, opened one unsaved buffer, and sent each of 21 typed texts 50 times, timing each request from write to answer.

| Typed text | Requests | Median | 95th percentile |
| --- | --- | --- | --- |
| After `[[` (8 texts, such as `[[`, `[[lo`, `[[Person/`, `[[2026-01-04`) | 400 | 21.8 ms | 23.4 ms |
| After `#` (4 texts) | 200 | 21.0 ms | 23.5 ms |
| After `((` (3 texts) | 150 | 3.3 ms | 4.1 ms |
| Property key (3 texts) | 150 | 0.4 ms | 0.5 ms |
| After `key::` (3 texts, one of them `tags::`) | 150 | 0.1 ms | 22.2 ms |

## Risks / Trade-offs

- A word typed at the start of a continuation line beneath a bullet is read as a property key, so the menu can offer keys while a person writes a second line of prose. → The context ends as soon as the line holds a space, and keys that match no typed letter are left out.
- The `:` trigger also fires after a single `:`, as in a time of day. → The context requires `key::` at the start of the line, and the server answers other places with an empty list in well under a millisecond.
- Building the page name list for each request grows with the garden. → At 9,855 documents it takes about 20 ms. A garden several times larger stays inside the target; the list can move into the index if a measurement shows the need.
- `blink.cmp` orders by its own fuzzy score first, so its menu can differ from the server's order. → `sortText` settles ties, and Neovim's built-in completion, which sorts by `sortText`, shows the server's order.

## Resolved Questions

### 1 - Where does the server learn what is being typed?

From the text of the cursor's line. The index's occurrences cover only syntax the parser recognizes, and a half-typed `[[Fru` is plain text to it.

### 2 - How many items does one answer hold?

One hundred, the number Logseq's search returns for each kind. A longer list is marked incomplete, so the editor asks again as the typed text grows.

### 3 - Why is every list of page names marked incomplete?

A journal offered for a typed date exists only once the typed text spells the date. An editor that received a complete list for `[[2026-01` would filter that list for `[[2026-01-04` and never ask the server, and the journal item would be missing.

### 4 - Does a candidate insert the closing pair?

Yes, always, and the replaced range takes in a closing pair that already follows the cursor. One rule covers an editor with a pairing plugin and one without.

### 5 - How is a tag whose name has a space written?

As `#[[name]]`. The server asks the parser whether `#name` reads as one tag with the same name, and adds the brackets when it does not.

### 6 - Which text matches a block?

The block's first line, which must hold every typed word. The item inserts the block's id.

### 7 - Which property keys are left out?

Logseq's card scheduling properties (`cards::is_hidden_property`) and the keys in `:block-hidden-properties`, which the index's settings hold. A key written only on the line being typed is also left out.

### 8 - Does the index keep a prepared list of page names?

No. Building the list for each request takes about 20 ms on the encode garden, a fifth of the target, and a prepared list would need its own update on every edit.

### 9 - Does plain fuzzy score order the names?

An exact name and then a name that starts with the typed text come ahead of other fuzzy matches. With plain fuzzy score, `[[lo` on the encode garden put `Blog` and `glob` ahead of `Logseq`. The published search groups page titles the same way.

### 10 - Does this step add a completion check to the Neovim task?

No. The end-to-end tests cover the protocol answers, and `mise run lsp:nvim-check` runs the documented snippet with `--clean`, which loads no completion plugin.

### 11 - Why does a completion change fix shutdown?

The end-to-end test that only reads the advertised capabilities shuts down right after `initialized`, and the server exited with status 1. The cause is in the main loop from [PR #645 — start lsg lsp](https://github.com/codekiln/logseq-gardener/pull/645). The coordinator of Issue #638 asked for the fix in this change because the workers on other steps reported the same failure.

## Open Questions

None.
