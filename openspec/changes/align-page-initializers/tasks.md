## 1. Page graph initializer

- [x] 1.1 Add `window.logseqGardenerInitSidebar` to `sidebar.js` and list it in `host.mjs`'s `inits`.
- [x] 1.2 End the previous Page graph drawing's layout worker in `graph.js` when the card draws again.

## 2. Tests and documentation

- [x] 2.1 Add unit tests for the initializer and the worker ending to `tests/sidebar.test.cjs` and `tests/graph.test.cjs`.
- [x] 2.2 Add a browser check of media, slides, flashcards, the Page graph card and Shift-click after a same-document navigation to `scripts/plugins/browser.py`.
- [x] 2.3 Name the Page graph card and the document-level features in the navigation steps of `lsg/Ref/Plugin Host API`.

## 3. Handoff

- [x] 3.1 Run `mise run plugins:browser-test` and `mise run ci`, and open the implementation pull request with a cleanup sub-issue.
