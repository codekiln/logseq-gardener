- # `lgpm init`
	- > Create or update an existing "Root Garden" environment for Logseq Patch Manager  
	- ## Prerequisites
		- **`lgpm`** is currently designed to work **in conjunction with [Lerna](https://lerna.js.org/)**. If you do not have a Lerna-based monorepo set up, please run `lerna init` first.
		- A typical setup flow might look like this:
			-
			  ```sh
			  # 1. Create an empty directory for your garden
			  mkdir ./logseq-garden
			  cd ./logseq-garden

			  # 2. Initialize Lerna (preview changes with --dry-run)
			  npx lerna init --dry-run

			  # 3. If everything looks good, remove --dry-run to finalize:
			  npx lerna init

			  # 4. Run lgpm init to prepare the root garden structure
			  npx lgpm init
			  ```
		- In this workflow, `lgpm init` will add itself as a devDependency in `package.json` (if not already present) and ensure a `"postinstall": "lgpm sync"` script is set up.
	- ## Usage
		-
		  ```sh
		  $ lgpm init
		  ```
		- `lgpm init` sets up the **root** of your Logseq Garden so you can manage “Garden Patches” (sub-graphs). It creates minimal files and directories to help you start organizing patches, intended to be used inside a Lerna-based environment.
		- When `lgpm init` is run, it will:
			- 1. **Create a `patches/` directory (if none exists)**
				- This directory houses each Garden Patch’s Logseq files.
			- 2. **Create or update a `logseq-gardener-config.toml` file**
				- This is a TOML config file for garden-wide (root-level) settings.
				- If an existing file is found, `lgpm init` currently does not merge or overwrite it (merging may come in a later version).
			- 3. **Update the root `package.json`, if present**
				- Ensures a `"postinstall": "lgpm sync"` script is added, so each time you run `npm install`, your patches re-sync automatically.
				- May also add `"lgpm"` as a dependency if it isn’t declared yet, so that `lgpm` commands are available as part of your Node environment.
		- ### Minimal Example
			-
			  ```sh
			  # In a directory where you've already run lerna init:
			  $ lgpm init
			  lgpm info Creating patches/ directory
			  lgpm info Creating or updating logseq-gardener-config.toml
			  lgpm info Updating root package.json to add "postinstall": "lgpm sync"
			  lgpm success Initialized Logseq Garden
			  ```
		- ### Next Steps
			- After `lgpm init`, you typically want to:
			- 1. **Add or register patches** to your new garden.
				- For example:
					-
					  ```sh
					  lgpm register python
					  ```
				- (Future versions of `lgpm` may also support `lgpm create <patchname>`.)
			- 2. **Map dependencies** so that patches can consume knowledge from other patches.
				- For example:
					-
					  ```sh
					  lgpm map python onto work
					  ```
			- 3. **(Optional) Run `npm install`** (or your preferred package manager’s install command)
				- This triggers your `"postinstall": "lgpm sync"` script and copies any necessary files into the appropriate patch directories.
		- ### Frequently Asked Questions
			- #### Do I need to use Git in the Logseq Garden **Root** to use `lgpm`?
				- Not required. Advanced users may choose to version-control the entire garden, but it's optional.
			- #### Do I need to use Git in the Logseq Garden **Patches** to use `lgpm`?
				- That’s the recommended and assumed configuration. This enables:
					- A safer use of `lgpm`, since you can revert to previous commits.
					- Sharing sub-graphs (patches) with others on GitHub.
				- If you choose not to version-control a patch, `lgpm` may still work, but that’s not an officially tested setup.
			- #### Does `lgpm init` create a Git repo or `.gitignore` in the Logseq Garden Root?
				- No. If you choose to version-control your whole garden, you can run `git init` (or clone an existing repo) on your own. However, `lgpm init` itself stays out of Git configuration for the Garden Root.
			- #### Can I skip installing dependencies after `lgpm init`?
				- Yes. `lgpm init` does not automatically run any installs. If you want to utilize the `"postinstall"` hook, run `npm install` or your preferred install command yourself.
			- #### Does `lgpm init` prompt me for advanced configuration?
				- Not yet. The current version is non-interactive and creates only what’s necessary to get started. Additional prompts or interactive fields may appear in later versions.
