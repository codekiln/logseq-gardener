## MODIFIED Requirements

### Requirement: Verified documentation

README commands and links SHALL be checked. The README's release claims SHALL match the published GitHub Releases, and its platform claims SHALL distinguish the targets that releases build from the platforms that the project checks run on. Every README link to the documentation site SHALL return a page. `mise run docs:check` SHALL fail when a README link to the documentation site names a page title that no page in `logseq-gardener-docs/pages/` has. `mise run docs:links` SHALL request every web link in the README and report each one that fails.

#### Scenario: Review README

- **WHEN** the contributor runs documentation checks and README style and purpose assessments
- **THEN** all local links resolve, commands pass, and no unsupported maturity or installation claim remains

#### Scenario: Released binaries

- **WHEN** a reader reads the README's project status
- **THEN** it says that GitHub Releases carry `lsg` binaries for the configured targets and links the releases page

#### Scenario: Site links

- **WHEN** a contributor runs `mise run docs:links`
- **THEN** every README link to https://codekiln.github.io/logseq-gardener/ returns a page

#### Scenario: Site link to a missing page

- **WHEN** the README links `https://codekiln.github.io/logseq-gardener/lsg/How%20To/Missing%20Page/` and no page in `logseq-gardener-docs/pages/` is titled `lsg/How To/Missing Page`
- **THEN** `mise run docs:check` fails and names the link
