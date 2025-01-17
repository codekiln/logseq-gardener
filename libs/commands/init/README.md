- # `lgpm init`
	-
	  > Create or update an existing "Root Garden" environment for Logseq Patch Manager  
	- ## Usage
		-
		  ```sh
		  $ lgpm init
		  ```
		- `lgpm init` sets up the **root** of your Logseq Garden so you can manage “Garden Patches” (sub-graphs). It creates minimal files and directories to help you start organizing patches. You do **not** need a Git or Lerna-based monorepo for basic usage—`lgpm` works with or without it.
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
			  # In a directory where you want to manage your garden patches:
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
				- If you plan to use Node-based workflow for automation:
					- This triggers your `"postinstall": "lgpm sync"` script and copies any necessary files into the appropriate patch directories.
				- If you aren’t using Node at all, you can still invoke `lgpm sync` directly whenever you need to.
		- ### Frequently Asked Questions
			- #### Do I need to use Git in the Logseq Garden **Root** to use `lgpm`?
				- No. Advanced users may choose to do this, but it's not a supported configuration by default.
			- #### Do I need to use Git in the Logseq Garden **Patches** to use `lgpm`?
				- That's the recommended and assumed configuration. This enables
					- a safer use of `lgpm`, since you can go back in time and get the previous revision
					- sharing sub-graphs with others on GitHub
				- Using `lgpm` without having a `git` repository for each of the Garden Patches is an unsupported setup, however, that doesn't mean that it can't be done. For example, you may have a Garden Patch that's only on one machine, and you may choose not to use git for that one patch, but that's not a setup that `lgpm` will test.
			- #### Does `lgpm init` create a a Git repo or `.gitignore` in the Logseq Garden Root?
				- No, but you should run `lgpm init`, which does by default.
				- If you choose to version-control your whole garden, you can run `git init` or clone an existing repo on your own. However, `lgpm init` itself stays out of Git configuration for the Garden Root.
			- #### Can I skip installing dependencies after `lgpm init`?
				- Yes. `lgpm init` does not automatically run any installs. If you want to utilize the `"postinstall"` hook or maintain Node dependencies, run `npm install` (or your preferred package manager’s install command) yourself.
			- #### Does `lgpm init` prompt me for advanced configuration?
				- Not yet. The current version is non-interactive and creates only what’s necessary to get started. Additional prompts or interactive fields may appear in later versions.