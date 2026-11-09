tags:: [[Diataxis/How To]]
see-also:: [[lsg/Parser Adoption]], [[lsg/How To/Update Dependencies]]

- # How To Change the Parser Revision
	- ## Overview
		- The SDK manifest, [crates/logseq-gardener-sdk/Cargo.toml](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/Cargo.toml), pins lsdoc to one git revision, and [THIRD_PARTY_NOTICES.md](https://github.com/codekiln/logseq-gardener/blob/main/THIRD_PARTY_NOTICES.md) records it. Follow these steps to move the SDK to another lsdoc revision and show that parsing still matches Logseq. [[lsg/Parser Adoption]] explains why the SDK uses lsdoc.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
		- The ghq checkouts that the [parser comparison guide](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/parser-comparison-evidence/experiments/parser-comparison/README.md) lists, at the revisions it names.
	- ## Steps
		- ### 1. Rerun the comparisons
			- Rerun the [fixture comparison](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/parser-comparison-evidence/experiments/parser-comparison/README.md) with `mise run parser:comparison-check`, and the public-garden comparisons with `mise run parser:garden-comparison docs`, `mise run parser:garden-comparison encode` and the [Pengx corpus script](https://github.com/codekiln/logseq-gardener/blob/main/experiments/parser-corpus/README.md).
			- Update the evidence on [[lsg/Parser Adoption]] with the new results.
		- ### 2. Pin the new revision
			- Pin the new revision in the SDK manifest, update [[lsg/Ref/Dependencies]], and commit the resulting `Cargo.lock`.
		- ### 3. Refresh the notices
			- Run `mise run notices:update`, then inspect the package inventory and the license collection. Gardener's resolved versions can differ from lsdoc's lockfile.
			- Record the upstream revision, the attribution and the modification status in `THIRD_PARTY_NOTICES.md`, and check that the dependency's license and attribution files are preserved.
		- ### 4. Run the checks
			- Run `mise run notices:check` and `mise run ci`.
			- Whenever you distribute builds, include the source materials that [[lsg/Licensing]] describes.
