# `lgpm init`

> Create or update an existing "Root Garden" environment for Logseq Patch Manager

## Usage

```sh
$ lgpm init
```

`lgpm init` sets up the **root** of your Logseq Garden so you can manage “Garden Patches” (sub-graphs) under a shared monorepo structure. This command assumes you have already:
1. Initialized a Git repository (e.g., via `git init` or a cloned project).
2. Run `lerna init` (and any other monorepo configuration you need).

When `lgpm init` is run, it will:

1. **Create a `patches/` directory (if none exists)**  
   - This directory houses each Garden Patch’s Logseq files.

2. **Create or update a `logseq-gardener-config.toml` file**  
   - This is a TOML config file for garden-wide (monorepo-wide) settings.
   - If an existing file is found, `lgpm init` will not merge or overwrite its content for now. (A prompt or merging feature may come in a later version.)

3. **Update the root `package.json`, if present**  
   - Ensures a `"postinstall": "lgpm sync"` script is added, so each time you run `npm install`, your patches remain in sync.
   - May also add `"lgpm"` as a dependency if it isn’t declared yet, so that `lgpm` commands are available in the monorepo.

### Minimal Example

```sh
# After you have a repo and have run 'lerna init':
$ lgpm init
lgpm info Creating patches/ directory
lgpm info Creating or updating logseq-gardener-config.toml
lgpm info Updating root package.json to add "postinstall": "lgpm sync"
lgpm success Initialized Logseq Garden
```

### Next Steps

After `lgpm init`, you typically want to:
1. **Add or register patches** to your new garden.  
   - For example:
     ```sh
     lgpm register python
     ```
   - (Future versions of `lgpm` may also support `lgpm create <patchname>`.)

2. **Map dependencies** so that patches can consume knowledge from other patches.  
   - For example:
     ```sh
     lgpm map python onto work
     ```

3. **Run `npm install`** (or your preferred package manager’s install command) to trigger the `"postinstall": "lgpm sync"` script, copying any necessary files into the correct patch directories.

> **Note**: The above features are in **early prototype** form. More prompts and additional config options may appear in later versions of `lgpm init`.

### Frequently Asked Questions

#### Does `lgpm init` create a Git repo or `.gitignore`?

No. We currently assume your repo is already managed by Git (and possibly configured with Lerna). Future versions may add more automation, but for now we recommend running `git init` or cloning an existing project first.

#### Can I skip the install step?

Yes. `lgpm init` does not currently auto-run `npm install`, so no special skip flag is needed. After init, run installs (if any) manually.

#### Does `lgpm init` prompt me for advanced configuration?

Not yet. The current version is non-interactive and creates only what’s necessary to get started.

---

For more information, check out [Logseq](https://logseq.com/) and watch for updates on `lgpm`’s [GitHub repository](#). 
