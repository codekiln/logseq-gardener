## MODIFIED Requirements

### Requirement: Page initializers

`lsg`'s per-page browser behavior that reads the page's `<main>` SHALL be registered as initializers that receive the page's `<main>` and signal, so the same behavior runs after a full load and after a same-document navigation. The registered initializers SHALL cover the remembered Properties choice, outline disclosure and zoom, references, All pages, theme controls, queries, journals and the right sidebar's Page graph card. Initializers SHALL remove their page-specific listeners when the signal aborts. Videos, YouTube timestamps, View as slides, flashcard reviews and the right sidebar's other cards SHALL keep working after a same-document navigation. The existing cancelable `lsg:media-seek` event SHALL keep its current behavior.

#### Scenario: Collapse after navigation

- **WHEN** a reader navigates in-document to a page with collapsed blocks and clicks a disclosure arrow
- **THEN** the block expands as it does after a full page load

#### Scenario: Page graph after navigation

- **WHEN** the right sidebar shows a Page graph card and the reader navigates in-document to another page
- **THEN** the card draws the new page's graph and the previous drawing's layout worker ends

#### Scenario: Page graph in a closed sidebar

- **WHEN** the right sidebar holds a Page graph card but is closed, the reader navigates in-document to another page, and then opens the sidebar
- **THEN** the card draws the page the reader is on

#### Scenario: Document-level features after navigation

- **WHEN** a reader navigates in-document to a page with a video, a YouTube timestamp, a flashcard deck or page links
- **THEN** Play loads the video, the timestamp loads it at that time, View as slides shows the page's blocks, the deck opens a review, and a Shift-click on a page link adds its card to the right sidebar, as after a full page load
