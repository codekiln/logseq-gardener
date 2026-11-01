## MODIFIED Requirements

### Requirement: Keep the outline within performance budgets

The outline SHALL add no file per block and no graph data to the browser. `lsg-outline.js` SHALL be at most 8 KB uncompressed and load after the page's `load` event. The outline CSS SHALL add at most 1.5 KB gzip to each page. The `tmux` fixture page SHALL stay under 100 KB gzip and grow by at most 2 KB gzip. A bullet click on `tmux` SHALL paint the zoomed view within 50 ms on the desktop profile and 150 ms on the phone profile. The existing budgets for data and script before the title and for peak heap SHALL hold. CI SHALL check the script and page sizes, and the comparison harness SHALL record the zoom time.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it fails if `lsg-outline.js` exceeds 8 KB, the outline styles exceed 1.5 KB gzip, or the largest fixture page exceeds its size budget

#### Scenario: Zoom timing recorded
- **WHEN** `parity:perf --record` runs on the encode garden
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page gains the zoom time on `tmux` for both profiles, marked when over budget
