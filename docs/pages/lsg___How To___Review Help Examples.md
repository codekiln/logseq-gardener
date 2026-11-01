tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Project Checks]], [[lsg/Ref/Command Line]]

- # How To Review Help Examples
	- ## Overview
		- `mise run clilint:check` checks the behavior of `lsg` automatically, and it reads a saved assessment, [tests/assessments/help.json](https://github.com/codekiln/logseq-gardener/blob/main/tests/assessments/help.json), for the quality of the help examples. The assessment applies only to the help text that Clilint captured for review, the root help `lsg --help`, and CI rejects it when that text changes. After you change the help text in [src/help.rs](https://github.com/codekiln/logseq-gardener/blob/main/src/help.rs), review its examples and refresh the assessment.
		- A change to another command's help, such as `lsg publish help`, leaves the root help and the saved assessment as they were, so `clilint:check` keeps passing. Review that command's examples anyway, and record the review under Recorded reviews.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`. `mise.toml` pins the Clilint revision that `mise install` builds.
		- Clilint's `assess-cli-help` skill, from [codekiln/clilint](https://github.com/codekiln/clilint).
		- The check bundle in `check-bundles/codekiln-help/` includes the hierarchical-help checker, copied from Clilint; [THIRD_PARTY_NOTICES.md](https://github.com/codekiln/logseq-gardener/blob/main/THIRD_PARTY_NOTICES.md) identifies its source.
	- ## Steps
		- ### 1. Build lsg
			- Run `mise run build:release`. `mise run clilint:check` checks `./target/release/lsg`.
		- ### 2. Capture a report
			- Capture a Clilint report with the same target and check bundle that `mise run clilint:check` uses: `./target/release/lsg` and [check-bundles/codekiln-help/clilint.toml](https://github.com/codekiln/logseq-gardener/blob/main/check-bundles/codekiln-help/clilint.toml).
		- ### 3. Review the examples
			- Follow the `assess-cli-help` skill to review the captured examples, and update `tests/assessments/help.json` with the result.
		- ### 4. Check the assessment
			- Run `mise run clilint:check` again. Every check must pass at the standard's full score.
	- ## Recorded reviews
		- Reviews of changes to the publishing help:
			- `--search`: the publication usage includes `--search`, and its behavior explains how search artifacts are generated and loaded on demand. The existing copyable publication example stays valid with search omitted. [[lsg/Ref/Static Search Data]] holds the search API details and exclusions.
			- `--timings`: the publication usage includes `--timings`, and the output section says where the phase times go in human and JSON output. The help examples are unchanged. [[lsg/Ref/Publication Speed]] names the phases and what they measure.
			- `--readable-urls`: the publication usage includes `--readable-urls`, and the behavior section describes the folder layout, what stays at the output root and which titles fail before output creation. The help examples are unchanged. [[lsg/Ref/Publish Options]] describes the option, its layout and its title rules.
			- Documentation links: the publishing help names the site pages that hold its details, such as `lsg/Ref/Static Search Data`, by title and URL, where it named files under `docs/` before. The examples and the root help that Clilint captures are unchanged, so the saved assessment still applies. [[lsg/How To/Link CLI Help to a Docs Page]] describes the links.
	- ## Troubleshooting
		- If `clilint:check` rejects the assessment after a help change, the saved assessment describes the old help text. Review the examples again from step 2.
