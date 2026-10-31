## ADDED Requirements

### Requirement: Selected publication view

The SDK SHALL provide one operation that builds a read-only view of a publication from a loaded garden, a filename format, a namespace selection and the publishing options. The view SHALL hold `apiVersion` `1`, a `revision`, a `pages` array and a `blocks` array. The publisher SHALL select, withhold and route pages through the same operation, so the view and the generated site agree on which pages are published.

#### Scenario: Same pages as the site

- **WHEN** a garden is published with build plugins
- **THEN** the view's page ids are the ids of the `page`, `journal` and `namespace` entries in `lsg-site.json`, and each page's `title` and `url` match its entry

#### Scenario: Selection moved without output changes

- **WHEN** the whole encode garden is published without plugins before and after the selection code moves into the SDK
- **THEN** both builds produce the same `lsg-site.json` revision

### Requirement: View pages

Each view page SHALL have `id` (the lower-cased page name), `title`, `url` (relative to the output root), `kind` (`page`, `journal` or `namespace`) and `properties`. Pages SHALL be sorted by `id`. Generated lists (All pages, the journals feed) SHALL be absent.

#### Scenario: Journal page

- **WHEN** a garden is published with `--journals`
- **THEN** each published journal appears once with kind `journal` and the URL of its generated file

### Requirement: Page properties in the view

A page's `properties` SHALL hold, for the plugin receiving the view, the values of the page properties it names in `pageProperties` that the page shows readers, as authored text. A property hidden by `:block-hidden-properties`, a page without page properties enabled and an unnamed property SHALL contribute nothing.

#### Scenario: Declared visible property

- **WHEN** a plugin names `recording` and a published page shows `recording:: https://example.com/a.mp3`
- **THEN** that page's `properties` holds `{"recording": "https://example.com/a.mp3"}` and no other property

#### Scenario: Hidden property

- **WHEN** a plugin names a property listed in `:block-hidden-properties`
- **THEN** no page's `properties` holds that property

### Requirement: View blocks

When the receiving plugin asks for blocks, `blocks` SHALL list every block of every view page with a source file, page by page in document order. Each block SHALL have `id`, `pageId`, `text` and `children` (the ids of its child blocks in order). A block's `id` SHALL be its `id::` UUID, lower-cased, when no other published block has the same UUID, and its position path (such as `3.1.2`) otherwise; the pair of `pageId` and `id` SHALL identify the block. `text` SHALL be the block's authored Markdown without its property lines and without its children. When the plugin does not ask for blocks, `blocks` SHALL be empty.

#### Scenario: Nested blocks

- **WHEN** a published page has a top-level block `Intro` with one child `Detail`
- **THEN** the view lists `{"id": "1", "text": "Intro", "children": ["1.1"]}` before `{"id": "1.1", "text": "Detail", "children": []}`, both with the page's id

#### Scenario: Shared UUID

- **WHEN** two published blocks carry the same `id::` UUID
- **THEN** both blocks are identified by their position paths, and no block in the view has that UUID as its id

#### Scenario: Properties removed from text

- **WHEN** a block's source has `id::` and `collapsed::` property lines under its first line
- **THEN** the block's `text` holds its first line only

#### Scenario: Pages only

- **WHEN** a plugin does not ask for blocks
- **THEN** its view's `blocks` is an empty array

### Requirement: Withheld content stays out of the view

The view SHALL omit pages excluded by namespace selection, withheld by `public:: false`, or withheld because the garden publishes only pages marked `public:: true`, their blocks and their properties, and SHALL contain no source file path, no garden root and no property value a reader cannot see.

#### Scenario: Excluded and withheld pages

- **WHEN** a garden has a page outside the selection and a selected page with `public:: false`
- **THEN** neither page's id, title, text or property values appear in the view

#### Scenario: Garden that publishes only marked pages

- **WHEN** a garden leaves `:publishing/all-pages-public?` off and marks one selected page `public:: true`
- **THEN** the view holds that page only, and its blocks are the only blocks in the view

#### Scenario: No source paths

- **WHEN** a plugin receives the view
- **THEN** no field of the view holds a source file path, and the garden folder's absolute path appears nowhere in it

### Requirement: View revision

The view's `revision` SHALL be a digest of the view's content and the `lsg` version. Two builds of the same garden with the same options and `lsg` version SHALL produce the same view and revision.

#### Scenario: Reproducible view

- **WHEN** the same garden is published twice with the same options and plugin
- **THEN** the plugin receives identical views both times
