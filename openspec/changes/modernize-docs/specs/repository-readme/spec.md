## MODIFIED Requirements

### Requirement: Reader entry point

The README SHALL explain the problem, audience, intended value, current maturity, maintainer, support, license, installation, first use, and links to focused pages on the documentation site at https://codekiln.github.io/logseq-gardener/ before implementation details. Installation SHALL start with the mise commands that install a released `lsg`: globally with `mise use -g github:codekiln/logseq-gardener`, and for one project with `mise use github:codekiln/logseq-gardener`, which pins `lsg` in that project's `mise.toml`. The README SHALL link the site's install pages for checksums, other platforms and building from source.

#### Scenario: First use

- **WHEN** `mise run docs:check` runs every `sh` code block in the README
- **THEN** each command succeeds and shows the documented help or version result

#### Scenario: Install with mise

- **WHEN** a reader reaches the README's installation section
- **THEN** it shows the global and the project mise commands before any other install method and links the `lsg/How To/Install with mise` page on the site

#### Scenario: Commands that change the machine

- **WHEN** `mise run docs:check` runs the README's `sh` code blocks
- **THEN** none of them runs `mise use` or another command that installs software or changes the reader's configuration

### Requirement: Verified documentation

README commands and links SHALL be checked. The README's release claims SHALL match the published GitHub Releases, and its platform claims SHALL distinguish the targets that releases build from the platforms that the project checks run on. Every README link to the documentation site SHALL return a page.

#### Scenario: Review README

- **WHEN** the contributor runs documentation checks and README style and purpose assessments
- **THEN** all local links resolve, commands pass, and no unsupported maturity or installation claim remains

#### Scenario: Released binaries

- **WHEN** a reader reads the README's project status
- **THEN** it says that GitHub Releases carry `lsg` binaries for the configured targets and links the releases page

#### Scenario: Site links

- **WHEN** a contributor runs `mise run docs:links`
- **THEN** every README link to https://codekiln.github.io/logseq-gardener/ returns a page
