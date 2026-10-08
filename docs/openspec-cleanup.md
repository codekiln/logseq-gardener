# Finish an OpenSpec change after merge

Create a cleanup sub-issue under the implementation issue when you submit an implementation PR. After the implementation merges, use the child issue to track synchronization of the requirements, archival of the completed change, and a separate follow-up PR.

## Track the follow-up

Check the implementation issue's existing children and related cleanup PRs before creating a child. Reuse a child for the same OpenSpec change, including a closed child whose cleanup PR already merged. The implementation issue can close when its implementation PR merges; the cleanup child stays open until the follow-up PR merges.

Use GitHub's **Create sub-issue** or **Add existing issue** control on the implementation issue. A link in an issue body alone does not establish the parent-child relationship. Verify the child appears in the parent's sub-issue list.

The child needs:

- The implementation issue and PR, with links and descriptive titles.
- The exact OpenSpec change name and its active directory.
- A dependency stating that the implementation PR must be merged into main before cleanup begins.
- Acceptance criteria: verify the implementation and completed artifacts/tasks, sync and verify every delta capability, archive, validate, and submit a separate follow-up PR against main.
- A completion condition: the cleanup PR has merged and the resulting main specs and archive have been checked on main.

[Issue #23 — Sync and archive the merged Markdown garden loader](https://github.com/codekiln/logseq-gardener/issues/23) is a child of [Issue #14 — Load Markdown garden sources into the Rust SDK](https://github.com/codekiln/logseq-gardener/issues/14). It follows [PR #15 — Markdown garden loader](https://github.com/codekiln/logseq-gardener/pull/15) and names the change `markdown-garden-loader`.

## Verify the merged implementation

Read the source PR's live state before editing specs:

```sh
gh pr view <implementation-pr> --json state,mergedAt,mergeCommit,baseRefName,url
```

Require `MERGED` and a merge commit. Fetch origin and verify that the implementation is present on current `origin/main`. For a PR merged directly into main, confirm that its merge commit is an ancestor of `origin/main`. A stacked PR may have merged into another branch: wait for integration into main, link the integration PR, and verify the integrated commit ancestry and implementation files on main. Record the source and integration PRs in the cleanup PR so reviewers can trace how the implementation reached main. A merged badge, checked task list, or closed issue alone is insufficient evidence.

Create a clean issue-named branch and worktree from current `origin/main` under the registered repository's `.worktrees/` directory. Keep open implementation worktrees available for their contributors. Record the starting main commit and source PR in the cleanup PR description.

Load [openspec-verify-change](../.rulesync/skills/openspec-verify-change/SKILL.md), [openspec-sync-specs](../.rulesync/skills/openspec-sync-specs/SKILL.md), and [openspec-archive-change](../.rulesync/skills/openspec-archive-change/SKILL.md). Pass the child's exact change name through the workflows. Repository instructions require synchronization before archival; run it inline and wait for verification before moving the change.

Run the following commands with the selected store flag when the change uses a store:

```sh
mise exec -- openspec list --json
mise exec -- openspec status --change <change-name> --json
mise exec -- openspec instructions apply --change <change-name> --json
```

Use the CLI's planning root, artifact paths, and context files. Require complete planning artifacts and completed tracked tasks, and check the merged implementation and scenario coverage against the requirements and design. Record verification evidence and unresolved gaps in the cleanup PR. Stop cleanup when implementation, artifacts, or tasks are incomplete.

## Sync requirements before archiving

Read every delta path in `artifactPaths.specs.existingOutputPaths`, preserving each capability's full relative path. Compare the delta with its main spec under the resolved planning root. Fetch the specs artifact instructions before writing main specs and apply the returned rules to their content.

Run the synchronization workflow inline for the selected change. Preserve existing scenarios and requirements outside the delta's intended edits. New capabilities need main specs containing their added requirements; a missing main spec cannot supply a requirement named by a modification or rename.

Compare every delta capability again before archiving:

- Added requirements are present with their scenarios.
- Modified descriptions and scenarios match the delta's intent while unrelated scenarios remain intact.
- Removed requirements are absent.
- Renamed requirements use the new name and the old name is absent.

If the delta removes every requirement in a capability, verify that the merged implementation has retired that capability and record the evidence in the cleanup PR before deleting its empty main spec. Resolve missing inputs, contradictory deltas, or failed comparisons before archival. A successful sync command or summary alone does not establish that every capability matches.

A later merged implementation may supersede part of an earlier delta. Inspect the related PRs and deltas in merge order, preserve the later implemented requirements, and explain the reconciliation in the cleanup PR. Treat an unresolved contradiction as a reason to stop, rather than replacing newer requirements with older text.

Use the archive workflow only after this comparison succeeds. Move the completed change and all its supporting files to the resolved archive directory using the workflow's date naming. For a change with no delta specs, record that finding and verify the remaining completion checks before archiving.

## Review the follow-up PR

Run `mise run openspec:validate`, `mise run rulesync:check`, and `mise run ci`. Inspect the diff: it should contain the selected change's archive move, synchronized main specs, and any necessary links or verification records. Preserve unrelated active changes.

Submit a separate PR against main. Link the cleanup child with `Closes`, and include the implementation PR, exact change name, starting main commit, affected capabilities, verification results, and archive path. Run codekiln-review before handing the PR to the user.

Fetch origin before final review. If main advanced, update the cleanup branch and repeat the comparison for every affected capability, resolve overlaps, and rerun the relevant validation. Reviewers approve the resulting specs and archive together.

After the cleanup PR merges, inspect current main for the resulting specs and archive before closing any remaining child. An existing archive is evidence to investigate: verify its source change, resulting requirements, and cleanup PR before considering the operation complete. Reuse an open cleanup PR and verify a merged one; create no second archive for the same change.

## Find pending cleanup

A contributor or future monitor starts with open cleanup children, including children of closed implementation issues. Read each linked implementation PR's live merge state and reuse any existing cleanup PR. A pending implementation PR remains ineligible.

Reconcile `openspec list --json` on current main with merged implementation PRs and their changed paths. For an active change whose implementation merged but has no cleanup child, create the missing child under its implementation issue after checking existing children and archives. Task completion helps identify candidates; the linked PR establishes whether implementation merged. Ambiguous ownership needs clarification before selecting a change.

Process overlapping changes in implementation merge order and preserve unrelated open work. A monitor should notify only when cleanup becomes actionable, completes, fails, or needs a decision. Scheduling is a separate request.
