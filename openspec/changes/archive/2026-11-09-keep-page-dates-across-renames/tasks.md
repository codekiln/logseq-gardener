## 1. Reproduce

- [x] 1.1 Add tests to `git_dates.rs` for a renamed page file, a renamed graph folder, a new page at a renamed page's old path and a file name with a double quote, and see them fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Read the whole repository's history with rename detection in one `git log`, and the garden's prefix and shallow state in one `git rev-parse`
- [x] 2.2 Track renames newest first, leave dates unchanged by a rename with identical text, and keep the paths under the garden's `pages/` and `journals/`

## 3. Verification

- [x] 3.1 Build the docs site before PR #566 and after it, with and without the fix, compare the page dates, and record the results in `verification.md`
- [x] 3.2 Compare the encode garden's page dates with and without the fix, time the history read, and run `mise run parity:publish-bench --corpus encode-garden --check`
- [x] 3.3 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `page-list` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #563.
