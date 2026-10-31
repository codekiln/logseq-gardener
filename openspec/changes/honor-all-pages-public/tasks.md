## 1. Reproduce

- [x] 1.1 Add publication tests for the setting absent, `false`, `true`, the older key, conflicting keys, a malformed value and a garden without `config.edn`, and confirm the absent, `false` and malformed cases fail on main.

## 2. Settings and selection

- [x] 2.1 Read `:publishing/all-pages-public?` and `:all-pages-public?` into `GardenSettings::all_pages_public`, with unit tests.
- [x] 2.2 Add `publishing::PublicPages` and `publishing::marked_public` for the `public:: true` and alias rule.
- [x] 2.3 Apply the rule to pages and journals in site selection, after namespace selection, with withheld counts, diagnostics and withheld titles.
- [x] 2.4 Set the key in existing test gardens that write a `config.edn` and expect every page published.

## 3. Documentation and validation

- [x] 3.1 Update the publishing, local-site and parser-adoption guides and the `lsg publish` help.
- [x] 3.2 Compare the encode garden's site before and after, run `mise run ci`, and record the results in `verification.md`.
