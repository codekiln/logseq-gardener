## ADDED Requirements

### Requirement: Load the All pages script after the first paint

An All pages screen SHALL carry no script element for `lsg-all-pages.js`. `lsg-shell.js` SHALL load it after the page's `load` event and the next paint, and a plugin host that swaps an All pages screen into a page without the script SHALL load it. Until the script runs, the static rows, paging links and UTC dates SHALL serve the reader.

#### Scenario: Script before the title

- **WHEN** a reader opens All pages on the encode garden
- **THEN** `lsg-all-pages.js` is requested after the title is visible, and the script before the title matches a page without All pages

#### Scenario: Host navigation to All pages

- **WHEN** a site built with a plugin host adopts All pages from another page in place
- **THEN** the host loads `lsg-all-pages.js`, and sorting by Backlinks works
