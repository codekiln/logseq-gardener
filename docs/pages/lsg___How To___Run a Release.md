tags:: [[Diataxis/How To]]
see-also:: [[lsg/Releases]], [[lsg/Ref/Release Archives]]

- # How To Run a Release
	- ## Overview
		- Releases run by themselves after a pull request merges. Follow these steps to take a change from its pull request to published binaries, to start release preparation without waiting for a merge, and to recover when a step fails. [[lsg/Releases]] explains the flow and the version policy.
	- ## Prerequisites
		- Release automation set up as [[lsg/How To/Set Up Release Automation]] describes, with the repository Actions variable `RELEASE_ENABLED` set to `true`.
		- `gh`, signed in to an account that can run the repository's workflows.
	- ## Steps
		- ### 1. Merge a pull request that releases
			- Give the pull request a title that starts a release: `✨ feat: ...`, `🩹 fix: ...`, or any type with `!` before the colon for a breaking change. Check the title with `mise run pr:title '<title>'`.
			- GitHub squash-merges the pull request after the required `title` and `ci` checks pass.
		- ### 2. Let the release pull request merge
			- The merge starts the `Prepare release` workflow. release-plz creates or updates the release pull request, `🔖 release: prepare v<version>`, and the workflow enables squash auto-merge on it.
			- To start release preparation without waiting for a merge, run:
			- ```sh
			  gh workflow run prepare-release.yml --repo codekiln/logseq-gardener
			  ```
			- The workflow runs only on `main`.
			- Read the new version and the `CHANGELOG.md` entry in the release pull request. The pull request merges by itself after it passes the same required checks.
		- ### 3. Check the published release
			- After the release pull request merges, `Prepare release` runs again and creates the `v<version>` tag. The tag starts the `Release` workflow, which runs the repository checks, builds every target, archives the dependency sources and creates the checksums.
			- When every job succeeds, the workflow completes the GitHub Release with the files that [[lsg/Ref/Release Archives]] lists. List the releases with:
			- ```sh
			  gh release list --repo codekiln/logseq-gardener
			  ```
	- ## Troubleshooting
		- If `Prepare release` fails, correct its configuration or credentials, then rerun the failed `Prepare release` run. release-plz updates the existing release pull request instead of opening another one.
		- If a required check fails on the release pull request, the pull request stays open and no tag is created.
		- If a dist build or pre-publish check fails, its tag stays in Git history and the workflow completes no GitHub Release. Fix the failure and rerun the failed `Release` run, with `gh run rerun <run-id> --failed`. Leave the existing version tag where it is.
		- If the App can no longer create a token, replace its key as [[lsg/How To/Rotate the Release App Key]] describes.
