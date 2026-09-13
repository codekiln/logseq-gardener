# Automated releases

Merging an ordinary pull request is the final recurring release action. The pull request title determines release impact, release-plz prepares and tags the version, and dist publishes the checksummed binaries.

The foundation includes release automation; publication remains disabled until the maintainer configures the App and sets the repository Actions variable `RELEASE_ENABLED=true`. No release credentials are stored in this repository.

## One-time GitHub setup

Configure squash merges with the pull request title as the commit title, enable auto-merge, and require `title` and `ci` on an up-to-date branch of `main`. These repository settings must be applied before release activation.

Create a GitHub App owned by `codekiln`, install it only on `codekiln/logseq-gardener`, and grant these repository permissions:

- Contents: read and write
- Pull requests: read and write
- Metadata: read-only, which GitHub grants automatically

The App does not need organization, administration, issue, deployment, package, or workflow permissions. Generate a private key, then add these Actions repository secrets:

- `RELEASE_APP_ID`: the App ID shown on the App settings page
- `RELEASE_APP_PRIVATE_KEY`: the complete PEM private key

Delete the downloaded private-key file after confirming the secret is stored. GitHub Actions exchanges these values for a short-lived installation token; workflows do not use a personal access token.

## Release flow

1. Pull request CI validates the Conventional Emoji Commit title and runs `mise run ci`.
2. GitHub squash-merges the pull request after the required checks pass.
3. `prepare-release.yml` runs on `main` with a short-lived GitHub App token.
4. release-plz creates or updates one release pull request and the workflow enables squash auto-merge.
5. After that pull request passes the same required checks and merges, release-plz creates the `v<version>` tag.
6. The dist-generated `release.yml` workflow runs the repository checks, builds all configured targets, creates archives and SHA-256 checksums, and completes the GitHub Release only after every required job succeeds.

release-plz runs in Git-only mode and does not publish to crates.io or create a GitHub Release. Dist does not choose versions or create tags.

## Local checks and maintenance

Run the complete release configuration checks with:

```sh
mise run release:check
```

The release impact test invokes the pinned release-plz binary against temporary Git repositories. It checks feature, fix, breaking, and non-release commits for both `0.x` and `1.x` versions.

After changing `dist-workspace.toml` or upgrading cargo-dist, regenerate and verify the generated workflow:

```sh
mise run dist:generate
mise run dist:check
```

Do not edit `.github/workflows/release.yml` by hand.

## Recovery

If preparation fails, rerun the failed `Prepare release` workflow after correcting its configuration or credentials. Release-plz updates the existing release pull request instead of opening duplicates.

If a required check fails, the release pull request remains open and no tag is created. If a dist build or pre-publish check fails, its tag remains in Git history but the workflow does not complete a GitHub Release. Fix the failure and rerun the failed release workflow; do not replace or move an existing version tag.

## Version policy

Before 1.0, features and fixes increment the patch version; breaking changes increment the minor version. From 1.0, features increment minor, fixes increment patch, and breaking changes increment major. Documentation-only changes do not trigger a release. The release impact test executes the pinned release-plz against temporary repositories for each case.

## Distribution materials

Cargo-dist builds Apple silicon and Intel macOS, x86-64 Linux, and x86-64 Windows archives and SHA-256 checksums. Archives include the project license, third-party notices, dependency licenses, and dependency inventory. Follow [licensing](licensing.md) to prepare exact dependency sources and retain the generated build configuration with each release.
