tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Releases]], [[lsg/Licensing]]

- # Release Archives Reference
	- ## Overview
		- Each release on [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) carries binary archives, the corresponding source for those binaries, and a checksum for every archive. The `Release` workflow creates all of them; [[lsg/Releases]] describes the flow.
	- ## Release files
		- | File | Contents |
		  | --- | --- |
		  | `logseq-gardener-aarch64-apple-darwin.tar.xz` | `lsg` for Apple silicon macOS |
		  | `logseq-gardener-x86_64-apple-darwin.tar.xz` | `lsg` for Intel macOS |
		  | `logseq-gardener-x86_64-unknown-linux-gnu.tar.xz` | `lsg` for x86-64 Linux |
		  | `logseq-gardener-x86_64-pc-windows-msvc.zip` | `lsg` for x86-64 Windows |
		  | `source.tar.gz` | The repository at the release tag, including the build scripts and `Cargo.lock` |
		  | `logseq-gardener-<version>-vendor.tar.gz` | Every locked dependency source from `cargo vendor --locked`, plus a `.cargo/config.toml` that points Cargo at them |
		  | `<archive>.sha256` | The SHA-256 checksum of one archive, for each archive above |
		  | `sha256.sum` | The checksums of the binary and source archives in one file |
		  | `dist-manifest.json` | dist's description of the release and its files |
	- ## Binary archive contents
		- Each binary archive holds the `lsg` executable and these files from the repository:
			- `LICENSE`, the complete AGPLv3 text;
			- `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.txt`, the third-party notices and the combined dependency license file;
			- `licenses/`, the license texts that the third-party notices refer to;
			- `README.md`, `CHANGELOG.md` and `CONTRIBUTING.md`; and
			- `docs/`, this documentation as a Logseq graph, with the dependency inventory `docs/dependency-sources.json`.
		- [dist-workspace.toml](https://github.com/codekiln/logseq-gardener/blob/main/dist-workspace.toml) lists the included files and folders in `include`; dist adds the README, the license and the changelog.
		- A reader who extracts an archive can open its `docs` folder as a graph in Logseq 0.10 desktop without network access. [[lsg/How To/Edit the Documentation]] shows how to open the graph.
	- ## Source archives
		- `source.tar.gz` and the vendor archive both extract into the same `logseq-gardener-<version>/` directory. After you extract both, `cargo build --locked --offline` builds `lsg` without network access.
		- The vendor archive is reproducible for a given commit: its entries are sorted, owners and timestamps are fixed to the commit, and gzip omits its own timestamp.
		- [vendor-sources.yml](https://github.com/codekiln/logseq-gardener/blob/main/.github/workflows/vendor-sources.yml) builds the vendor archive with `mise run release:vendor`. `dist-workspace.toml` adds that workflow to the generated `release.yml` through `global-artifacts-jobs`, so pull requests build the archive as well. If the vendor archive cannot be built, the workflow creates no release.
		- `mise run release:vendor` builds the vendor archive locally into `target/distrib`, or into a directory you pass. It requires GNU tar, installed as `tar` or `gtar`.
