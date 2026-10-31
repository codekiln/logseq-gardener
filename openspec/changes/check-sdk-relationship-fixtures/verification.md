# Verification

## Relationship comparison

`tests/og_relationships.rs` passes for both filename formats. Before any SDK change, the SDK already matched every recorded value: eight stored pages with their names, titles, files, aliases, namespace parents and journal date; `Other Name` finding `Alias Source.md` and `Title Override.md`; `Second Name` finding `Alias Source.md`; and the pages without a file `comparison`, `Missing Target`, `Names` and `Research`, plus `Names/Nested` with parent `names` under the default format.

## Regressions that fail the checks

Each SDK change below was applied on its own, the relationship test was run, and the change was reverted.

| SDK change | Result |
| --- | --- |
| `page_names::alias_names` reads the second block instead of the leading properties | Both format tests fail: `Alias Source` and `Chosen Name` lose their aliases. |
| `PageNameIndex` keeps only the last page for a shared name | Both format tests fail: `Other Name` finds only `pages/Title Override.md`. |
| The graph index splits namespace parents on `//` instead of `/` | Both format tests fail: `Names/Nested` loses its parent `names`. |

Appending a line to the copied `pages/Media.md` made `mise run parser:sdk-fixtures-check` exit 1 with "differs from the recorded input" and a pointer to the update command. Restoring the note made it pass.

## Checks

- `mise run ci`: format, lint, tests, release build, parity budgets, OpenSpec validation, the parser baseline, the new SDK fixture check, skill versions, RuleSync and notices passed. `docs:check` stopped at its `mise install` step because a machine-wide mise configuration pins Python 3.12.1, which has no GitHub attestation. Its local link check passes on the changed documents when run alone. `clilint:check` (18 checks at score 4) and `release:check` passed when run individually. GitHub CI runs the full task.
- `openspec validate check-sdk-relationship-fixtures --strict` passes.
