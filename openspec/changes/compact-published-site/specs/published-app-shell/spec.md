## MODIFIED Requirements

### Requirement: Keep the shell's cost fixed per page

The shell's behaviour SHALL ship as one shared `lsg-shell.js` file at the site root, loaded with `defer` by every page and at most 6.9 KB uncompressed, including the loader that imports the flashcard review on first use. The shell's styles SHALL ship in the shared `lsg-site.css`. The header and sidebar markup SHALL stay within 6 KB per page for the encode garden's favorites. A generated page's size SHALL stay the same when unrelated pages are added to the garden. The size checks SHALL run in CI through the parity budgets check.

#### Scenario: Shared script
- **WHEN** a site is generated
- **THEN** the output contains one `lsg-shell.js` and every page references it with `defer`

#### Scenario: Shared shell styles
- **WHEN** a site is generated
- **THEN** the shell's styles are in `lsg-site.css`, and no page carries them inline

#### Scenario: Garden growth
- **WHEN** the parity fixture garden is built again with 2,000 unrelated pages added
- **THEN** every fixture page and `lsg-shell.js` keep the same size and stay within their budgets
