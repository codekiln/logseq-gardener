## MODIFIED Requirements

### Requirement: Build the same garden with lsg for the export comparison

`mise run parity:logseq-export` SHALL build `lsg` from the checkout and publish the garden into the same run directory. For the docs graph it SHALL use the options `docs:site` uses. For another garden it SHALL use the `lsg publish` options passed to the task, and it SHALL stop when another garden is given without options. It SHALL read the published pages' titles and URLs from the site's `lsg-site.json`.

#### Scenario: Docs graph by default
- **WHEN** a contributor runs `mise run parity:logseq-export` without arguments
- **THEN** the task publishes `logseq-gardener-docs` with `--filename-format triple-lowbar --include lsg --include Diataxis --homepage lsg --search --readable-urls`

#### Scenario: Another garden without options
- **WHEN** a contributor passes `--garden` with a garden other than `logseq-gardener-docs` and no `lsg publish` options
- **THEN** the task stops and asks for the options that select the garden's pages

### Requirement: Check lsg against performance budgets

The harness SHALL read budgets for each measure and profile from a committed budget file, and the results SHALL mark every `lsg` value that exceeds its budget. `mise run parity:perf --record` SHALL rewrite the `lsg/Ref/Logseq Compatibility/Performance` page, `logseq-gardener-docs/pages/lsg___Ref___Logseq Compatibility___Performance.md`, as Logseq outline Markdown with the results and the garden commit, `gh-pages` commit and `lsg` version they came from.

#### Scenario: Page over budget
- **WHEN** an `lsg` fixture page transfers more data before its title is visible than the budget allows
- **THEN** the results mark that value as over budget and name the budget

#### Scenario: Recorded rows
- **WHEN** a contributor runs `mise run parity:perf --record`
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page holds one row per fixture page, build and profile, with the commits and version used, and `mise run docs:site` publishes it with no diagnostic

