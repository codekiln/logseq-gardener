## MODIFIED Requirements

### Requirement: Keep reference sections within budgets

The `[references]` table in `scripts/parity/budgets.toml` SHALL set: `references.js` at most 6 KB minified, references styles at most 1.05 KB gzip, each fragment file at most 100 KB as stored, first group visible at most 300 ms on desktop and 1,500 ms on the phone profile, peak phone heap at most 20 MB after opening both reference sections on `mise` and scrolling to the end, a filter change on `mise` at most 50 ms on desktop, and whole-garden build time at most 25% above the build before this change. `parity:budgets` SHALL check the size budgets in CI, and `parity:perf` SHALL mark the others.

#### Scenario: Oversized fragment
- **WHEN** a fragment file exceeds 100 KB as stored
- **THEN** `parity:budgets` fails and names the file
