# Verification: keep-page-dates-across-renames

Checked on 2026-11-09 on an Apple M2 Pro with git 2.56.0. "Before the fix" is `lsg` built from `origin/main` at `5b9a5fe`; "with the fix" is this branch.

## SDK tests

These tests in `crates/logseq-gardener-sdk/src/git_dates.rs` fail on `origin/main` and pass now:

| Test | On `origin/main` |
| --- | --- |
| `a_renamed_page_file_keeps_its_first_commit_time` | created 4000, the last rename, instead of 1000 |
| `a_renamed_graph_folder_keeps_page_dates` | created and updated 3000, the folder move, instead of 1000 and 2000 |
| `a_page_created_where_another_was_renamed_away_starts_its_own_history` | the renamed page is created at 2000 instead of 1000 |
| `a_file_name_with_quotes_has_dates` | no dates |

## Docs site before and after PR #566

The docs site was built with the options of `mise run docs:site` three times: from `docs/` at `e7dfa16`, the commit before PR #566, with `lsg` from before the fix; from `logseq-gardener-docs/` at `5b9a5fe` with `lsg` from before the fix; and from `logseq-gardener-docs/` with the fix. The dates come from `_pages/list.json`.

| Build after PR #566 | Pages with both dates as before PR #566 | Created date differs | Updated date differs |
| --- | --- | --- | --- |
| Before the fix | 0 of 89 | 89 (all 2026-11-09) | 89 (all 2026-11-09) |
| With the fix | 84 of 89 | 0 | 5 |

PR #566 edited the text of the 5 pages whose updated date differs, so their updated date is the move commit: `lsg/How To/Edit the Documentation`, `lsg/How To/Install a Release Archive`, `lsg/How To/Link CLI Help to a Docs Page`, `lsg/Ref/Release Archives` and `lsg/Static Publishing`.

## Encode garden

At encode garden commit `66b81e9543`, published with `--include-all --journals`, 18,959 entries:

| Change with the fix | Entries |
| --- | --- |
| Earlier created date: history before a rename | 757 |
| Earlier updated date: the last commit was a rename with identical text | 107 |
| Later created date: a new page at a renamed page's old path | 2 |
| Dates for the first time: a file name with a double quote | 7 |
| Later updated date | 0 |

The pages with a later created date are `Logseq/Entity/Podcast`, which was renamed to `Logseq/Entity/Podcast/Episode` before a new `Logseq/Entity/Podcast` page was written, and `Person/Hironao OTSUBO/GitHub/ghq`.

## Speed

| Measure | Before the fix | With the fix |
| --- | --- | --- |
| `GitDates::read` on the encode garden, fastest of 5 | 3.14 s | 2.08 s |
| `history` phase of an encode garden build (waiting after render) | under 0.1 s in recorded builds | 0.00 s |
| The `git log` call alone on the docs garden, median of 5 | 0.03 s | 0.11 s |

`mise run parity:publish-bench --corpus encode-garden --runs 3 --check` with the fix: fastest warm build 19.09 s, within the 23.31 s limit set by the recorded row for an Apple M2 Pro (19.42 s plus 20%); peak memory 3,406 MB, within its budget.
