tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Set Up Release Automation]], [[lsg/Releases]]

- # How To Rotate the Release App Key
	- ## Overview
		- Release automation signs in as the GitHub App `logseq-gardener-release` with a private key stored in the Actions secret `RELEASE_APP_PRIVATE_KEY`. Rotate the key once a year, or right away if it may have been exposed.
	- ## Prerequisites
		- Access to the App's settings and to the `codekiln` 1Password vault.
		- The 1Password CLI (`op`), fnox with the machine-wide `sync-age` provider described in the [fnox golden path](https://fnox.jdx.dev/guide/golden-path.html), `gh` and `jq`.
	- ## Steps
		- ### 1. Generate a new key
			- On the App's [private keys page](https://github.com/settings/apps/logseq-gardener-release/key_pairs), select Generate a private key. GitHub downloads a new `.pem` file and keeps the old key valid.
		- ### 2. Store the new key
			- Run the store task with the downloaded file:
			- ```sh
			  mise run release:store-app-key ~/Downloads/logseq-gardener-release.<date>.private-key.pem
			  ```
			- The task writes the key to the 1Password Secure Note `github-app-logseq-gardener-release`, refreshes your encrypted fnox cache in the gitignored `fnox.local.toml`, clears the fnox daemon, copies the App ID and the key into the Actions secrets, and deletes the `.pem` file.
			- If the 1Password item does not exist yet, also pass `--app-id <id>`, so the task can create it.
		- ### 3. Delete the old key
			- Delete the old key on the [private keys page](https://github.com/settings/apps/logseq-gardener-release/key_pairs).
	- ## Troubleshooting
		- If the task stops because a tool is missing, install `op`, `fnox`, `gh` or `jq` as the task names it.
		- If the task reports that the file is not a GitHub App private key, pass the `.pem` file that GitHub downloaded from the private keys page.
