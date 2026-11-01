tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Contribute a Change]], [[lsg/Releases]]

- # Project Checks Reference
	- ## Overview
		- `mise run ci` runs the checks that GitHub Actions runs on every pull request and push to `main`. It runs them in the order below and stops at the first failure. Each check is also a task that runs alone. `mise tasks` lists every task, and `mise run <task> --help` shows a task's arguments.
		- The tasks are executable files in [.mise/tasks](https://github.com/codekiln/logseq-gardener/tree/main/.mise/tasks), and [mise.toml](https://github.com/codekiln/logseq-gardener/blob/main/mise.toml) pins the tools they run.
	- ## Checks in mise run ci
		- | Task | What it checks |
		  | --- | --- |
		  | `mise run format` | Rust formatting, with `cargo fmt --all -- --check`. `mise exec -- cargo fmt` applies the formatting. |
		  | `mise run lint` | Clippy, with warnings denied. |
		  | `mise run test` | The workspace tests of the CLI and the SDK, including an SDK example compiled as a library consumer; the SDK's documentation tests; the tests of the hierarchical-help checker and the parity scripts; and the JavaScript tests in `tests/*.test.cjs`. |
		  | `mise run pr:title-test` | The pull request title check accepts valid Conventional Emoji titles and rejects invalid ones. |
		  | `mise run build:release` | The workspace builds as optimized release binaries. |
		  | `mise run parity:budgets` | The sizes of `lsg` pages on the fixture garden stay within the parity budgets. |
		  | `mise run openspec:validate` | Every OpenSpec change and spec validates strictly. |
		  | `mise run parser:baseline-check` | The mldoc parser comparison baseline. |
		  | `mise run parser:sdk-fixtures-check` | The SDK relationship fixtures match the recorded Logseq OG evidence. |
		  | `mise run openspec:check-skill-versions` | The OpenSpec skills in `.rulesync/skills/` match the version of the managed OpenSpec CLI. |
		  | `mise run rulesync:check` | The generated agent instructions match fresh RuleSync output. |
		  | `mise run notices:check` | The dependency source inventory and the combined license file match the locked dependencies. |
		  | `mise run docs:check` | The docs site builds with no `lsg` diagnostics; the `sh` code blocks in `README.md` run, and its links to the documentation site name pages; every page carries the kind tag its title names; and the remaining Markdown documents have `:hidden` entries and working local links. [[lsg/How To/Edit the Documentation]] describes it. |
		  | `mise run clilint:check` | Command behavior and help navigation against the pinned Clilint standards and the saved help assessment. Every check must pass at its full score. [[lsg/How To/Review Help Examples]] refreshes the assessment. |
		  | `mise run release:check` | The release configuration: the release impact test and the dist configuration check, both described below. |
	- ## Release configuration checks
		- `mise run release:check` runs `mise run release:impact-test` and `mise run dist:check`.
		- The release impact test runs the pinned release-plz binary against temporary Git repositories. It checks feature, fix, breaking and non-release commits for both `0.x` and `1.x` versions, and checks that a change to the SDK alone raises the shared version.
		- dist generates `.github/workflows/release.yml` from [dist-workspace.toml](https://github.com/codekiln/logseq-gardener/blob/main/dist-workspace.toml). `mise run dist:check` fails when the committed workflow differs from what dist generates, and plans a release. After a change to `dist-workspace.toml` or an upgrade of cargo-dist, `mise run dist:generate` regenerates the workflow, and the workflow changes only that way.
	- ## GitHub Actions
		- | Workflow and job | What it runs |
		  | --- | --- |
		  | CI `title` | `mise run pr:title` on the pull request title. Required. |
		  | CI `ci` | `mise run ci`, with the full git history that `lsg` needs to read page dates. Required. |
		  | CI `build-plugins` | The build plugin benchmark on a pinned commit of the encode garden, against the `[build_plugins]` budgets. |
		  | CI `publication-speed` | Cold and warm `lsg publish` builds of a generated Gitpa-sized garden and a pinned encode garden commit, against the recorded publication speed rows. |
		  | Release | On a pull request and on a version tag: `mise run ci`, then every binary archive and the vendor archive, uploaded as workflow artifacts. On a version tag, it then publishes the GitHub Release. |
		- `main` requires the `title` and `ci` checks on a branch that is up to date with `main`.
	- ## Tasks outside mise run ci
		- `mise run docs:links` requests every web link in `README.md`, including its links to the documentation site, and needs network access.
		- `mise run plugins:browser-test`, `mise run properties:browser-test`, `mise run readable:browser-test` and `mise run search:browser-test` check the published site in a browser. The properties test needs Google Chrome.
		- `mise run parser:comparison-check` and `mise run parser:og-check` compare parsers and Logseq relationships, and read pinned source from local ghq checkouts.
