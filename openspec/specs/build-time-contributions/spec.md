## Purpose

The person publishing a garden can add a program that supplies content for its pages while `lsg publish` builds the site. A podcast garden, for example, can use one to list its episodes on the homepage. This program, a build plugin, reads the publication view: the list of pages and blocks that `lsg publish` will put on the site. It returns rows to show on those pages, and values that its own script reads in the reader's browser. The person publishing names a build plugin in a plugin manifest, and a Rust program that uses the SDK can register one directly. `lsg publish` checks everything a build plugin returns before it writes any file. Readers see the added rows on the page, while backlinks, search and the graph view list only what the garden's own files contain.

## Requirements

### Requirement: Build plugins

A build plugin SHALL be either a manifest with a `build` section passed to the publisher, or a Rust value implementing the SDK's `BuildPlugin` trait in `SiteOptions::build_plugins`. Both kinds SHALL share one id namespace, receive the publication view filtered for their `pageProperties` and block choice, and return contributions of the same types. Each plugin SHALL receive the same view; no plugin SHALL see another plugin's contributions.

#### Scenario: Rust plugin

- **WHEN** a Rust caller adds a `BuildPlugin` to `SiteOptions::build_plugins` and publishes
- **THEN** the plugin's `contribute` is called once with the view and its contributions appear in the output

#### Scenario: Shared id namespace

- **WHEN** a Rust plugin and a manifest declare the same id
- **THEN** publication fails before any file is written and the error names the id

### Requirement: Process plugin protocol

For a manifest with a `build` section, the publisher SHALL start the `build.command` array once per build, without a shell, in the manifest's folder. A first element containing `/` SHALL resolve against the manifest's folder; otherwise it SHALL be looked up on `PATH`. The publisher SHALL write one JSON object `{"apiVersion": 1, "plugin": {"id", "version"}, "view": …}` to the process's standard input and close it, read one JSON object `{"apiVersion": 1, "contributions": […]}` from its standard output, and pass its standard error through to its own. `build.blocks` (default `false`) SHALL choose whether the view holds blocks. The publisher SHALL fail the build, naming the plugin, when the process cannot start, exits non-zero, writes output that is not that JSON object, answers with another `apiVersion`, writes more than 64 MB, or runs longer than `build.timeoutSeconds` (default 60, at most 600); on timeout it SHALL kill the process.

#### Scenario: One process per build

- **WHEN** a garden with 500 published pages is built with one process plugin
- **THEN** the plugin's command runs once

#### Scenario: Plugin exits with an error

- **WHEN** a process plugin exits with status 1
- **THEN** publication fails, the error names the plugin id and the exit status, and no output directory exists

#### Scenario: Timeout

- **WHEN** a process plugin with `timeoutSeconds` 1 runs for 5 seconds
- **THEN** the publisher kills it after about 1 second and fails, naming the plugin and the timeout

### Requirement: Derived rows

A `rows` contribution SHALL name a view `pageId`, an optional `blockId` of that page, and a non-empty list of rows. Each row SHALL have `content`, a list of inline nodes, and `children`, a list of rows. Inline nodes SHALL be `{"text"}`, `{"page", "label"?}`, `{"page", "block", "label"?}` or `{"url", "label"}` with an `http`, `https` or `mailto` URL. The publisher SHALL render the rows as outline bullets inside an element with class `lsg-derived` and `data-lsg-plugin` set to the plugin id: after the target block's own children, or, without `blockId`, at the end of the page outline before the reference sections. Text SHALL be escaped and never parsed as Markdown or HTML. A page link SHALL point at the target page's URL and show its title unless a label is given; a block link SHALL add the block's anchor, and the publisher SHALL emit that anchor on the target page. Contributions to the same place SHALL appear in plugin id order, then in the order the plugin returned them.

#### Scenario: Episode list under a heading

- **WHEN** a plugin contributes two rows linking `episode/1` and `episode/2` under block `4` of the homepage
- **THEN** the homepage shows the two links as bullets after block 4's children, inside `.lsg-derived[data-lsg-plugin]`, and each link opens the episode's generated page

#### Scenario: Link to a block without a UUID

- **WHEN** a row links to block `2.3` of a page
- **THEN** the link's URL is that page's URL with a fragment, and that page has an element with the fragment's id at block 2.3

#### Scenario: Text is not markup

- **WHEN** a text node holds `[[Secret]] <b>x</b>`
- **THEN** the page shows those characters literally and no reference or element is created from them

### Requirement: Derived rows stay out of indexes

Derived rows SHALL create no linked or unlinked reference, no backlink count, no search record, no graph edge, no block anchor of their own and no content in any embed or block reference of their target. The reference fragments, search files and graph data of a build with row contributions SHALL be identical to those of the same build without them.

#### Scenario: No backlink from a derived link

- **WHEN** a plugin adds a row linking `episode/1` to the homepage
- **THEN** the Linked References of `episode/1` and its count in All pages are the same as without the plugin

#### Scenario: No search record

- **WHEN** a plugin adds a row with the text `derived-needle` and search is enabled
- **THEN** a search for `derived-needle` returns nothing and the search files are identical to the build without the plugin

#### Scenario: Embedded target

- **WHEN** another page embeds the block that received rows
- **THEN** the embed shows the block's authored children only

### Requirement: Contributed metadata

A `metadata` contribution SHALL name a view `pageId` and a JSON object of `values`. The publisher SHALL merge the values into that plugin's entry in the page's `data-lsg-page` metadata, beside its page property values. The build SHALL fail when the plugin has no `module`, when a key repeats one of the plugin's `pageProperties`, when two contributions of the plugin set the same key on one page, or when the plugin's entry for a page exceeds 64 KB of JSON.

#### Scenario: Chapter data for the player

- **WHEN** a plugin with a module contributes `{"chapters": [{"start": 0, "title": "Intro"}]}` for `episode/1`
- **THEN** the episode's page metadata holds the chapters under the plugin's id, and the browser host passes them to the plugin's `onPageAdopted` listener as `metadata.chapters`

#### Scenario: Metadata without a browser module

- **WHEN** a build-only plugin returns a `metadata` contribution
- **THEN** publication fails before any file is written, naming the plugin and the missing module

### Requirement: Contributions checked before output

The publisher SHALL validate every contribution against the view before writing any file. It SHALL reject, with an error naming the plugin, the contribution's position and the problem: an unknown `kind`, a `pageId` or `blockId` outside the plugin's view, a link to a page or block outside the view, an unsupported URL scheme, an empty row list, and a malformed field. The SDK SHALL expose the same validation and link resolution to other callers.

#### Scenario: Unknown page

- **WHEN** a plugin contributes rows to `private/notes`, a page withheld from the view
- **THEN** publication fails before the output directory exists, and the error names the plugin, the contribution and the page id

#### Scenario: Block outside a pages-only view

- **WHEN** a plugin that did not ask for blocks contributes rows under a `blockId`
- **THEN** publication fails and the error says the block is outside the plugin's view

### Requirement: Plugins run alongside indexing

The publisher SHALL build the view after selection and run each build plugin on its own thread while it builds the reference index, graph, queries and search records, and SHALL wait for every plugin before rendering the first page. Output SHALL not depend on which plugin finishes first.

#### Scenario: Deterministic output

- **WHEN** the same garden is built twice with two plugins whose run times differ between builds
- **THEN** both outputs are identical

### Requirement: No build work without build plugins

When no manifest has a `build` section and `SiteOptions::build_plugins` is empty, the publisher SHALL build no view, start no plugin thread or process, and write the same files as a build of the same commit before this change.

#### Scenario: Plain build

- **WHEN** the fixture garden is published without plugins
- **THEN** the output is identical to the output of the parent commit's `lsg`

#### Scenario: Plugin that contributes nothing

- **WHEN** the fixture garden is published with a build-only plugin that returns no contributions
- **THEN** the output is identical to the build without plugins

### Requirement: Build plugin budgets

The `[build_plugins]` table in `scripts/parity/budgets.toml` SHALL hold the budgets for build plugins, and a CI job SHALL check them on a pinned commit of the encode garden against a build of the same binary without plugins: added build time per published page at most 0.05 ms for the in-process no-op fixture with blocks, 0.1 ms for the process no-op fixture without blocks and 0.3 ms with blocks; peak memory of the `lsg` process with the process fixture and blocks at most 15% above the build without plugins; and identical output for a plugin that contributes nothing. `parity:budgets` in `mise run ci` SHALL check on the fixture garden that derived rows on one page stay at or under 30 KB gzip, that the page stays within `[desktop]` and `[phone]`, and that pages without contributions are unchanged apart from the plugin metadata tags and the source anchors that block links need.

#### Scenario: Whole-garden job

- **WHEN** the CI job runs on a pull request
- **THEN** it reports each `[build_plugins]` measure for the whole encode garden and fails when one exceeds its budget

#### Scenario: Fixture rows

- **WHEN** `mise run ci` runs `parity:budgets`
- **THEN** the fixture build plugin's rows pass the derived-row size and first-paint checks, and every page without a contribution matches the build without plugins apart from the plugin metadata tags and the source anchors that block links need
