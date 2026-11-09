tags:: [[Diataxis/How To]]
see-also:: [[lsg/Releases]], [[lsg/How To/Rotate the Release App Key]]

- # How To Set Up Release Automation
	- ## Overview
		- Release automation needs repository settings, a GitHub App with two Actions secrets, and the repository Actions variable `RELEASE_ENABLED`. Follow these steps to set it up for `codekiln/logseq-gardener`, or again after the App or its secrets are lost. The repository is set up now, and [[lsg/Releases]] explains what the automation does.
	- ## Prerequisites
		- Admin access to `codekiln/logseq-gardener` and to the `codekiln` account's GitHub Apps.
		- To store the key with the repository's task: the 1Password CLI (`op`), fnox with the machine-wide `sync-age` provider described in the [fnox golden path](https://fnox.jdx.dev/guide/golden-path.html), `gh` and `jq`.
	- ## Steps
		- ### 1. Configure the repository
			- Configure squash merges with the pull request title as the commit title, and enable auto-merge.
			- Protect `main`: require the `title` and `ci` checks, on a branch that is up to date with `main`.
			- Apply these settings before you set `RELEASE_ENABLED`.
		- ### 2. Create the GitHub App
			- Create a GitHub App owned by `codekiln` and install it only on `codekiln/logseq-gardener`.
			- Grant these repository permissions:
				- Contents: read and write
				- Pull requests: read and write
				- Metadata: read-only, which GitHub grants automatically
			- The App needs these permissions alone; leave organization, administration, issue, deployment, package and workflow permissions unset.
		- ### 3. Store the App ID and private key
			- On the App's settings page, note the App ID and generate a private key. GitHub downloads it as a `.pem` file.
			- Run the store task with the key file and the App ID:
			- ```sh
			  mise run release:store-app-key ~/Downloads/logseq-gardener-release.<date>.private-key.pem --app-id <id>
			  ```
			- The task creates the 1Password Secure Note `github-app-logseq-gardener-release` in the `codekiln` vault, with the App ID in its `app-id` field and the private key in its concealed `private-key` field. It copies both values into the Actions repository secrets `RELEASE_APP_ID` and `RELEASE_APP_PRIVATE_KEY`, then deletes the `.pem` file.
			- [fnox.toml](https://github.com/codekiln/logseq-gardener/blob/main/fnox.toml) refers to the 1Password fields under its `release` profile. It holds references only.
			- Without the task, add the two Actions secrets by hand: `RELEASE_APP_ID` holds the App ID shown on the App settings page, and `RELEASE_APP_PRIVATE_KEY` holds the complete PEM private key. Delete the downloaded key file after you confirm the secret is stored.
		- ### 4. Turn on release preparation
			- Set the repository Actions variable:
			- ```sh
			  gh variable set RELEASE_ENABLED --body true --repo codekiln/logseq-gardener
			  ```
			- The next push to `main` runs `Prepare release`. GitHub Actions exchanges the App's secrets for a short-lived installation token; the workflows use no personal access token.
