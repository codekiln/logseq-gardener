## 1. SDK query

- [x] 1.1 Add `index/problems.rs` with `Problem`, `ProblemKind`, `Severity`, `RelatedLocation` and `GardenIndex::problems`
- [x] 1.2 Unit tests: each kind of problem, related locations within one document and across documents, a page without a file outside an embed, and a fix made through a buffer of another document

## 2. Language server

- [x] 2.1 Add `features/diagnostics.rs`: turn problems into diagnostics, keep the lists last published, and publish the lists that changed
- [x] 2.2 Call it from the text synchronization handlers on open, change and close
- [x] 2.3 Call it from the file watching handler for changed and deleted files, changed folders and a reload after `logseq/config.edn` changes
- [x] 2.4 Send `relatedInformation` when the client supports it, and name the places in the message otherwise

## 3. End-to-end tests

- [x] 3.1 Add the `related_information` option to the test client
- [x] 3.2 `tests/lsp_diagnostics.rs`: open a page with each problem, check the diagnostics, fix each one with `didChange` and check that they clear
- [x] 3.3 A fix and a break made in another open document, and two open documents that share an id
- [x] 3.4 A link, tag and property value naming a page without a file produce no diagnostic
- [x] 3.5 Files changed and deleted outside the editor, closing a document, a client without related information, and ranges in both encodings

## 4. Help and documentation

- [x] 4.1 Describe the diagnostics in the `lsg lsp` help and keep `mise run clilint:check` passing
- [x] 4.2 Add the diagnostics to `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line`

## 5. Checks

- [x] 5.1 Measure the reindex and `problems` times on the encode garden with a release build and record them in the pull request
- [x] 5.2 Run `mise run ci`
