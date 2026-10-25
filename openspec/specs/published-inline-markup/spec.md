## Purpose

Define how published pages show page references, tags, outside links, emphasis, inline code, sized images and unsupported macros like Logseq, and the size budgets that keep this markup from slowing first paint.

## Requirements

### Requirement: Render page references like Logseq

The publisher SHALL render each page reference, plain or labelled, as a link to the referenced page inside `span.page-reference`. When the garden's brackets are on, `[[` before the link and `]]` after it SHALL show in a grey colour at opacity 0.3, drawn by the stylesheet on `span.page-reference` rather than written as elements, so they add no bytes per reference; when brackets are off, the reference SHALL carry class `no-brackets` and show no brackets. The link text SHALL be the reference as typed, or its label. The link SHALL use Logseq's page-reference colour for the active theme, normal weight, no underline, and the reference SHALL show a faint background with a 3px radius on hover. A reference that resolves to no published page or to several SHALL keep the same brackets and text without a link, and keep its diagnostic.

#### Scenario: Brackets on
- **WHEN** the `tmux` page references `[[tmux/Config]]`
- **THEN** the page shows `[[tmux/Config]]` with grey brackets at opacity 0.3 around a link to `tmux/Config` in Logseq's link colour with no underline

#### Scenario: Labelled reference
- **WHEN** a block contains `[LangSmith roadshow, Boston]([[LangSmith/26/09/29 Tue - Deep Agents]])`
- **THEN** the page shows `[[LangSmith roadshow, Boston]]` with the label linked to that page

#### Scenario: Brackets off
- **WHEN** a garden sets `:ui/show-brackets? false`
- **THEN** its page references render as links inside `span.page-reference.no-brackets` and show no brackets

#### Scenario: Unpublished target
- **WHEN** a reference names a page with no published file
- **THEN** its text shows between brackets, with no link, and the build reports a diagnostic

### Requirement: Render tags as links

The publisher SHALL render `#tag` and `#[[multi word]]` as `a.tag` linking to the page the SDK names for the tag, with text `#` followed by the tag text as typed and no brackets. Tags SHALL use Logseq's tag colour for the active theme, 0.9em font size, opacity 0.8, `0 2px` padding and a 4px radius, and on hover full opacity and an underline. A tag whose page is unpublished or ambiguous SHALL render `span.tag` with the same look and text, without a link, and keep its diagnostic.

#### Scenario: Tag link
- **WHEN** the `1Password/Quick Access` page is published and contains `#Keyshort`
- **THEN** `#Keyshort` is an `a.tag` linking to the `Keyshort` page with the tag style above

#### Scenario: Multi-word tag
- **WHEN** a block contains `#[[multi word]]` and page `multi word` is published
- **THEN** the page shows `#multi word` linking to that page

#### Scenario: Unpublished tag page
- **WHEN** a tag names a page with no published file
- **THEN** it renders as `span.tag` text with the tag style and the build reports a diagnostic

### Requirement: Mark outside links

The publisher SHALL render links to `http` and `https` addresses as `a.external-link` with `target="_blank"` and `rel="noopener noreferrer"`, in Logseq's outside-link colour for the active theme, with no text decoration and a 1px solid bottom border in the link colour. `mailto:` links SHALL keep their current markup.

#### Scenario: Heading link
- **WHEN** the `tmux` page has a heading linking to `https://github.com/tmux/tmux`
- **THEN** the link has class `external-link`, opens a new tab with `rel="noopener noreferrer"`, and shows a 1px solid bottom border

### Requirement: Style emphasis and inline code like Logseq

The publisher SHALL render bold at weight 600, keep italic, strikethrough and highlight in their current elements, and give highlights Logseq's highlight colours for the active theme. Inline code SHALL use Logseq's monospace font stack (`"Fira Code"` first), 0.8em font size, Logseq's code background for the active theme, the surrounding text colour, `3px 5px` padding and a 4px radius.

#### Scenario: Highlight
- **WHEN** `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` is opened
- **THEN** its highlighted phrase has Logseq's built-in highlight: a `#fef3ac` fill, `#262626` text, `2px 4px` padding and a 3px radius

#### Scenario: Inline code
- **WHEN** `1Password/Quick Access` is opened
- **THEN** the inline code `Cmd+\` has a 0.8em font size, `3px 5px` padding, a 4px radius and Logseq's code background

### Requirement: Size images as authored

When image syntax carries a `{:width …, :height …}` map, the publisher SHALL write each positive integer value as the image's `width` or `height` attribute and ignore other keys and values. Every image SHALL load lazily, keep its `alt` text, carry the same text as `title`, have a 3px radius, and never exceed the width of its container, keeping its aspect ratio when scaled down.

#### Scenario: Authored size
- **WHEN** `1Password/Dev/Connect Server` is opened at 1280×800
- **THEN** its connect diagram renders 231px wide and 265px tall

#### Scenario: Narrow screen
- **WHEN** an image has a `:width` larger than the content column
- **THEN** the image fits the column and keeps its aspect ratio

#### Scenario: Bad size map
- **WHEN** an image carries `{:width "big"}`
- **THEN** the image renders at its natural size with no `width` attribute

### Requirement: Show unknown macros as warnings

The publisher SHALL render a macro it has no handler for as `span.warning` with title `Unsupported macro name: <name>` and text `{{<name> <arguments joined by ", ">}}`, in Logseq's warning colours for the active theme with `.1em .4em` padding, and SHALL keep the build diagnostic. `{{renderer …}}` SHALL render nothing and report a diagnostic. The macros `embed`, `query`, `video`, `youtube`, `youtube-timestamp`, `cards`, `cloze`, `namespace` and `tweet` SHALL keep their current rendering until the issues that own them change it.

#### Scenario: Unknown macro
- **WHEN** a page contains `{{pdf x}}`
- **THEN** the page shows a warning chip with text `{{pdf x}}` and title `Unsupported macro name: pdf`

#### Scenario: Owned macro
- **WHEN** a page contains `{{query (todo now)}}`
- **THEN** it renders as it does today

### Requirement: Keep inline markup within budgets

Inline markup SHALL add no JavaScript and no graph data to the browser. The inline styles SHALL add at most 1 KB gzip to each page. The `tmux` fixture page SHALL stay under its 100 KB gzip budget and grow by at most 2 KB gzip. CI SHALL check both sizes.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it fails if the inline styles exceed 1 KB gzip or the largest fixture page exceeds its size budget
