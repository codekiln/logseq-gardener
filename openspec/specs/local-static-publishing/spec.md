## Purpose

Define selected, source-preserving local HTML publication.

## Requirements

### Requirement: Generate selected standalone HTML pages

The SDK SHALL combine loaded Markdown, explicit filename interpretation, the existing namespace selection policy and the garden's public-page rule to generate a navigable index and standalone HTML pages. It SHALL count and skip journals unless the caller requests them, and SHALL then publish them according to the published-journals capability. It SHALL withhold any document containing a parsed `public:: false` property, and let exclusion override explicit public intent. It SHALL use deterministic routes and reject selected lowercase-title ambiguity and route collisions. It SHALL add safe namespace navigation destinations according to the published-namespace-hierarchy capability and include them in the complete page index. Generated namespace destinations SHALL contain navigation derived from selected visible titles; source-page reporting and source lookup SHALL retain their existing meaning.

The public-page rule SHALL follow Logseq's publishing export. When `logseq/config.edn` sets `:publishing/all-pages-public?` or `:all-pages-public?` to `true`, every page SHALL be public. Otherwise a page SHALL be public when its leading properties set `public:: true`, or when it names such a page with `alias::` or such a page names it. A selected page that is not public SHALL be withheld, counted as withheld and reported with a diagnostic, and its title SHALL be treated like a withheld title in namespace navigation. A setting value other than `true`, `false` or `nil` SHALL count as `false` and produce a diagnostic. A garden without `logseq/config.edn` SHALL treat every page as public.

#### Scenario: Select and exclude namespaces
- **WHEN** a garden contains `Notes/Start`, `Notes/Other`, and `Notes/Private/Hidden`, with include `Notes` and exclude `Notes/Private`
- **THEN** Start and Other are the only source pages in the index and generated output, accompanied by their safe namespace navigation destinations

#### Scenario: Private nested property withholds the page
- **WHEN** an included page contains a nested parsed `public:: false` property
- **THEN** the entire page is withheld, including its text and referenced assets

#### Scenario: Ambiguous selected title
- **WHEN** selected files derive titles that compare equal after Unicode lowercase conversion
- **THEN** generation fails before creating output

#### Scenario: Journals without the request
- **WHEN** a garden with journals is published without requesting journals
- **THEN** no journal page or feed is generated and the report counts every journal as skipped

#### Scenario: Setting absent or false
- **WHEN** a garden's `config.edn` leaves out `:publishing/all-pages-public?`, or sets it or `:all-pages-public?` to `false`, and include `Notes` selects `Notes/Marked` with `public:: true`, `Notes/Unmarked` with no `public::` property, and `Notes/Late` whose `public:: true` is on a later block
- **THEN** only `Notes/Marked` is published, and `Notes/Unmarked` and `Notes/Late` are absent and counted as withheld

#### Scenario: Alias of a marked page
- **WHEN** the setting is off and an unmarked selected page names a `public:: true` page with `alias::`
- **THEN** both pages are published

#### Scenario: Setting true
- **WHEN** a garden's `config.edn` sets `:publishing/all-pages-public? true`, or sets `:all-pages-public? true` while the newer key is absent or `false`
- **THEN** every selected page without `public:: false` is published

#### Scenario: Marked page outside the selection
- **WHEN** a page marked `public:: true` lies outside every included namespace
- **THEN** it is absent from the output

#### Scenario: Garden without a config file
- **WHEN** a garden has no `logseq/config.edn`
- **THEN** every selected page without `public:: false` is published

### Requirement: Render supported syntax and report unsupported references

The publisher SHALL render outlines, headings, paragraphs, inline emphasis, literal code, quotes, tables, selected-page links, and supported local asset links. It SHALL render local and HTTPS MP3, WAV, and OGG image syntax as audio controls with a plain escaped accessible label and a link to the recording. Local recording links SHALL request a download. Hosted recording links SHALL say “Open recording” and omit the download attribute; download behavior depends on the source host and browser. It SHALL render HTTPS raster image syntax as images, including PNG, JPEG, GIF, WebP, AVIF, and ICO. HTTPS media URLs SHALL have an authority, a supported path extension, and no control characters. Generation SHALL perform no remote fetch; the browser requests remote media when viewing the site. Ordinary Markdown media links SHALL remain links. It SHALL escape source HTML and attributes. It SHALL link only to selected generated page destinations; unique selected-page aliases SHALL resolve to their existing routes, while ambiguous or unresolved page links SHALL keep selected-source labels and receive local diagnostics. Unavailable block references, macros, unsupported remote media, Hiccup, and unsupported syntax SHALL produce visible fallback content and local diagnostics. It SHALL link unique explicit outline UUID references only to generated selected targets, with bounded escaped plain target labels when source labels are absent. It SHALL render bounded selected-content transclusions according to the selected-content-transclusion capability. Arbitrary queries SHALL retain visible fallbacks and local diagnostics; the publisher SHALL perform no query evaluation.

#### Scenario: A selected page links to another selected page
- **WHEN** Start references Other using a page reference
- **THEN** its relative HTML link opens Other's generated page

#### Scenario: Excluded embed and block target
- **WHEN** a selected page references or embeds an excluded page or block whose source contains a sentinel
- **THEN** the sentinel is absent from all output and the unavailable references receive diagnostics

#### Scenario: Source HTML is escaped
- **WHEN** selected source contains script-like HTML or an unsafe URL scheme
- **THEN** generated HTML preserves a safe visible fallback and executes no source-provided markup or unsafe URL

#### Scenario: Browser audio and artwork
- **WHEN** a retained page contains local or HTTPS MP3 image syntax and HTTPS GIF artwork
- **THEN** its HTML provides labelled audio controls, a recording link, and an artwork image with escaped attributes

#### Scenario: Local and hosted recording links
- **WHEN** a retained page contains local and HTTPS MP3 image syntax
- **THEN** the copied local recording link requests a download and the hosted recording link says “Open recording” without a download attribute

#### Scenario: Excluded media remains absent
- **WHEN** excluded or private pages contain local audio and unique HTTPS media URLs
- **THEN** no generated file contains their media URL, asset, or private text

#### Scenario: Unsupported remote media
- **WHEN** an image-syntax URL uses an unsafe scheme, control characters, lacks an HTTPS authority, or has an unsupported extension
- **THEN** its label remains visible with a local diagnostic and no media element requests the URL

#### Scenario: A selected outline reference navigates locally
- **WHEN** a selected page references a unique explicit UUID on a rendered selected outline item
- **THEN** its relative HTML link opens the target page at the matching item anchor

#### Scenario: A workshop alias navigates locally
- **WHEN** the workshop summary references a unique alias declared by another selected visible page
- **THEN** its relative HTML link opens that page and preserves the source alias label

### Requirement: Copy only supported referenced assets

The publisher SHALL copy only supported asset files requested by rendered selected content. It SHALL resolve decoded local paths relative to source documents and constrain them to the garden's assets directory, rejecting symlinks, special files, absolute paths, unsupported schemes and suffixes, and unsupported extensions. Invalid assets SHALL produce placeholders and local diagnostics.

#### Scenario: Referenced image is copied
- **WHEN** a selected page references a supported image under assets
- **THEN** the image is copied once into a deterministic route used by the page

#### Scenario: Excluded-only asset stays absent
- **WHEN** an asset is referenced only by excluded or withheld content
- **THEN** neither its file nor its content appears in output

#### Scenario: Unsafe asset path
- **WHEN** selected content requests an asset path outside assets or through a symlink
- **THEN** the request is rejected without copying its content

### Requirement: Preserve source gardens and reject existing output

The publisher SHALL require a fresh output directory with an existing parent outside the source garden. It SHALL never replace an existing output directory or write into the source garden. It SHALL prepare inputs before creating output and report filesystem failures. It SHALL provide a repeatable Rust command and record a browser-inspected real-garden demonstration.

#### Scenario: Rebuild into existing output
- **WHEN** the output directory already exists
- **THEN** the publisher fails without changing its contents

#### Scenario: Output inside source
- **WHEN** the requested output parent resolves beneath the source garden
- **THEN** the publisher fails without creating output

#### Scenario: Real-garden demonstration
- **WHEN** a documented public-garden namespace example is generated and inspected locally
- **THEN** its index, readable page content, navigation, and source preservation are recorded

### Requirement: Select a published homepage

The SDK SHALL accept an optional homepage page selector resolved through selected visible titles and aliases. A unique selector SHALL render the selected page at index.html and retain its ordinary page route. The complete page index SHALL remain available at pages.html and generated pages SHALL expose navigation to it. An unavailable or ambiguous homepage selector SHALL fail before output is created. Without a selector, index.html SHALL hold the journals feed's first batch when the published-journals capability writes a feed, with the complete page index at pages.html, and SHALL otherwise hold the complete page index.

#### Scenario: Choose the homepage by alias
- **WHEN** the caller chooses a unique selected visible alias as the homepage
- **THEN** index.html contains the target page's content with functioning local links and media, its ordinary page route remains available, and pages.html lists selected pages

#### Scenario: Reject a withheld homepage
- **WHEN** the requested homepage resolves only to an excluded or private page, or its selected lookup is ambiguous
- **THEN** publication fails before creating output and introduces no withheld content

#### Scenario: Preserve default publication
- **WHEN** the caller supplies no homepage selector and requests no journals
- **THEN** index.html retains the generated selected-page index

#### Scenario: Journals feed as the home page
- **WHEN** the caller supplies no homepage selector and journals are published with a feed
- **THEN** index.html holds the feed's first batch and pages.html lists selected pages, including journal pages
