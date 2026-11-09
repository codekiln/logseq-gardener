## Purpose

Find the pages that exist only because published blocks name them, as Logseq makes a page for every name a block links or tags, so the CLI, the language server and the publisher agree on which pages exist, what they are called and what refers to them.

## Requirements

### Requirement: Find the reference-only pages of a publication

The SDK SHALL return the reference-only pages of a publication: each name that a published block writes as a page reference, a tag or a page-valued property value, including `tags::`, and each namespace parent of such a name, when no published page, journal, alias or generated namespace page answers to it and no page or journal file in the garden has it as its title, alias or date. A task marker or priority SHALL add no reference-only page. The operation SHALL leave garden files unchanged.

#### Scenario: Link to a name with no file

- **WHEN** `Deno` and `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` link `[[Deno Deploy]]` and no file holds `Deno Deploy`
- **THEN** `Deno Deploy` is a reference-only page

#### Scenario: Tags make pages

- **WHEN** a published block writes `(pytest issue #10977)` and `Sourcegraph` and 17 other published pages list `company` in `tags::`, and neither name has a file
- **THEN** `10977)` and `company` are reference-only pages

#### Scenario: Namespace parents

- **WHEN** a published block links `[[1Password/Desktop/Settings/Developer/SSH Agent/Advanced]]`, and no file or published title answers to it, `1Password/Desktop/Settings`, `1Password/Desktop/Settings/Developer` or `1Password/Desktop/Settings/Developer/SSH Agent`
- **THEN** each of those four names is a reference-only page

#### Scenario: Name that is already a generated namespace page

- **WHEN** a published block links `[[AI]]`, `AI` has no file and `AI/General` is a published page
- **THEN** `AI` stays a generated namespace page and is no reference-only page

#### Scenario: Task marker

- **WHEN** a published block reads `TODO [#A] ship`, no block links `TODO` or `A`, and neither has a file
- **THEN** neither `TODO` nor `A` is a reference-only page

### Requirement: Keep withheld and unselected names unresolved

A name that a page or journal file in the garden has as its title, alias or date SHALL never be a reference-only page, whether that page is published, withheld by `public:: false` or by the all-pages-public rule, or outside the namespace selection. A name that only withheld pages, unselected pages or skipped journals write SHALL be no reference-only page. A file that `:hidden` in `logseq/config.edn` hides SHALL count as no file, as Logseq does not read it.

#### Scenario: Link to a withheld page

- **WHEN** the journal `2026_10_22` links `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav`, whose page has `public:: false`
- **THEN** that name is no reference-only page and the link stays unresolved

#### Scenario: Link to an unselected page

- **WHEN** the garden is published with `--include Deno` and a page under `Deno` links `[[mise]]`, which has a file
- **THEN** `mise` is no reference-only page and the link stays unresolved

#### Scenario: Alias of a withheld page

- **WHEN** a page with `public:: false` has `alias:: [[Hidden Name]]` and a published block links `[[Hidden Name]]`
- **THEN** `Hidden Name` is no reference-only page and the link stays unresolved

#### Scenario: Name only a private page writes

- **WHEN** the only block that links `Secret Project` is on a page with `public:: false`, and `Secret Project` has no file
- **THEN** `Secret Project` is no reference-only page

#### Scenario: Hidden file

- **WHEN** `:hidden` hides `pages/Draft.md` and a published block links `[[Draft]]`
- **THEN** `Draft` is a reference-only page

### Requirement: Publish reference-only pages under any selection

A reference-only page SHALL be part of the publication whenever a published block names it or one of its namespace descendants, whatever top-level namespaces the selection includes.

#### Scenario: Narrow selection

- **WHEN** the garden is published with `--include Deno`
- **THEN** `Deno Deploy` is a reference-only page of the publication, with the two blocks under `Deno` that link it

### Requirement: Title reference-only pages as Logseq does

A reference-only page SHALL take the spelling of the first reference to it that Logseq reads, by the rules the SDK already applies to alias names, after dropping one slash at each end of the name. A name that spells a date SHALL take the journal title format's title for that date and count as a journal. A name of a page Logseq creates in every graph SHALL take Logseq's spelling. A space before a slash SHALL stay in the parent's title.

#### Scenario: Two spellings

- **WHEN** published blocks write `Typescript` 28 times and `TypeScript` 15 times, and Logseq reads a `Typescript` reference first
- **THEN** the reference-only page is titled `Typescript`, and links in either spelling name it

#### Scenario: Boundary slash on a tag

- **WHEN** a published block writes `(C#/D♭)` and `D♭)` has no file
- **THEN** the reference-only page is titled `D♭)`, and no page is titled `/D♭)`

#### Scenario: Date without a journal file

- **WHEN** the journal title format is `MMM do, yyyy`, a published block links `[[Nov 18th, 2024]]`, and no journal file has that date
- **THEN** the reference-only page is titled `Nov 18th, 2024` and is a journal

#### Scenario: Space before a slash

- **WHEN** `Mac/OS/App` links `[[MacOS /Applications Folder]]` and neither it nor `MacOS ` has a file
- **THEN** `MacOS /Applications Folder` and `MacOS ` are reference-only pages, and `MacOS ` keeps its trailing space

### Requirement: Resolve links to every published page

The page-name index that page references, tags and page-valued properties use SHALL resolve published pages, journals, aliases, reference-only pages and generated namespace pages, by the same lookup key and journal-date rules, so every caller of the SDK resolves a name the same way. Generated namespace pages SHALL stay outside alias, embed, block reference and homepage lookup.

#### Scenario: Reference-only page in any case

- **WHEN** one block links `[[Deno Deploy]]` and another `[[deno deploy]]`
- **THEN** both resolve to the reference-only page `Deno Deploy`

#### Scenario: Generated namespace page

- **WHEN** `Person/Cal Newport/Pod` has `created-by:: [[Person/Cal Newport]]`, a block elsewhere links `[[Ableton/Live]]`, and both names are generated namespace pages
- **THEN** both references resolve to those generated namespace pages

#### Scenario: Embed of a generated namespace page

- **WHEN** a published page embeds `{{embed [[AI]]}}` and `AI` is a generated namespace page
- **THEN** the embed keeps its unavailable fallback

### Requirement: Route reference-only pages

A reference-only page SHALL have the route its title gives under the publication's page URL scheme, including the digest address for a title that cannot have a readable URL. A reference-only page whose route collides with another page's route or a file the build writes at the output root SHALL be left out of the publication with one diagnostic naming both, and references to it SHALL stay unresolved.

#### Scenario: Digest route

- **WHEN** the garden is published with the default page URLs
- **THEN** `Deno Deploy` has the route `p-<digest of "Deno Deploy">.html`

#### Scenario: Readable route for an unsafe title

- **WHEN** the garden is published with `--readable-urls`
- **THEN** `Deno Deploy` has the route `Deno%20Deploy/`, and `MacOS ` has its digest route

#### Scenario: Collision

- **WHEN** the garden is published with `--readable-urls` and a published block links `[[Assets/Logo]]`, whose first segment is the `assets` folder the build writes at the output root
- **THEN** `Assets/Logo` is left out, one diagnostic names it and `assets`, the link stays unresolved and the build completes

### Requirement: Answer what refers to a reference-only page

For a reference-only page, the SDK SHALL answer linked references, tagged pages, hierarchy rows, unlinked mentions and the backlink count from the same reference and namespace indexes and by the same rules as for a page with a file.

#### Scenario: Linked references

- **WHEN** the linked references of `Deno Deploy` are requested on the encode garden
- **THEN** they are the two blocks on `Deno` and on `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` that link it

#### Scenario: Tagged pages

- **WHEN** the tagged pages of `company` are requested
- **THEN** the 18 pages whose `tags::` list `company` are returned, starting with the first by lower-cased title

#### Scenario: Namespace parent's references

- **WHEN** the linked references of `1Password/Desktop/Settings` are requested
- **THEN** they include the block on `1Password/Dev/ssh/Tutorial/Setup Walkthrough` that links `1Password/Desktop/Settings/Developer/SSH Agent/Advanced`

### Requirement: Keep reference-only pages current as files change

When the reference index replaces one document's contribution, the SDK SHALL update the reference-only pages, their titles and their references so that every answer equals the answer of a full rebuild.

#### Scenario: New name

- **WHEN** a block linking `[[Brand New Name]]` is added to `Deno` and only `Deno`'s contribution is replaced
- **THEN** `Brand New Name` is a reference-only page with that block as its linked reference

#### Scenario: Last reference removed

- **WHEN** both blocks that link `Deno Deploy` are removed and the contributions of their two pages are replaced
- **THEN** `Deno Deploy` is no longer a reference-only page
