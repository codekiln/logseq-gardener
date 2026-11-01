## MODIFIED Requirements

### Requirement: Add cards with Shift-click

At 640px and wider, Shift-click on a page reference, tag, page-valued property, Pages tagged with row, reference group title, page embed header, query result, All pages row, plugin-derived page link or breadcrumb crumb SHALL add a card for that page; on a bullet, a card for that block; on a block reference, a Block references card for the referenced block. A link SHALL count as a page link when it sits inside `.page-reference`, has the class `page-ref` or `tag`, or its address is a `p-*.html` file, and a link with the class `block-ref-link` SHALL count as a block reference at any address, so the gesture keeps working when a host rewrites page addresses after the build. Gardener SHALL render each of those page links inside `.page-reference` or with the class `page-ref` or `tag`, as Logseq renders every page link as `a.page-ref`; a plugin-derived link to a block of a page SHALL stay a plain link. A card's route SHALL be the address the browser resolves for the link, or for the current page when the target is one of its bullets, written relative to the site root that serves `lsg-sidebar.js`. The main page and its URL SHALL stay the same. The new card SHALL go to the top, expanded, and the sidebar SHALL open. A target already in the sidebar SHALL move to the top and expand. The placeholder Contents card SHALL be replaced by the first added card. Links and bullets inside cards SHALL behave the same way, and a plain click on a link inside a card SHALL open the page in the main column.

#### Scenario: Page and block cards on tmux
- **WHEN** a reader on `tmux` Shift-clicks the link to `tmux/Idea/Pomodoro with status line and silent flash` and then the second bullet
- **THEN** the sidebar holds a block card whose header is the breadcrumb starting `tmux`, above a page card for `tmux/Idea/Pomodoro with status line and silent flash`, and the URL is unchanged

#### Scenario: Block reference
- **WHEN** a reader on `VSCode/Proj/24t3/LearnVSCode/01 Terminal` Shift-clicks its block reference
- **THEN** a `Block references` card shows the referenced block and its children

#### Scenario: Card already present
- **WHEN** a reader Shift-clicks a page that already has a collapsed card lower in the sidebar
- **THEN** that card moves to the top, expanded, and no second card for the page appears

#### Scenario: Page links at rewritten addresses
- **WHEN** a host serves the site root at `/gitpa/` and pages at readable routes, and a reader on `/gitpa/GitP/A/Session/26/10/29-Thu/` Shift-clicks an inline page reference to `/gitpa/GitP/A/Session/26/10/28-Wed/`, a query result `a.page-ref` and a tag `a.tag`
- **THEN** each adds a page card that shows its page, the first is saved with the route `GitP/A/Session/26/10/28-Wed/`, and a plain click on the same links still navigates

#### Scenario: Bullet on a page at a rewritten address
- **WHEN** a reader on `/gitpa/GitP/A/Session/26/10/29-Thu/` Shift-clicks the page's first bullet
- **THEN** the block card is saved with the route `GitP/A/Session/26/10/29-Thu/` and block `1`, and shows that page's block

#### Scenario: Links inside a card for a rewritten route
- **WHEN** a reader opens the card menu of the page card for `GitP/A/Session/26/10/28-Wed/` and chooses Open as page, or follows a crumb in a block card from that page
- **THEN** the browser opens `/gitpa/GitP/A/Session/26/10/28-Wed/`, with `#zoom/` and the block's identifier for a crumb

#### Scenario: Link inside a flashcard block
- **WHEN** a reader Shift-clicks a page reference inside a flashcard block (`li.lsg-card`)
- **THEN** a card for that page is added

#### Scenario: Every kind of page link at rewritten addresses
- **WHEN** a host serves pages at readable routes under `/gitpa/`, and a reader Shift-clicks a page-valued `prev::` property, a Pages tagged with row, a Linked References group title and a page embed header
- **THEN** each adds a card for its page, and a plain click on the same links still navigates
