## Purpose

Define how published pages show fenced code, example blocks and Markdown tables like Logseq's published site, with token colours computed at build time, and the size and build-time budgets that keep them from slowing first paint.

## Requirements

### Requirement: Render fenced code like Logseq

The publisher SHALL render each fenced code block as `div.cp__fenced-code-block` with the fence language in `data-lang`, containing the code text from the SDK. When the block has a language, a `div.extensions__code-lang` label SHALL show it in lower case at the top right, 12.25px, with `4px 6px` padding, opacity 0.8 rising to 1 on hover, on the editor's background; a block without a language SHALL have no label. The code SHALL use the font stack `"Fira Code", Monaco, Menlo, Consolas, "COURIER NEW", monospace` at 12px with 16px lines, Logseq's editor background for the active theme, a 2px radius, 18px of space above the first line and 62px below the last, and SHALL scroll sideways instead of wrapping.

#### Scenario: Labelled Rust block
- **WHEN** the `Rust/loop/label` page is published
- **THEN** its code block shows the label `rust`, starts at column 0 with `fn loop_labels_to_disambiguate_multiple_loops() {`, and has the font, size, line height, background, radius and spacing above

#### Scenario: Block without a language
- **WHEN** a fence has no language
- **THEN** the block renders with the same box and no label

#### Scenario: Long line on a phone
- **WHEN** a code line is wider than the content column at 390×844
- **THEN** the code block scrolls sideways and the page does not

### Requirement: Colour code tokens at build time

For a block whose language the SDK maps to a mode, the publisher SHALL wrap tokens in spans carrying Logseq's CodeMirror token classes (`cm-keyword`, `cm-string`, `cm-number`, `cm-def`, `cm-operator`, `cm-type`, `cm-variable`, `cm-comment` and the others the analysis lists), computed during the build, and SHALL style them with Logseq's `lsradix` colours for the active theme, comments in italic. A block whose language maps to no mode SHALL render as plain escaped text. Highlighting SHALL add no script to the page.

#### Scenario: Rust keywords and strings
- **WHEN** the `Rust/loop/label` page is published
- **THEN** `loop`, `let` and `break` carry `cm-keyword` in `rgb(189,75,0)` in the light theme and string literals carry `cm-string` in `rgb(41,124,59)`

#### Scenario: Mixed page
- **WHEN** `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain` is published
- **THEN** its Python and Bash blocks have token spans and its Mermaid block is plain text labelled `mermaid`

#### Scenario: Dark theme
- **WHEN** a reader views a highlighted block in the dark theme
- **THEN** the tokens and editor use Logseq's dark `lsradix` colours

### Requirement: Number code lines

Each fenced code block SHALL show a line-number gutter with Logseq's gutter background and number colour for the active theme, numbers at 12px with `0 5px` padding, one number per code line starting at 1. Selecting and copying a block SHALL copy only the code text.

#### Scenario: Line numbers
- **WHEN** the `Rust/loop/label` page is published
- **THEN** its Rust block shows numbers 1 to 23 in the gutter

#### Scenario: Copy
- **WHEN** a reader selects the whole Rust block and copies it
- **THEN** the clipboard holds the code lines without numbers

### Requirement: Render example blocks as wrapped text

The publisher SHALL render `#+BEGIN_EXAMPLE` blocks as `pre.pre-wrap-white-space` with the SDK's code text, wrapping long lines, on the theme's secondary background, without a label, gutter or token spans.

#### Scenario: Long example line
- **WHEN** a test garden has an example block with a line wider than the content column
- **THEN** the line wraps inside the block

### Requirement: Render Markdown tables like Logseq

The publisher SHALL render each Markdown table as `div.table-wrapper` containing a `table.table-auto` with the header row in `thead` as `th` cells with `scope="col"` and the other rows in `tbody`; every cell SHALL carry `org-left`, and alignment markers SHALL have no effect. The table SHALL fill the content column with collapsed borders and a `14px 0` margin. Header cells SHALL be 14px, weight 400, left aligned, in Logseq's header colour for the active theme, with `10px 8px` padding and a 2px bottom rule and no background. Body cells SHALL be 16px with `6px 8px` padding and no rules between rows, and rows SHALL alternate Logseq's two stripe colours for the active theme. The wrapper SHALL scroll sideways when the table is wider than the column. Inline markup in cells SHALL render as elsewhere on the page.

#### Scenario: Two-column table
- **WHEN** the `Rust/loop/label` page is published
- **THEN** its table has a wrapper, a `thead` with two `th` cells, a `tbody`, spans the content column and shows striped rows with the header style above

#### Scenario: Several tables
- **WHEN** `Claude/Code/Plugin/Marketplace/Auto-Update` is published
- **THEN** each of its four tables has the markup and computed styles above at 1280×800

#### Scenario: Wide table on a phone
- **WHEN** the widest fixture table is viewed at 390×844
- **THEN** the table scrolls inside its wrapper and the page does not scroll sideways

### Requirement: Keep code and tables within budget

The code and table styles every page carries SHALL stay within 1.5 KB gzip. The largest code page among the fixture pages SHALL stay within the existing first-paint data budget and grow by no more than the `[code]` budget over the build without highlighting. No script SHALL be added for code or tables. A whole-garden build SHALL take no more than 20% longer than the build without highlighting. `parity:budgets` SHALL check the style and page-growth budgets in CI, and `parity:perf` SHALL record build time.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on a build of the fixture pages
- **THEN** it fails if the code and table styles exceed 1.5 KB gzip or the largest code page grows past its budget

#### Scenario: Build time
- **WHEN** `parity:perf --record` builds the whole encode garden
- **THEN** the recorded build time is no more than 20% above the build without highlighting
