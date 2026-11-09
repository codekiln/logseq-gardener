## RENAMED Requirements

- FROM: `### Requirement: Completed-result latency measurement`
- TO: `### Requirement: Drawn-result latency measurement`

## MODIFIED Requirements

### Requirement: Drawn-result latency measurement

The shared controller SHALL coalesce pending input without a fixed debounce delay and cancel pending evaluation on close/revision changes. The existing parity harness SHALL time each keystroke until the dialog draws a result list for that input or for a longer input that starts with it, which `data-lsg-search-drawn` on `#lsg-search-results` marks. A drawn list SHALL include no-match outcomes, and the harness SHALL count, as unanswered, each keystroke no drawn list answered. A drawn list SHALL wait for the shards its query needs; a block row whose record chunk is still loading SHALL show its page title and a placeholder line until the chunk arrives, which `data-lsg-search-complete` then marks. Desktop and phone p95 budgets SHALL remain 50ms and 100ms.

#### Scenario: Rapid input and dismissal

- **WHEN** input changes repeatedly before evaluation or search closes
- **THEN** only current input can publish results and closed/stale work cannot update the dialog

#### Scenario: Keystroke ends at the drawn list

- **WHEN** a keystroke's query matches block rows whose record chunk has not arrived
- **THEN** the harness ends that keystroke when the list is drawn with placeholder lines, and the search browser check sees `data-lsg-search-complete` advance and the block text replace the placeholders once the chunk arrives

#### Scenario: Query with no results

- **WHEN** a keystroke's query matches nothing
- **THEN** the dialog draws the no-results state, `data-lsg-search-drawn` advances, and the keystroke is measured rather than counted as unanswered
