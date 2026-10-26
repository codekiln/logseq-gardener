## 1. SDK

- [x] 1.1 Add a failing unit test in `publishing.rs` for page names with spaced segments under an included root and an excluded root
- [x] 1.2 Match page names with `valid_page_name`, keeping `valid_name` for selectors

## 2. Harness

- [x] 2.1 Read `lsg` link titles without trimming in `scripts/parity/common.py`
- [x] 2.2 Apply the page-name rule in `selectable` in `scripts/parity/compare.py`

## 3. Verification

- [x] 3.1 Publish the `Obsidian` and `Claude` roots of the encode garden and confirm the three pages appear under Logseq's titles
- [x] 3.2 Run `parity:build` and `parity:compare` on the whole encode garden: "Titles lsg cannot select" and "Only in lsg" are both 0; record the result in `verification.md`
