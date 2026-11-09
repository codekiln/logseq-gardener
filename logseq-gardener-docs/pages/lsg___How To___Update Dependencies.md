tags:: [[Diataxis/How To]]
see-also:: [[lsg/Licensing]], [[lsg/Ref/Dependencies]], [[lsg/How To/Change the Parser Revision]]

- # How To Update Dependencies
	- ## Overview
		- Every change to the Cargo dependencies also updates the license notices and the dependency source inventory. Follow these steps when you add, remove or upgrade a dependency. To move the parser to another revision, follow [[lsg/How To/Change the Parser Revision]], which adds the parser comparisons to these steps.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
	- ## Steps
		- ### 1. Change the dependency
			- Edit the manifest of the package that uses it: [Cargo.toml](https://github.com/codekiln/logseq-gardener/blob/main/Cargo.toml) for the CLI, or [crates/logseq-gardener-sdk/Cargo.toml](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/Cargo.toml) for the SDK.
			- For a new direct dependency, explain why the package needs it in a comment beside it in the manifest and on [[lsg/Ref/Dependencies]].
			- Commit `Cargo.lock`, and build with `--locked`.
		- ### 2. Refresh the notices
			- Run `mise run notices:update`. It rewrites `docs/dependency-sources.json` and `THIRD_PARTY_LICENSES.txt` from the locked packages, including packages used only on some platforms.
			- Review the changes to the source inventory and the combined license file. Check that each new dependency's upstream license and NOTICE text is in the combined file.
		- ### 3. Review each new dependency
			- For each new dependency or copied source, record its version or revision, upstream source, license, copyright attribution, modifications, required notices and source-distribution obligations. A license identifier alone is not a completed review.
			- Before you distribute a modified dependency, add its modification date, a summary of the changes, its attribution, its exact upstream revision and its source obligations to [THIRD_PARTY_NOTICES.md](https://github.com/codekiln/logseq-gardener/blob/main/THIRD_PARTY_NOTICES.md). [[lsg/Licensing]] explains what a distribution must carry.
		- ### 4. Run the checks
			- Run `mise run notices:check`, which rejects stale generated files, then `mise run ci`.
	- ## Troubleshooting
		- If `notices:check` fails, run `mise run notices:update` and commit the regenerated files.
		- If `notices:update` stops with `review missing license metadata`, the named package declares no license. Review its license by hand before you use it.
		- To find out why a locked package is present, run `cargo tree -i <package>` from the repository root.
