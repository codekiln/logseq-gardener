## MODIFIED Requirements

### Requirement: Lazy validated search loading

The controller SHALL fetch no search data or evaluator before the visitor opens search or points at or focuses the search button. It SHALL then fetch the manifest from the address the page names, revalidating it with the server, and the engine and title index together, and on each query only the shards and record chunks its words need, validating same-origin paths, responses, version, digests and shape. Fetched files SHALL be cached by manifest address while the page stays open. When a file the manifest names is missing or fails validation, the controller SHALL forget the manifest and every file it named, so that a retry loads the current manifest. Close/retry/address changes SHALL abort or invalidate stale work; loading, empty, no-results and error/retry states SHALL stay distinct, with no automatic reload.

#### Scenario: Closing a slow first load

- **WHEN** a visitor closes search before its fetch completes
- **THEN** the request is aborted or its result ignored and a later open can load successfully

#### Scenario: Publication metadata changes

- **WHEN** the current page names another manifest address or none
- **THEN** old pending/cached results are invalidated and unusable controls are hidden

#### Scenario: A keystroke fetches only its shards

- **WHEN** the title index is loaded and a visitor types `tm` and then `tmux`
- **THEN** the controller fetches the `tm` shard once, fetches record chunks only for block results it shows, and fetches no other shard

#### Scenario: The site was published again after search loaded

- **WHEN** a shard the loaded manifest names is missing because the site was published again
- **THEN** the dialog shows Retry, and Retry fetches the manifest again and answers from the current publication's files
