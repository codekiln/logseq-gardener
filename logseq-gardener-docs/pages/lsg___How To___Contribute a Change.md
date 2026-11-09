tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Project Checks]], [[lsg/How To/Edit the Documentation]], [[lsg/Architecture]]

- # How To Contribute a Change
	- ## Overview
		- Take a change to Logseq Gardener from its plan to a merged pull request: plan it in OpenSpec, keep the agent instructions current with RuleSync, pass the project checks, and open a pull request whose title says what kind of change it is.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with the pinned tools installed by `mise install`.
		- `mise tasks` lists the project's tasks, and `mise run <task> --help` shows a task's arguments.
	- ## Steps
		- ### 1. Plan the change in OpenSpec
			- Plan the change in OpenSpec, and keep its requirements and their verification together. A change lives in [openspec/changes](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes) until it is archived, and the main specs live in [openspec/specs](https://github.com/codekiln/logseq-gardener/tree/main/openspec/specs).
			- Coding agents load the matching OpenSpec skill from [.rulesync/skills](https://github.com/codekiln/logseq-gardener/tree/main/.rulesync/skills) before they edit an OpenSpec artifact.
			- `mise run openspec:validate` validates every change and spec strictly.
		- ### 2. Make the change
			- [[lsg/Architecture]] explains which package does what, and [[lsg/Parser Adoption]] records the parser decision.
			- If the change edits these pages, follow [[lsg/How To/Edit the Documentation]].
			- If it changes the help text of `lsg`, review the help examples as [[lsg/How To/Review Help Examples]] describes.
			- If it adds or updates a dependency, follow [[lsg/How To/Update Dependencies]].
		- ### 3. Update the agent instructions with RuleSync
			- The instructions for coding agents live in [.rulesync](https://github.com/codekiln/logseq-gardener/tree/main/.rulesync). RuleSync generates each tool's files from it, such as `AGENTS.md` and `CLAUDE.md`, which git ignores. The repository's git hooks regenerate them after a pull or a checkout.
			- After you edit `.rulesync/`, run:
			- ```sh
			  mise exec -- rulesync generate
			  mise run rulesync:check
			  ```
			- Edit the source in `.rulesync/`; RuleSync overwrites the generated files.
		- ### 4. Run the checks
			- Run `mise run ci`. It runs the same checks as GitHub Actions, which [[lsg/Ref/Project Checks]] lists.
			- Apply Rust formatting with `mise exec -- cargo fmt`.
		- ### 5. Open the pull request
			- Give the pull request a Conventional Emoji title, such as `✨ feat: add page lookup`, `🩹 fix: correct output` or `📝 docs: explain help`. Check it with `mise run pr:title '<title>'`.
			- GitHub squash-merges the pull request with its title as the commit title, and the title decides whether the merge starts a release. [[lsg/Releases]] explains the release flow and the version policy.
			- For an OpenSpec implementation pull request, create a cleanup sub-issue under its implementation issue before you hand the pull request over for review.
		- ### 6. Finish the OpenSpec change after merge
			- After the implementation merges, follow [[lsg/How To/Finish an OpenSpec Change After Merge]] to sync the requirements, archive the completed change and submit a separate follow-up pull request.
