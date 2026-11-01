tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Install with mise]], [[lsg/How To/Publish Selected Namespaces]], [[lsg/Ref/Publish Options]]

- # How To Publish a Garden to GitHub Pages
	- ## Overview
		- Publish a Logseq garden from its GitHub repository to GitHub Pages with a released `lsg`. A GitHub Actions workflow installs `lsg` with mise, deploys the site on every push to `main`, and uploads the site as a preview on every pull request.
		- GitHub Pages serves a repository's site under the repository's name, such as https://codekiln.github.io/logseq-gardener/ for this documentation. Every link `lsg` writes is relative to the site, so the site works under that path. With `--readable-urls`, each page's address is its title, such as `https://OWNER.github.io/REPOSITORY/Notes/Garden/`.
	- ## Prerequisites
		- A Logseq garden in a GitHub repository, with a `pages` or `journals` folder.
		- [mise](https://mise.jdx.dev/) on your computer.
		- Admin access to the repository, to turn on GitHub Pages.
		- The GitHub CLI, `gh`, for the commands that turn on Pages and download a preview. The repository's settings pages and the Actions tab do the same.
	- ## Steps
		- ### 1. Pin `lsg` in the repository
			- In the root of the repository, run:
			- ```sh
			  mise use --pin github:codekiln/logseq-gardener
			  ```
			- mise installs the latest release of `lsg` and records its version in `mise.toml`:
			- ```toml
			  [tools]
			  "github:codekiln/logseq-gardener" = "0.0.8"
			  ```
			- Run `mise x -- lsg --version` to check that it prints the same version.
			- The workflow installs the version in `mise.toml`, so every build uses the same `lsg` until you change that line. Change it in a pull request and read the preview before you merge. Without `--pin`, mise writes `latest`. [[lsg/How To/Install with mise]] has the other ways to install `lsg`.
		- ### 2. Publish the garden on your computer
			- Publish into a new folder named after the repository, outside the garden:
			- ```sh
			  mkdir -p /tmp/pages-preview
			  mise x -- lsg publish --graph . --output /tmp/pages-preview/REPOSITORY \
			    --filename-format triple-lowbar --include Notes \
			    --homepage Notes --search --readable-urls
			  ```
			- Replace `REPOSITORY` with the repository's name, and `Notes` with your namespace and home page.
			- | Option | What it does |
			  | --- | --- |
			  | `--graph .` | Reads the garden in the current folder. |
			  | `--output DIR` | Writes the site to `DIR`, a folder that does not exist yet, outside the garden. Its parent folder must exist. |
			  | `--filename-format triple-lowbar` | Reads file names in the format that `:file/name-format :triple-lowbar` in `logseq/config.edn` sets, such as `Notes___Garden.md` for `Notes/Garden`. Use `legacy` for a garden without that setting. |
			  | `--include Notes` | Publishes the page `Notes` and every page under it, such as `Notes/Garden`. Repeat it for each namespace. |
			  | `--homepage Notes` | Shows the page `Notes` at `index.html` and moves the list of all pages to `pages.html`. |
			  | `--search` | Adds the search dialog and its index. |
			  | `--readable-urls` | Writes each page to a folder named by its title, such as `Notes/Garden/index.html`, and links to it as `Notes/Garden/`. |
			- `lsg` publishes a selected page when it is marked `public:: true`, or when `logseq/config.edn` sets `:publishing/all-pages-public? true`, and leaves out pages marked `public:: false`. [[lsg/Ref/Selection and Visibility]] has the full rules.
			- `lsg` prints a line for each diagnostic, such as a link to a page you did not select or syntax it does not render, and ends with a count. Fix the pages, or select the pages they link.
			- Serve the parent folder, so the site sits under the repository's name as it will on GitHub Pages:
			- ```sh
			  python3 -m http.server --directory /tmp/pages-preview 8000
			  ```
			- Open `http://localhost:8000/REPOSITORY/` and follow a few links and a search result.
		- ### 3. Add the workflow
			- Save this file as `.github/workflows/pages.yml`, with your own `--include` and `--homepage` options:
			- ```yaml
			  name: Pages
			  on:
			    push:
			      branches: [main]
			    pull_request:
			    workflow_dispatch:
			  permissions:
			    contents: read
			  jobs:
			    build:
			      runs-on: ubuntu-latest
			      steps:
			        - uses: actions/checkout@v4
			          with:
			            fetch-depth: 0
			        - uses: jdx/mise-action@v5
			        - name: Publish the garden
			          run: |
			            lsg publish --graph . --output "$RUNNER_TEMP/site" \
			              --filename-format triple-lowbar --include Notes \
			              --homepage Notes --search --readable-urls --format json \
			              | jq -e '.diagnostic_count == 0'
			        - uses: actions/upload-pages-artifact@v5
			          with:
			            path: ${{ runner.temp }}/site
			            retention-days: 14
			    deploy:
			      if: github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')
			      needs: build
			      runs-on: ubuntu-latest
			      permissions:
			        pages: write
			        id-token: write
			      environment:
			        name: github-pages
			        url: ${{ steps.deployment.outputs.page_url }}
			      concurrency:
			        group: pages
			        cancel-in-progress: false
			      steps:
			        - id: deployment
			          uses: actions/deploy-pages@v5
			  ```
			- `fetch-depth: 0` checks out the full history. `lsg` shows each page's first and last commit dates in the list of all pages, and reports a diagnostic when the checkout is a shallow clone, which `actions/checkout` makes by default.
			- `jdx/mise-action` installs the tools in `mise.toml`, `lsg` among them, and puts them on the `PATH`.
			- The site goes to `$RUNNER_TEMP/site`, a folder outside the garden.
			- `--format json` prints the publishing report as JSON, and `jq -e '.diagnostic_count == 0'` fails the step when `lsg` reports a diagnostic. The diagnostics appear in the log above the report. Remove the `jq` line to publish a site that has diagnostics.
			- `actions/upload-pages-artifact` packs the site and uploads it as the artifact `github-pages`. On a pull request this artifact is the preview, kept for `retention-days` days; the default is one day.
			- The `deploy` job runs for pushes and manual runs on `main`. Only this job can write to Pages. Its `pages` concurrency group runs deployments one at a time and lets a running deployment finish.
		- ### 4. Turn on GitHub Pages
			- Set the repository's Pages source to GitHub Actions:
			- ```sh
			  gh api -X POST repos/OWNER/REPOSITORY/pages -f build_type=workflow
			  ```
			- In the browser, open the repository's Settings, choose Pages, and set Source to GitHub Actions.
			- GitHub creates the `github-pages` environment, which accepts deployments from the default branch.
		- ### 5. Read the preview on a pull request
			- Commit `mise.toml` and the workflow on a branch and open a pull request. The Pages workflow publishes the garden, uploads the `github-pages` artifact and skips the deploy job.
			- Download the artifact from the run's summary page, or with `gh`, and unpack it into a folder named after the repository:
			- ```sh
			  gh run list --workflow pages.yml --branch BRANCH
			  gh run download RUN_ID --name github-pages --dir /tmp/pages-pr
			  mkdir /tmp/pages-pr/REPOSITORY
			  tar -xf /tmp/pages-pr/artifact.tar -C /tmp/pages-pr/REPOSITORY
			  python3 -m http.server --directory /tmp/pages-pr 8000
			  ```
			- `gh run list` prints the run's ID. Open `http://localhost:8000/REPOSITORY/` to read the site the pull request would deploy.
		- ### 6. Merge and open the site
			- Merge the pull request. The workflow runs on `main`, and its deploy job publishes the site at `https://OWNER.github.io/REPOSITORY/` and links that address on the run's summary page.
			- To deploy again without a new commit, run the workflow from the Actions tab or with `gh workflow run pages.yml --ref main`.
	- ## Worked examples
		- ### Logseq Gardener's documentation
			- [docs-site.yml](https://github.com/codekiln/logseq-gardener/blob/main/.github/workflows/docs-site.yml) publishes these pages to https://codekiln.github.io/logseq-gardener/ with the jobs above. Its build job builds `lsg` from the commit it publishes, so the documentation and the features it describes land together.
			- The build step runs `mise run docs:site --output site`. The [docs:site task](https://github.com/codekiln/logseq-gardener/blob/main/.mise/tasks/docs/site) runs `cargo build`, then `lsg publish` with `--graph docs`, `--include lsg`, `--include Diataxis`, `--homepage lsg`, `--search` and `--readable-urls`, and fails when the report counts a diagnostic. The garden is the `docs` folder, so `site` at the root of the checkout is outside it.
			- [[lsg/How To/Edit the Documentation]] shows how to build the same site on your computer.
		- ### Ghost in the Patch
			- [gh-pages.yml](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml) publishes the Ghost in the Patch podcast site to https://codekiln.github.io/gitpa/ with a released `lsg`. Its [publishing notes](https://github.com/codekiln/gitpa/blob/main/docs/gardener-publishing.md) describe the build, the preview and the upgrade steps.
			- It downloads the Linux release archive of `lsg` and checks it against a SHA-256 hash pinned in the workflow; to upgrade, change the version and the hash together. `MISE_DISABLE_TOOLS` on the mise step keeps mise from installing the `lsg` that its `mise.toml` declares for local builds.
			- `scripts/build_site.py` prepares a copy of the garden, runs `lsg publish` with an `--include` for each public page, `--homepage`, `--stylesheet` and `--search`, then edits the published pages for the homepage query and the episode player. It reads the published pages from `lsg-site.json` and copies each page to a folder named by its title, the layout that `--readable-urls` writes.
			- It packs the site into a tar file before it uploads the preview with `actions/upload-artifact`, because artifact file names reject characters such as `?` that its page titles contain. `actions/upload-pages-artifact` packs the site the same way.
			- It deploys by pushing the site to a `gh-pages` branch, asks GitHub to build Pages, and waits until the live site reports the new revision. Pages runs Jekyll on a branch, and Jekyll skips folders whose names start with `_`, such as the `_refs` folder `lsg` writes, so the build adds an empty `.nojekyll` file. A site deployed with `actions/deploy-pages` needs no `.nojekyll` file.
	- ## Troubleshooting
		- If `lsg` stops with `output must be outside the source garden`, publish to a folder outside the garden, such as `$RUNNER_TEMP/site` in a workflow.
		- If `lsg` reports `Page dates come from a shallow clone and reflect only the history it holds`, set `fetch-depth: 0` on `actions/checkout`.
		- If a readable build stops and names pages whose titles cannot be folder names, such as two titles that differ only in letter case, rename a page or give it a `title::` property.
		- If the deploy job reports `Failed to create deployment (status: 404)`, turn on GitHub Pages with the GitHub Actions source, as step 4 shows, then run the workflow again on `main`.
