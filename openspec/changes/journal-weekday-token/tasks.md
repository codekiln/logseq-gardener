## 1. Reproduce

- [x] 1.1 Add tests that write and read back `E`, `EEE` and `EEEE` titles, accept `E, MM/dd/yyyy` in garden settings, and publish a garden that uses it; confirm they fail on main.

## 2. Weekday tokens

- [x] 2.1 Accept `E` as a short weekday token in `DateFormat::parse` and keep `EE` and `EEEEE` unsupported.
- [x] 2.2 Read a weekday of either name length for every weekday token.

## 3. Documentation and validation

- [x] 3.1 Remove the journal title limitation from `docs/parser-adoption.md`.
- [x] 3.2 Compare a published garden before and after, run `mise run ci`, and record the results in `verification.md`.
