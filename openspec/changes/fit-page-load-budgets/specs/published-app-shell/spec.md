## ADDED Requirements

### Requirement: Load page features after the first paint

`lsg-shell.js` SHALL start the scripts it loads after the page's `load` event, which are the outline, the reference sections, the journals feed, All pages and a restored right sidebar, only after the next paint, so no page's first paint waits on them.

#### Scenario: Small page on a fast connection

- **WHEN** a reader opens `Contents` on the desktop profile and the `load` event fires before the title is painted
- **THEN** `lsg-outline.js` and `lsg-references.js` are requested after the title is visible
