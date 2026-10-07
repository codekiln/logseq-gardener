## MODIFIED Requirements

### Requirement: Render supported syntax and report unsupported references

The publisher SHALL render outlines, headings, paragraphs, inline emphasis, literal code, quotes, tables, selected-page links, and supported local asset links. It SHALL render local and HTTPS MP3, WAV, and OGG image syntax as audio controls with a plain escaped accessible label and a link to the recording. Local recording links SHALL request a download. Hosted recording links SHALL say “Open recording” and omit the download attribute; download behavior depends on the source host and browser. It SHALL render HTTPS raster image syntax as images, including PNG, JPEG, GIF, WebP, AVIF, and ICO. HTTPS media URLs SHALL have an authority, a supported path extension, and no control characters. Generation SHALL perform no remote fetch; the browser requests remote media when viewing the site. Ordinary Markdown media links SHALL remain links. It SHALL escape source HTML and attributes. It SHALL link only to selected generated page destinations; unresolved page links SHALL keep selected-source labels. Unavailable block references, macros, unsupported remote media, Hiccup, and unsupported syntax SHALL produce visible fallback content and local diagnostics. It SHALL link unique explicit outline UUID references only to generated selected targets, with bounded escaped plain target labels when source labels are absent. It SHALL perform no query evaluation or transclusion.

#### Scenario: A selected page links to another selected page
- **WHEN** Start references Other using a page reference
- **THEN** its relative HTML link opens Other's generated page

#### Scenario: Excluded embed and block target
- **WHEN** a selected page references or embeds an excluded page or block whose source contains a sentinel
- **THEN** the sentinel is absent from all output and the unsupported references receive diagnostics

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
