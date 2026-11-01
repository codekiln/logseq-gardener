tags:: [[Diataxis/Explanation]]
see-also:: [[lsg/How To/Run a Release]], [[lsg/Ref/Release Archives]], [[lsg/How To/Set Up Release Automation]]

- # Releases
	- A merged pull request whose title marks a feature, a fix or a breaking change leads to a new release of `lsg`. The pull request title decides the release's impact, release-plz prepares the version and its tag, and dist builds the checksummed binaries and publishes them on [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases). Releases v0.0.1 through v0.0.8 are published there, and v0.0.8 is the latest.
	- Release automation runs while the repository Actions variable `RELEASE_ENABLED` is `true`, as it is now. It acts through a GitHub App, `logseq-gardener-release`, whose ID and private key are Actions secrets; the repository holds no release credentials. GitHub starts workflows for a pull request or a tag that the App's token creates, so the release pull request runs CI and the tag runs the release workflow. [[lsg/How To/Set Up Release Automation]] sets up the App, and [[lsg/How To/Rotate the Release App Key]] replaces its key.
	- ## Release flow
		- Pull request CI validates the Conventional Emoji title and runs `mise run ci`.
		- GitHub squash-merges the pull request after the required checks pass, so its title becomes the commit title on `main`.
		- The `Prepare release` workflow, [prepare-release.yml](https://github.com/codekiln/logseq-gardener/blob/main/.github/workflows/prepare-release.yml), runs on `main` with a short-lived token from the App.
		- release-plz creates or updates one release pull request, titled like `🔖 release: prepare v0.0.8`. It raises the version of both packages and adds the release's changes to [CHANGELOG.md](https://github.com/codekiln/logseq-gardener/blob/main/CHANGELOG.md). The workflow enables squash auto-merge on it.
		- After the release pull request passes the same required checks and merges, the next `Prepare release` run creates the `v<version>` tag.
		- The tag starts the dist-generated `Release` workflow, [release.yml](https://github.com/codekiln/logseq-gardener/blob/main/.github/workflows/release.yml). It runs the repository checks, builds every configured target, archives the dependency sources and creates SHA-256 checksums. It completes the GitHub Release only after every required job succeeds, with the version's `CHANGELOG.md` entry as its release notes.
		- release-plz runs in Git-only mode: it publishes nothing to crates.io and creates no GitHub Release. dist chooses no version and creates no tag.
		- [[lsg/How To/Run a Release]] follows these steps and says how to recover when one fails, and [[lsg/Ref/Release Archives]] lists what a release contains.
	- ## Version policy
		- The CLI and the SDK share one release version. release-plz updates both package versions, and the CLI package creates the shared tag. The CLI uses the SDK from this workspace, and neither package is published to crates.io.
		- Before 1.0, features and fixes increment the patch version, and breaking changes increment the minor version. From 1.0, features increment the minor version, fixes increment the patch version, and breaking changes increment the major version.
		- A breaking change carries `!` before the colon of its title, such as `✨ feat!: replace behavior`.
		- Only features, fixes and breaking changes start a release. Documentation, test, CI, style, refactoring, build and other maintenance changes ship with the next release, and the changelog leaves them out. It lists features under Added and fixes under Fixed.
		- [release-plz.toml](https://github.com/codekiln/logseq-gardener/blob/main/release-plz.toml) holds the rules before 1.0 and [release-plz-post-1.toml](https://github.com/codekiln/logseq-gardener/blob/main/release-plz-post-1.toml) from 1.0; `mise run release:config` prints the one that applies to the current version. The release impact test runs the pinned release-plz against temporary repositories for each case, as [[lsg/Ref/Project Checks]] describes.
