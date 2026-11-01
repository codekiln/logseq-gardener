tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Contribute a Change]], [[lsg/Ref/Publishing Cleanup Verification]]

- # How To Finish an OpenSpec Change After Merge
	- ## Overview
		- Each OpenSpec implementation pull request gets a cleanup sub-issue under its implementation issue. After the implementation merges, the cleanup child tracks the rest of the change: syncing its requirements into the main specs, archiving the completed change, and a separate follow-up pull request. Follow these steps from the moment you submit an implementation pull request until its cleanup pull request merges.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`, which include the OpenSpec CLI.
		- `gh`, signed in to an account that can create sub-issues.
		- The OpenSpec skills [openspec-verify-change](https://github.com/codekiln/logseq-gardener/blob/main/.rulesync/skills/openspec-verify-change/SKILL.md), [openspec-sync-specs](https://github.com/codekiln/logseq-gardener/blob/main/.rulesync/skills/openspec-sync-specs/SKILL.md) and [openspec-archive-change](https://github.com/codekiln/logseq-gardener/blob/main/.rulesync/skills/openspec-archive-change/SKILL.md).
	- ## Steps
		- ### 1. Create the cleanup sub-issue
			- Create the cleanup sub-issue when you submit the implementation pull request. Check the implementation issue's existing children and related cleanup pull requests first. Reuse a child for the same OpenSpec change, including a closed child whose cleanup pull request already merged.
			- Use GitHub's Create sub-issue or Add existing issue control on the implementation issue. A link in an issue body alone does not establish the parent-child relationship. Check that the child appears in the parent's sub-issue list.
			- Give the child:
				- the implementation issue and pull request, with links and descriptive titles;
				- the exact OpenSpec change name and its active directory;
				- a dependency stating that the implementation pull request must be merged into `main` before cleanup begins;
				- acceptance criteria: verify the implementation and the completed artifacts and tasks, sync and verify every delta capability, archive, validate, and submit a separate follow-up pull request against `main`; and
				- a completion condition: the cleanup pull request has merged, and the resulting main specs and archive have been checked on `main`.
			- The implementation issue can close when its implementation pull request merges. The cleanup child stays open until the follow-up pull request merges.
			- For example, [Issue #23 Sync and archive the merged Markdown garden loader](https://github.com/codekiln/logseq-gardener/issues/23) is a child of [Issue #14 Load Markdown garden sources into the Rust SDK](https://github.com/codekiln/logseq-gardener/issues/14). It follows [PR #15 Markdown garden loader](https://github.com/codekiln/logseq-gardener/pull/15) and names the change `markdown-garden-loader`.
		- ### 2. Verify the merged implementation
			- Read the source pull request's live state before you edit specs:
			- ```sh
			  gh pr view <implementation-pr> --json state,mergedAt,mergeCommit,baseRefName,url
			  ```
			- Require `MERGED` and a merge commit. Fetch `origin` and check that the implementation is present on current `origin/main`. For a pull request merged directly into `main`, check that its merge commit is an ancestor of `origin/main`.
			- A stacked pull request may have merged into another branch. Wait for its integration into `main`, link the integration pull request, and check the integrated commit's ancestry and the implementation files on `main`. Record the source and integration pull requests in the cleanup pull request, so reviewers can trace how the implementation reached `main`. A merged badge, a checked task list or a closed issue alone is insufficient evidence.
			- Create a clean branch and worktree named after the cleanup issue, from current `origin/main`, under the registered repository's `.worktrees/` directory. Keep open implementation worktrees available for their contributors. Record the starting `main` commit and the source pull request in the cleanup pull request's description.
			- Load the three OpenSpec skills and pass the child's exact change name through their workflows. The repository's instructions require syncing before archiving; run the sync inline and wait for its verification before you move the change.
			- Run these commands, adding the store flag when the change uses a store:
			- ```sh
			  mise exec -- openspec list --json
			  mise exec -- openspec status --change <change-name> --json
			  mise exec -- openspec instructions apply --change <change-name> --json
			  ```
			- Use the planning root, artifact paths and context files that the CLI returns. Require complete planning artifacts and completed tracked tasks, and check the merged implementation and its scenario coverage against the requirements and the design. Record the verification evidence and any unresolved gaps in the cleanup pull request. Stop cleanup when the implementation, the artifacts or the tasks are incomplete.
		- ### 3. Sync the requirements
			- Read every delta path in `artifactPaths.specs.existingOutputPaths`, keeping each capability's full relative path. Compare each delta with its main spec under the resolved planning root. Fetch the specs artifact instructions before you write main specs, and apply the rules they return.
			- Run the sync workflow inline for the selected change. Keep existing scenarios and requirements outside the delta's intended edits. A new capability needs a main spec that holds its added requirements; a missing main spec cannot supply a requirement that a modification or a rename names.
			- Compare every delta capability again before you archive:
				- Added requirements are present with their scenarios.
				- Modified descriptions and scenarios match the delta's intent, and unrelated scenarios remain intact.
				- Removed requirements are absent.
				- Renamed requirements use the new name, and the old name is absent.
			- If the delta removes every requirement in a capability, check that the merged implementation has retired that capability, and record the evidence in the cleanup pull request before you delete its empty main spec.
			- Resolve missing inputs, contradictory deltas and failed comparisons before archiving. A successful sync command or summary alone does not establish that every capability matches.
			- A later merged implementation may supersede part of an earlier delta. Inspect the related pull requests and deltas in merge order, keep the later implemented requirements, and explain the reconciliation in the cleanup pull request. Treat an unresolved contradiction as a reason to stop; newer requirements keep their text.
		- ### 4. Archive the change
			- Run the archive workflow only after the comparison succeeds. It moves the completed change and all its supporting files to the resolved archive directory, named by the workflow's date naming.
			- For a change with no delta specs, record that finding and check the remaining completion conditions before archiving.
		- ### 5. Open the follow-up pull request
			- Run `mise run openspec:validate`, `mise run rulesync:check` and `mise run ci`.
			- Inspect the diff. It holds the selected change's archive move, the synced main specs, and any necessary links or verification records. Unrelated active changes stay as they are.
			- Submit a separate pull request against `main`. Link the cleanup child with `Closes`, and include the implementation pull request, the exact change name, the starting `main` commit, the affected capabilities, the verification results and the archive path. Run the codekiln-review skill before you hand the pull request to the user.
			- Fetch `origin` before the final review. If `main` has advanced, update the cleanup branch, repeat the comparison for every affected capability, resolve overlaps, and rerun the relevant validation. Reviewers approve the resulting specs and the archive together.
		- ### 6. Check the result on main
			- After the cleanup pull request merges, inspect current `main` for the resulting specs and archive before you close any remaining child.
			- An existing archive is evidence to investigate: check its source change, its resulting requirements and its cleanup pull request before you consider the operation complete. Reuse an open cleanup pull request and check a merged one; create no second archive for the same change.
	- ## Find pending cleanup
		- Start with open cleanup children, including children of closed implementation issues. Read each linked implementation pull request's live merge state, and reuse any existing cleanup pull request. A pending implementation pull request is not ready for cleanup.
		- Reconcile `openspec list --json` on current `main` with merged implementation pull requests and their changed paths. For an active change whose implementation merged but has no cleanup child, create the missing child under its implementation issue after checking existing children and archives. Task completion helps identify candidates; the linked pull request establishes whether the implementation merged. Ask who owns a change before you select it when its ownership is unclear.
		- Process overlapping changes in implementation merge order, and leave unrelated open work as it is.
		- A contributor or a future monitor follows these steps. A monitor notifies only when cleanup becomes actionable, completes, fails or needs a decision. Scheduling a monitor is a separate request.
		- [[lsg/Ref/Publishing Cleanup Verification]] is the record of one cleanup that archived several publishing changes together.
