## ADDED Requirements

### Requirement: Render block references like Logseq

The publisher SHALL render a resolved block reference as a link to the block's anchor on its page, holding `span.block-ref` with the SDK's display content: the first line through the inline renderer, a task checkbox for a task marker with the done style for `DONE`, and an inline heading at 1rem for a heading block, as `span.block-ref-heading` with `role="heading"` and its `aria-level`. The reference SHALL use the body text colour, a `0.5px` dashed bottom border in the link colour, `2px 0` padding and `cursor: alias`, in both themes. A labelled reference SHALL show its label inside `span.block-ref`.

#### Scenario: Done task reference
- **WHEN** a reader opens `Claude/Code`
- **THEN** the first block reference shows a checked checkbox and a page reference with its brackets, inside a dashed underline

#### Scenario: Reference destination
- **WHEN** a reader clicks a block reference
- **THEN** the browser opens the referenced block's anchor on its page

### Requirement: Render invalid block references as warnings

The publisher SHALL render a block reference that the SDK cannot resolve as `span.warning` with the title `Block ref invalid`, showing `((uuid))`, and SHALL keep its diagnostic.

#### Scenario: Missing uuid
- **WHEN** a test garden page references a uuid that no published block carries
- **THEN** the page shows `span.warning` titled `Block ref invalid` with `((uuid))` and the build reports a diagnostic

### Requirement: Render embeds as Logseq's wells

The publisher SHALL render a block embed as `div.embed-block` and a page embed as `div.embed-page`, each with Logseq's well: background, 8px radius, 8px margin and `12px 8px` padding, in both themes. The embedded blocks SHALL render through the outline renderer with bullets and collapse arrows, honouring each block's `collapsed::`, with no duplicate HTML IDs. A page embed SHALL start with a header holding the page icon and a link to the page in the link colour at 19.2px and weight 700, with no brackets.

#### Scenario: Block embed outline
- **WHEN** a reader opens `Spotify/Tool/TUI`
- **THEN** the embed shows a shaded well with bullets, and collapsing an embedded block works

#### Scenario: Page embed header
- **WHEN** a reader opens `AI/Coding/History`
- **THEN** each page embed shows a header linking to its phase page

### Requirement: Shade nested embeds

Each nested well SHALL take Logseq's next shade, from level 2 to level 5, the deepest level the nesting limit allows, in both themes.

#### Scenario: Second level
- **WHEN** a reader opens `AI/Coding/History` in the light theme
- **THEN** a well inside a well has the background `rgb(233,236,239)`

### Requirement: Render unavailable embeds like Logseq

The publisher SHALL render nothing for an unavailable block embed, the well and header with no body for a page embed whose page has no published file, and only the header for a self embed. A page that is not published SHALL appear in the header as text, without a link. Each case SHALL keep its diagnostic. Past the nesting limit, the publisher SHALL render `p.warning` reading `Embed depth is too deep` for an embed and `span.warning` reading `Block ref nesting is too deep` for a block reference.

#### Scenario: Unknown block embed
- **WHEN** a test garden page embeds an unknown uuid
- **THEN** the page shows nothing in its place and the build reports a diagnostic

#### Scenario: Depth limit
- **WHEN** a test garden chains seven page embeds
- **THEN** the sixth level shows `Embed depth is too deep`

### Requirement: Keep transclusion within budgets

The transclusion styles every page carries SHALL stay within 1 KB gzip. Transclusion SHALL add no script beyond the outline script. The fixture page that embeds the most SHALL stay within the 100 KB first-paint budget. `parity:budgets` SHALL check the style size in CI and SHALL fail when no fixture page holds an embed.

#### Scenario: Budget check
- **WHEN** CI runs `parity:budgets`
- **THEN** it fails if the transclusion styles exceed 1 KB gzip, if no fixture page holds an embed, or if an embedding fixture page exceeds 100 KB gzip
