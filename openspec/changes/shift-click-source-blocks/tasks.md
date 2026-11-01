## 1. Shell and sidebar scripts

- [x] 1.1 In `shell.js`, send Shift-click on a link whose address holds `#source-block-` to the right sidebar
- [x] 1.2 In `sidebar.js`, have `target()` turn a `#source-block-` bullet into a block card and a `#source-block-` link into a Block references card, routed at the source page
- [x] 1.3 In `sidebar.js`, have `find()` look a block up by its `source-block-` anchor

## 2. Tests and docs

- [x] 2.1 In `tests/shell.test.cjs`, check that a `#source-block-` link at a `/gitpa/` address reaches the sidebar
- [x] 2.2 In `tests/sidebar.test.cjs`, cover a Linked References crumb and bullet at a `/gitpa/` readable route: card types, routes, content, crumbs and a restore on another page
- [x] 2.3 Describe the gesture and the `#source-block-` address in the Right sidebar section of `docs/local-site.md`

## 3. Verification

- [x] 3.1 Build a Gitpa preview with this branch's `lsg`, Shift-click a crumb and a bullet in Linked References, plain-click a crumb, and record the result in `verification.md`
- [x] 3.2 Run `mise run ci`
