## ADDED Requirements

### Requirement: Optional publication timings

lsg publish SHALL accept an optional --timings flag. With it, human output SHALL keep its stdout report and add one final stderr line, after any diagnostics, that names each publication phase from the SDK in order with its time in milliseconds and the total. JSON output SHALL add `timings`, a list of objects with `phase` and `ms`, and `total_ms`. The timings SHALL contain no page title, path or content. Without --timings, stdout and stderr SHALL be unchanged, and with or without it the published files SHALL be the same.

#### Scenario: Human timings
- **WHEN** a caller publishes with --timings
- **THEN** stdout matches a publication without the flag, and stderr ends with one line that starts with `Timings in milliseconds: read` and lists every phase and the total

#### Scenario: JSON timings
- **WHEN** a caller publishes with --timings and --format json
- **THEN** the report lists the phases read, parse, index, plugins, render, history, search, graph and write in that order, and `total_ms` equals their sum to within rounding

#### Scenario: Published files are unchanged
- **WHEN** a caller publishes the same garden twice, once with --timings and once without
- **THEN** both sites contain the same files with the same contents, and a JSON report without --timings has no `timings` or `total_ms`
