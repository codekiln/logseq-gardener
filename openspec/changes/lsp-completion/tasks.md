## 1. SDK completion context

- [x] 1.1 Add `index/completion.rs` with `CompletionContext`, `CompletionKind` and `completion_context`, reading an open `[[` or `((`, a tag, a property key and a property value from the cursor's line
- [x] 1.2 Extend the replaced range over the rest of the name and a closing `]]` or `))` after the cursor
- [x] 1.3 Unit tests for each kind, for embeds, for `#[[`, for prose and for the lines where Logseq reads properties

## 2. SDK candidates

- [x] 2.1 Add `GardenIndex::completions` with `Candidate`, `CandidateKind` and `Completions`
- [x] 2.2 Page names: titles, aliases, namespace parents, a journal for a typed date, and the level beneath a typed namespace
- [x] 2.3 Rank an exact name, then a name that starts with the typed text, then other matches, each by `search::fuzzy_score`
- [x] 2.4 Write a tag candidate as `name` or `[[name]]` by asking the parser
- [x] 2.5 Block candidates matched by every typed word, with the block's page
- [x] 2.6 Property keys without hidden properties, and property values, ranked by score and then by use count
- [x] 2.7 Cut the list at the limit and report a partial list
- [x] 2.8 Unit tests for each candidate kind, the ranking and the limit

## 3. Server

- [x] 3.1 Add `features/completion.rs`, which converts the position, asks the SDK and builds items with `textEdit`, `filterText`, `sortText`, `kind`, `detail` and `labelDetails`
- [x] 3.2 Register `textDocument/completion` in `dispatch.rs` and advertise the provider and its trigger characters in `capabilities.rs`

## 4. End-to-end tests

- [x] 4.1 Add `tests/lsp_completion.rs` covering each trigger, an alias item, a namespace level, a journal title, a tag with a space and a closing pair after the cursor
- [x] 4.2 Cover both position encodings, a buffer without a file, a file that is not open, prose, hidden property keys and the 100 item limit

## 5. Documentation and measurement

- [x] 5.1 Describe completion in `lsg lsp help`, `lsg/Ref/Command Line` and `lsg/How To/Edit a Garden in Neovim`
- [x] 5.2 Measure completion response time on the encode garden on a release build and record the numbers and method in `design.md` and the PR
- [ ] 5.3 Run `mise run ci`
