## 1. Fixtures and expected results

- [x] 1.1 Add `scripts/og_relationship_fixtures.py` with `--check` and `--update`, and the `parser:sdk-fixtures-check` and `parser:sdk-fixtures-update` mise tasks.
- [x] 1.2 Generate the copied notes and `expected.json`, and write the fixture guide covering the source revision, the derivation and regeneration.

## 2. SDK test

- [x] 2.1 Add `GraphIndex::namespace_parent`.
- [x] 2.2 Add `tests/og_relationships.rs` comparing the SDK's results with `expected.json` for both filename formats, and point `candidates_reproduce_shared_og_alias_ambiguity` at the copied notes.

## 3. CI, documentation and verification

- [x] 3.1 Run the fixture check in `mise run ci` and describe the check in `docs/parser-adoption.md`.
- [x] 3.2 Show that alias, ambiguity and namespace regressions and a changed note fail the checks, run `mise run ci`, and record the results in `verification.md`.
