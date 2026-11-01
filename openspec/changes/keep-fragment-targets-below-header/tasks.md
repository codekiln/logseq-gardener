## 1. Scroll padding

- [x] 1.1 Assert `html{scroll-padding-top:56px}` in the shell styles in `tests/site_shell.rs`, and see it fail
- [x] 1.2 Add `html{scroll-padding-top:56px}` to `STYLES` in `crates/logseq-gardener-sdk/src/site/shell.rs`
- [x] 1.3 Say on lsg/Ref/Host Integration API, in `docs/pages/lsg___Ref___Host Integration API.md`, that fragment scrolling clears the header, so a host needs no rule of its own

## 2. Verification

- [x] 2.1 Publish a fixture garden with a long page before and after the change, open a `#source-block-…` address, follow a same-page fragment link and choose a note result in search at 1280px and 390px, and record where the block lands in `verification.md`
- [x] 2.2 Run `mise run ci`
