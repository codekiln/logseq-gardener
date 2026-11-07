## ADDED Requirements

### Requirement: Old URL redirect option

`lsg publish` SHALL accept an optional `--redirect-old-urls` flag and pass it to the SDK's site options. A redirect that would replace another generated file SHALL exit one with empty stdout, a message naming the page and the file on stderr and no output directory. Publishing help SHALL describe the flag, the old addresses it redirects, and SHALL name the `lsg/Ref/Publish Options` page with its documentation site URL.

#### Scenario: Publish with redirects

- **WHEN** a caller runs `lsg publish --readable-urls --redirect-old-urls`
- **THEN** publishing succeeds, the report fields and index path are unchanged, and the output has a redirect at each page's old digest file

#### Scenario: Discover redirects offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--redirect-old-urls` and names the `lsg/Ref/Publish Options` page with its documentation site URL
