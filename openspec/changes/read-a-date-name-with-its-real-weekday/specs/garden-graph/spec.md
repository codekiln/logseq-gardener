## ADDED Requirements

### Requirement: Title a page that names a date with the date's own weekday

A page name that spells a date in the journal title format, or in `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd`, SHALL name the journal with that date, as Logseq does. When no page with a file answers to the name, the reference-only page SHALL take the title the journal title format gives the date, whatever weekday or spelling the block wrote, and the page SHALL count as a journal. The graph SHALL hold one node for every spelling of the date.

#### Scenario: Wrong weekday
- **WHEN** the title format is `yyyy-MM-dd EEE`, a published page links `[[1967-09-03 Fri]]`, and no file answers to that date
- **THEN** the graph holds a reference-only journal node titled `1967-09-03 Sun` and no node titled `1967-09-03 Fri`

#### Scenario: Other spelling of the date
- **WHEN** the title format is `yyyy-MM-dd EEE`, one page links `[[Sep 3rd, 1967]]`, another links `[[1967-09-03 Fri]]`, and no file answers to that date
- **THEN** both pages have an edge to the one node titled `1967-09-03 Sun`
