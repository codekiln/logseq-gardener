tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Install a Release Archive]], [[lsg/How To/Build from Source]]

- # How To Install with mise
	- ## Overview
		- [mise](https://mise.jdx.dev/) installs `lsg` from the archives that [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) publish for each version. Install `lsg` for every project on your computer, or for one project, whose `mise.toml` then lists it.
	- ## Prerequisites
		- mise, installed and activated in your shell, as mise's [Getting Started](https://mise.jdx.dev/getting-started.html) guide shows.
		- A computer that releases build `lsg` for: macOS on Apple silicon or Intel, Linux on x86-64, or Windows on x86-64.
	- ## Steps
		- ### 1. Install lsg
			- Install `lsg` for every project on this computer, or for the project in the current folder:
			  id:: 23ecf0c0-bef8-45bf-8fa8-26a273de5901
				- ```sh
				  # Every project: adds lsg to your global mise config
				  mise use -g github:codekiln/logseq-gardener
				  # The project in the current folder: adds lsg to its mise.toml
				  mise use github:codekiln/logseq-gardener
				  ```
			- Each command installs the newest release and adds `"github:codekiln/logseq-gardener" = "latest"` to the `[tools]` table of its config file. The global config is usually `~/.config/mise/config.toml`.
			- If a version of `lsg` is already installed, mise uses that version. Write `github:codekiln/logseq-gardener@latest` to install the newest release.
			- Commit the project's `mise.toml`. Everyone who works on the project then gets `lsg` from `mise install`.
		- ### 2. Check the install
			- Run `lsg --version`. It prints the version mise installed, such as `lsg 0.0.8`.
		- ### 3. Pin a version
			- Write the version after `@` to install that version and record it in the config file:
				- ```sh
				  mise use github:codekiln/logseq-gardener@0.0.8
				  ```
			- The project's `mise.toml` then holds `"github:codekiln/logseq-gardener" = "0.0.8"`, and `mise install` installs 0.0.8 for everyone who works on the project.
			- `mise use --pin github:codekiln/logseq-gardener` records the version that `latest` resolves to, such as `0.0.8`. Add `-g` to either command to pin the version of the global install.
		- ### 4. Run one version once
			- `mise exec`, or its short form `mise x`, runs a version of `lsg` without adding it to a config file. mise installs the version first if it is missing.
				- ```sh
				  mise x github:codekiln/logseq-gardener@0.0.8 -- lsg --version
				  ```
			- The command prints `lsg 0.0.8`.
	- ## Checksums
		- Each release holds an archive for each platform and, beside it, a `.sha256` file with the archive's SHA-256 checksum, such as `logseq-gardener-aarch64-apple-darwin.tar.xz.sha256`.
		- mise takes the archive's expected checksum from the SHA-256 digest that GitHub's API reports for each file of a release. It reads that release data from [mise-versions](https://mise-versions.jdx.dev), a shared cache of public GitHub release data, or from GitHub itself. When the release data has no digest for the archive, mise downloads the archive's `.sha256` file and reads the checksum from it.
		- After the download, mise computes the archive's SHA-256 hash and compares it with the expected checksum. When they differ, mise stops with a checksum mismatch error before it unpacks the archive, and `lsg` stays uninstalled.
		- In a project that turns on lockfiles with `lockfile = true` under `[settings]` in `mise.toml`, `mise use` writes `mise.lock`, which records the archive address and checksum for each platform. Later installs in that project compare the download with the checksum in `mise.lock`.
		- mise also looks for GitHub artifact attestations and SLSA provenance for the archive. Releases of `lsg` publish neither, so the checksum is what mise verifies.
		- To check an archive yourself, follow [[lsg/How To/Install a Release Archive]].
	- ## Troubleshooting
		- If the shell reports `lsg: command not found`, mise is not activated in that shell. Activate it as mise's Getting Started guide shows, or run `lsg` through mise, as in `mise exec -- lsg --version`.
		- If mise reports a checksum mismatch, the downloaded archive differs from the checksum in the release data or in `mise.lock`, and mise has installed nothing. Report the error message in an [issue](https://github.com/codekiln/logseq-gardener/issues).
