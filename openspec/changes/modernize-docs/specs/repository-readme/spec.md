## MODIFIED Requirements

### Requirement: Reader entry point

The README SHALL explain the problem, audience, intended value, current maturity, maintainer, support, license, installation, first use, and links to focused pages on the documentation site at https://codekiln.github.io/logseq-gardener/ before implementation details. Installation SHALL start with the mise commands that install a released `lsg`: globally with `mise use -g github:codekiln/logseq-gardener`, and for one project with `mise use github:codekiln/logseq-gardener`, which pins `lsg` in that project's `mise.toml`. The README SHALL link the site's install pages for checksums, other platforms and building from source. The README SHALL put commands that install software or change the reader's configuration in `bash` code blocks. `mise run docs:check` SHALL run every `sh` code block in the README with the checkout's release build of `lsg` first on `PATH`, and SHALL fail before it runs any block when an `sh` code block runs `mise use`, `mise install`, `mise trust`, `cargo install` or `brew install`.

#### Scenario: First use

- **WHEN** `mise run docs:check` runs every `sh` code block in the README
- **THEN** each command runs the `lsg` built from the checkout, succeeds and shows the documented help or version result

#### Scenario: Install with mise

- **WHEN** a reader reaches the README's installation section
- **THEN** it shows the global and the project mise commands before any other install method and links the `lsg/How To/Install with mise` page on the site

#### Scenario: Commands that change the machine

- **WHEN** an `sh` code block in the README runs `mise use -g github:codekiln/logseq-gardener`
- **THEN** `mise run docs:check` fails, prints the block and runs no README command

### Requirement: Verified documentation

README commands and links SHALL be checked. The README's release claims SHALL match the published GitHub Releases, and its platform claims SHALL distinguish the targets that releases build from the platforms that the project checks run on. Every README link to the documentation site SHALL return a page. `mise run docs:check` SHALL fail when a README link to the documentation site names a page title that no page in `docs/pages/` has. `mise run docs:links` SHALL request every web link in the README and report each one that fails.

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

- **WHEN** the README links `https://codekiln.github.io/logseq-gardener/lsg/How%20To/Missing%20Page/` and no page in `docs/pages/` is titled `lsg/How To/Missing Page`
- **THEN** `mise run docs:check` fails and names the link
