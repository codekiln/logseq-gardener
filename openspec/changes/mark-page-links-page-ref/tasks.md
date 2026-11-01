## 1. Page link markup

- [x] 1.1 Render page links in page-valued text properties with `class="page-ref"` in `properties.rs`
- [x] 1.2 Render Pages tagged with rows and Linked and Unlinked References group titles with `class="page-ref"` in `references.rs`
- [x] 1.3 Render a page embed header's link with `class="page-ref"` in `render.rs`
- [x] 1.4 Render a plugin-derived link to a whole page with `class="page-ref"` in `derived.rs`, and leave a link to a block plain

## 2. Tests and docs

- [x] 2.1 Assert the class on each kind in `tests/site_properties.rs`, `tests/site_references.rs`, `tests/site_embeds.rs` and `tests/site_build_plugins.rs`, and that a derived block link stays plain
- [x] 2.2 List the page links Gardener marks in the Right sidebar section of `docs/local-site.md`

## 3. Verification

- [x] 3.1 Build a Gitpa preview with this branch's `lsg` through Gitpa's `scripts/build_site.py`, serve it, Shift-click each kind of link and plain-click one, and record the result in `verification.md`
- [x] 3.2 Run `mise run ci`
