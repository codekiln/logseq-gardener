## ADDED Requirements

### Requirement: Link shared stylesheets in place of inline styles

The publisher SHALL write each style block that it adds to page heads as one file at the site root and SHALL link that file from the position the inline block would hold: the built-in page styles as `lsg-site.css` on every page, the search styles as `lsg-search.css` on every page when search is published, the flashcard widget styles as `lsg-flashcards-widget.css` on pages with a flashcard widget, and the derived-row styles as `lsg-derived.css` on pages with derived rows. A page SHALL contain no inline `<style>` element written by the publisher. The order in which styles apply SHALL stay the same, so a stylesheet given with `--stylesheet` SHALL be linked after `lsg-site.css` and before `lsg-search.css`. Every page SHALL look as it did with inline styles.

#### Scenario: Ordinary page
- **WHEN** the encode garden is published with search and `--stylesheet logseq/custom.css`
- **THEN** the head of `mise` links `lsg-site.css`, then `custom.css`, then `lsg-search.css`, and holds no `<style>` element

#### Scenario: Page with a flashcard widget
- **WHEN** a page holds a flashcard widget
- **THEN** it links `lsg-flashcards-widget.css`, and a page without one does not

#### Scenario: Same look
- **WHEN** the parity screenshots run on the fixture pages in both appearances before and after this change
- **THEN** the screenshots match

### Requirement: Keep only pre-paint scripts inline

Every page SHALL carry inline only the scripts that must run before it paints: applying the saved layout and theme in the head, and revealing the element named by the address's fragment at the end of the body. Together they SHALL be at most 1.5 KB uncompressed. The theme controller SHALL ship as `lsg-theme.js` and the search start-up as `lsg-search-boot.js`, each loaded with `defer` from the place its inline script held. Theme choices, `window.logseqGardenerInitTheme`, fragment reveal, and search opening after the page's second animation frame SHALL behave as before.

#### Scenario: Saved dark theme
- **WHEN** a reader who chose Dark opens another page
- **THEN** the page paints dark from its first frame

#### Scenario: Link to a collapsed block
- **WHEN** a reader opens a link whose fragment names a block inside a collapsed parent
- **THEN** the parent is open at first paint and the block is in view

#### Scenario: Inline script size
- **WHEN** `parity:budgets` builds the fixture garden
- **THEN** each page's inline scripts written by the publisher, other than dialogue and page-property start-ups, total at most 1.5 KB, and the check fails and names the page otherwise

### Requirement: Name files that every page references stably

Every file that every page references, including `lsg-site.css`, `lsg-search.css`, `lsg-theme.js`, `lsg-search-boot.js` and `lsg-shell.js`, SHALL have a fixed name, and its contents SHALL depend only on the `lsg` version. A change to the garden SHALL leave these files and every page's references to them unchanged. With `--readable-urls`, these names SHALL be reserved like the site's other generated files.

#### Scenario: Garden edit
- **WHEN** the garden is published, one block is added to one page, and the garden is published again with the same `lsg`
- **THEN** `lsg-site.css`, `lsg-search.css`, `lsg-theme.js` and `lsg-search-boot.js` are identical in both outputs, and so is each page's markup that refers to them

### Requirement: Keep the encode garden's site within size budgets

The `[site]` table in `scripts/parity/budgets.toml` SHALL set the encode garden's site at most 550 MB, its page HTML at the site root at most 200 MB and its `_refs/` directory at most 100 MB, measured as the total size of the files `lsg publish` writes with `--journals`, `--search` and every top-level namespace included. `parity:publish-bench --check` SHALL fail when the encode garden's output exceeds any of these, naming the part and its size. CI SHALL run that check on the pinned encode garden commit. The publication SHALL need no post-processing to meet these budgets. The same table SHALL set `build_slowdown_pct` to 10, and `parity:perf --baseline-lsg` SHALL mark a whole encode garden build more than 10% slower than the build before this change.

#### Scenario: Within budget
- **WHEN** CI publishes the pinned encode garden commit
- **THEN** the site, its page HTML and `_refs/` are within their budgets and the check passes

#### Scenario: Oversized part
- **WHEN** a change makes the encode garden's `_refs/` 120 MB
- **THEN** `parity:publish-bench --check` fails and reports `_refs/` at 120 MB against its 100 MB budget

### Requirement: Measure the encode garden against Logseq's build

The whole encode garden SHALL be published with this change and compared with Logseq's `gh-pages` build by `parity:compare` and `parity:perf`, and the Performance page SHALL record both sites' sizes and file counts beside the page-load rows. The page-load rows SHALL keep their budgets: at most 100 KB gzip of data and 16 KB of script before the title on a first visit, the title visible within 300 ms on desktop and 1,500 ms on the phone profile, and a peak JS heap of at most 10 MB. A second page opened in the same visit SHALL load no stylesheet or script from the network before its title.

#### Scenario: First visit
- **WHEN** `parity:perf` loads `Foam` with an empty cache on the phone profile
- **THEN** the data before the title, which now includes `lsg-site.css`, stays within 100 KB gzip and the title appears within 1,500 ms

#### Scenario: Second page
- **WHEN** a reader on `Foam` follows a link to `mise`
- **THEN** `mise` requests only its own document from the network before its title
