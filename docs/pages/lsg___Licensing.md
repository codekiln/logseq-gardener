tags:: [[Diataxis/Explanation]]
see-also:: [[lsg/How To/Update Dependencies]], [[lsg/Ref/Dependencies]], [[lsg/Ref/Release Archives]]

- # Licensing
	- Logseq Gardener uses AGPL-3.0-only. [LICENSE](https://github.com/codekiln/logseq-gardener/blob/main/LICENSE) contains the complete AGPLv3 text. Third-party code and dependencies keep their own licenses, and the repository records each one, so every distributed build carries the notices and the source its licenses require.
	- ## License records
		- [THIRD_PARTY_NOTICES.md](https://github.com/codekiln/logseq-gardener/blob/main/THIRD_PARTY_NOTICES.md) records adapted source: its revision, attribution, modifications and distribution terms.
		- [THIRD_PARTY_LICENSES.txt](https://github.com/codekiln/logseq-gardener/blob/main/THIRD_PARTY_LICENSES.txt) collects the upstream license texts of the locked Cargo packages in one file.
		- The dependency source inventory, [docs/dependency-sources.json](https://github.com/codekiln/logseq-gardener/blob/main/docs/dependency-sources.json), records each locked Cargo package with its source, checksum and declared license.
		- [scripts/notices.py](https://github.com/codekiln/logseq-gardener/blob/main/scripts/notices.py) writes the inventory and the combined license file. `mise run notices:update` runs it, and `mise run notices:check` rejects stale generated files.
		- [[lsg/Ref/Dependencies]] explains why the CLI and the SDK use their direct dependencies, and [[lsg/Parser Adoption]] holds the parser's pinned source and license evidence.
		- [[lsg/How To/Update Dependencies]] keeps these records current when a dependency changes.
	- ## What the AGPL asks of a distribution
		- Each GitHub Release carries the corresponding source for its binaries: the repository at the release tag and every locked dependency source, from which `cargo build --locked --offline` builds without network access. The release workflow creates both archives, and creates no release when it cannot build them. [[lsg/Ref/Release Archives]] lists them.
		- Binary archives include `LICENSE`, `THIRD_PARTY_NOTICES.md`, `THIRD_PARTY_LICENSES.txt` and the dependency inventory.
		- Modified AGPL components require the applicable modification notices and corresponding source.
		- AGPL section 13 asks a modified program that interacts with remote users over a network to offer those users its corresponding source. The current CLI works offline: it prints its help and version, looks up page files and publishes static sites. Future software that offers network interaction must implement the corresponding-source offer for remote users.
