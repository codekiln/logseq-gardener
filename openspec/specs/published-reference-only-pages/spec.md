## Purpose

Publish a static page for every page that exists only because published blocks name it, reach it from every link, tag and view that names it, and keep its cost to readers and to the build within measured budgets.

## Requirements

### Requirement: Publish a page for each reference-only page

`lsg publish` SHALL write one page for each reference-only page the SDK returns. The page SHALL show its title, then the sections that apply in the published reference sections' order: Pages tagged with, Linked References, Hierarchy and Unlinked References. It SHALL show no outline, no property box and no Namespace notice.

#### Scenario: Deno Deploy

- **WHEN** a reader opens `Deno Deploy` on the encode garden
- **THEN** the page is titled `Deno Deploy`, Linked References reads `2 Linked References` and lists the blocks on `Deno` and on `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?`, and Unlinked References shows its collapsed header

#### Scenario: Tag page

- **WHEN** a reader opens `company`
- **THEN** the page shows `Pages tagged with "company"` listing 18 pages, then its Linked References

#### Scenario: Namespaced name

- **WHEN** a reader opens `1Password/Desktop/Settings/Developer/SSH Agent/Advanced`
- **THEN** the page shows Linked References with the block on `1Password/Dev/ssh/Tutorial/Setup Walkthrough`, and a Hierarchy section only when the namespace index gives it rows under the existing rules

### Requirement: Link to reference-only pages as to any page

A page reference, tag or page-valued property that names a reference-only page SHALL render as the same link markup as a reference to a page with a file: `a.page-ref` inside `.page-reference`, or `a.tag` for a tag, with no `unresolved` class, no Namespace style and no diagnostic. A reference to a generated namespace page SHALL link to it with the Namespace link style.

#### Scenario: Link on Deno

- **WHEN** a reader opens `Deno`
- **THEN** `[[Deno Deploy]]` is a link to the `Deno Deploy` page, styled as other page links, and the build reports no diagnostic for it

#### Scenario: Tag with a boundary slash

- **WHEN** a reader opens `Ableton/Live/UG/29 Audio Effect Reference/39 Tuner/04 Note Spellings`
- **THEN** the tag written `#/D♭)` in `(C#/D♭)` links to the page `D♭)`

#### Scenario: Link to a generated namespace page

- **WHEN** a block links `[[Ableton/Live]]`, a generated namespace page
- **THEN** the link opens `Ableton/Live` with the dotted underline and accessible Namespace name

### Requirement: Report only the references that stay unresolved

The unresolved page reference and unresolved tag diagnostics SHALL name only references to withheld, unselected or ambiguous names and to reference-only pages left out because of a route collision.

#### Scenario: Encode garden

- **WHEN** the encode garden is published with every top-level namespace, journals and search
- **THEN** the unresolved-reference diagnostics name only `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav` and `GitP/A/Session/26/10/22-Thu/Asset/Audio/Trimmed/wav`

### Requirement: List reference-only pages in All pages and search

All pages SHALL list each reference-only page as a row with its backlink count, no dates and the ordinary page link style; a reference-only journal SHALL be a journal row. Search SHALL find reference-only pages by title, ranked by the same rules as other titles.

#### Scenario: All pages row

- **WHEN** a reader searches All pages for `Deno Deploy`
- **THEN** a row links `Deno Deploy` with backlink count 2 and empty date cells

#### Scenario: Search by title

- **WHEN** a reader types `deno deploy` in search
- **THEN** `Deno Deploy` is the first page result and opens its page

### Requirement: Open reference-only pages from the graph and the right sidebar

Clicking a reference-only node in the graph view that has a route SHALL open its page. The page graph card SHALL open it the same way. Shift-click on a link to a reference-only page SHALL add a page card that shows the page as the main column shows it.

#### Scenario: Graph node

- **WHEN** a reader clicks the `Deno Deploy` node in the graph view
- **THEN** the browser opens the `Deno Deploy` page

#### Scenario: Right sidebar card

- **WHEN** a reader on `Deno` Shift-clicks `[[Deno Deploy]]`
- **THEN** the sidebar gains a page card for `Deno Deploy` showing its Linked References header, and the main page stays on `Deno`

### Requirement: Rewrite only the affected files after an edit

After a block naming a new reference-only page is added, publishing again into the same output SHALL change only the referring page, the new page, its reference fragments and Page graph file, the files that list pages, the graph data and the search files whose content depends on the new title. Removing the last reference to a reference-only page SHALL remove its page and fragments.

#### Scenario: New name

- **WHEN** the fixture garden is published, a block linking `[[Brand New Name]]` is added to one page and the garden is published again
- **THEN** the second build changes that page and writes `Brand New Name` and its fragments, and every other page file is unchanged

### Requirement: Keep reference-only pages within budgets

The `[reference_only]` table in `scripts/parity/budgets.toml` SHALL cap a reference-only page at 11 KB and 4 KB gzip and keep its first paint within the `[desktop]` and `[phone]` budgets. In CI, `parity:budgets` SHALL check those sizes on the fixture garden, and that adding 2,000 reference-only names to it leaves every other page's file unchanged. `parity:perf --baseline-lsg` SHALL mark a whole encode garden build over `build_slowdown_pct`, 10.

#### Scenario: Oversized page

- **WHEN** a fixture reference-only page grows to 5 KB gzip
- **THEN** `parity:budgets` fails and names the page and its size

#### Scenario: Garden growth

- **WHEN** `parity:budgets` publishes the fixture garden with 2,000 more reference-only names linked from one added page
- **THEN** every page file other than the added page and the new pages is unchanged, the fixture reference-only page included

### Requirement: Measure reference-only pages against Logseq's build

`parity:perf` SHALL load `Deno Deploy` in the whole encode garden's `lsg` build and in Logseq's publish-spa build of the same garden, on the desktop and phone profiles, and record data and script before the title, time to title, peak JS heap and search keystroke latency on the Performance page. `parity:compare` SHALL report no reference-only page from Logseq's build as missing from `lsg`, outside property pages, built-in pages and aliases.

#### Scenario: First visit to Deno Deploy

- **WHEN** `parity:perf` loads `Deno Deploy` with an empty cache on the phone profile
- **THEN** the `lsg` page sends at most 100 KB gzip of data and 16 KB of script before its title, shows its title within 1,500 ms and peaks at most 10 MB of JS heap, and both builds' values appear on the Performance page

#### Scenario: Missing pages

- **WHEN** `parity:compare` runs on the encode garden
- **THEN** its reference-only group lists none of `Deno Deploy`, `company`, `10977)`, `D♭)`, `Typescript` and `Nov 18th, 2024`
