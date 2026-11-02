## 1. Redirects in the SDK

- [x] 1.1 Add `SiteOptions::old_url_redirects` and write the name map, the `index.html` script and, with readable URLs, the digest file redirects; verify with publish tests for a title, a journal, an alias, an ambiguous alias, a fragment, `#/all-pages`, `#/graph` and a page that keeps its digest file
- [x] 1.2 Fail before output when a redirect name is already generated or shared by two pages; verify with a publish test that checks the error and that no output directory exists
- [x] 1.3 Verify that output without the option is unchanged with the existing byte-for-byte publish tests
- [x] 1.4 Verify in a browser that a digest file redirect keeps the fragment and that `index.html#/page/<name>` opens the page, from disk and under a path prefix

## 2. CLI and docs

- [x] 2.1 Add `--redirect-old-urls` to `lsg publish`, its usage line and help text, and update `tests/assessments/help.json`; verify with the CLI tests and the help assessment
- [x] 2.2 Describe the option in `lsg/Ref/Publish Options`; verify the docs graph check passes

## 3. Integration

- [x] 3.1 Run `mise run ci` and verify it passes

## Workflow follow-up

- Create the cleanup sub-issue under issue #469.
- After merge, sync the delta specs and archive the change from a fresh main worktree.
