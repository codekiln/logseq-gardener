tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Contribute a Change]], [[Diataxis]]

- # How To Edit the Documentation
	- ## Overview
		- The `lsg` documentation is a Logseq graph in the `docs` folder of the repository, and `lsg` publishes the graph as the documentation site. Edit the pages in Logseq 0.10 desktop or in a text editor, then build the site to check them.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
		- Logseq 0.10 desktop, if you want to edit the pages in Logseq.
	- ## Steps
		- ### 1. Open the graph in Logseq
			- In Logseq, open the graph menu at the top of the left sidebar, choose Add new graph, then Choose a folder, and select the `docs` folder of your checkout.
			- Logseq opens the [[lsg]] page. The left sidebar lists the favorites from `docs/logseq/config.edn`, and the Contents page in the right sidebar links the homepage and the kind pages.
			- After you pull or switch branches, choose Re-index from the graph menu, so Logseq reads the changed files.
		- ### 2. Name and tag a page
			- Choose the kind of page from what the reader needs, and give the page a title and a tag from this table. [[Diataxis]] describes each kind.
			- | Reader need | Title | Tag |
			  | --- | --- | --- |
			  | do a task | `lsg/How To/<task>` | `[[Diataxis/How To]]` |
			  | learn by doing | `lsg/Tutorial/<goal>` | `[[Diataxis/Tutorial]]` |
			  | look something up | `lsg/Ref/<subject>` | `[[Diataxis/Reference]]` |
			  | understand why | `lsg/<concept>` | `[[Diataxis/Explanation]]` |
			- Keep `?`, `:`, `#`, `%`, `"` and `'` out of titles, so file names and page addresses stay plain, and make each title differ from the others by more than letter case.
			- Put the tag on the first line of the page as a page property, such as `tags:: [[Diataxis/How To]]`, followed by a blank line and the page's blocks.
			- Logseq saves the page in `docs/pages/` and names the file after the title, with each `/` written as `___`: the page `lsg/How To/Install with mise` is the file `docs/pages/lsg___How To___Install with mise.md`. If you write the file in a text editor, name it the same way.
		- ### 3. Write the page
			- Write Logseq outline Markdown: page properties as `key:: value` lines at the top, every block starting with `- `, and child blocks indented with one tab per level. Keep a table or a fenced code block inside one block.
			- Link another page with a page reference, such as `[[lsg/Ref/Command Line]]`.
			- Link a file in the repository with its GitHub address on `main`, such as `https://github.com/codekiln/logseq-gardener/blob/main/src/help.rs`, or a `tree/main/` address for a folder. `lsg` reads a relative link as an asset.
			- Put an image that a page shows in `docs/assets/`.
			- Leave out queries, raw HTML, drawers, footnotes and macros that `lsg` does not render; `lsg` reports each one as a diagnostic.
		- ### 4. Convert an old document
			- A Markdown document in `docs/` outside `docs/pages/` is not a page yet. A line in `:hidden` in `docs/logseq/config.edn` keeps it out of the graph, such as `"/notes.md"` for `docs/notes.md`.
			- Rewrite the document as pages: a paragraph becomes a block, a list becomes child blocks, and a heading becomes a heading block with its content as children.
			- Delete the old document and remove its line from `:hidden`.
			- Point every link to the old document at the new page. Inside the graph, use a page reference. `README.md` links the page on the documentation site, such as `https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20with%20mise/` for `lsg/How To/Install with mise`. Elsewhere in the repository, name the page title and its file, such as `lsg/How To/Install with mise` in `docs/pages/lsg___How To___Install with mise.md`.
		- ### 5. Build and check the site
			- Run `mise run docs:site`. It builds `lsg` from your checkout, publishes the graph with search into a new folder under `target/docs-site/`, and prints the report and every diagnostic. It fails when `lsg` reports any diagnostic and keeps the folder, so you can open its `index.html` and look.
			- Pass `--output DIR` to publish into a folder of your choice that does not exist yet.
			- Run `mise run docs:check` before you open a pull request; `mise run ci` runs it as well. It builds the site, runs the `sh` code blocks in `README.md`, checks that every README link to the documentation site names a page, and checks every page's tag and the links and `:hidden` entries of the remaining old documents.
			- Write a README command that installs software or changes the reader's configuration, such as `mise use -g`, in a `bash` code block. `docs:check` runs only `sh` blocks, with your checkout's `lsg` first on `PATH`, and fails when one of them runs `mise use`, `mise install`, `mise trust`, `cargo install` or `brew install`.
			- `mise run docs:links` requests every web link in `README.md` from the web. A link to a page that your branch adds fails until the branch merges and the site deploys.
			- `docs:check` also loads the graph with Logseq's own file parser and with the `lsg` SDK, and fails when they disagree about a page, title, alias, page reference, block reference or property. It prints each difference with its file, and with its line for a block. `mise run docs:parser-check` runs this check alone.
			- The first run copies Logseq's parser source from your ghq checkout of [logseq/logseq](https://github.com/logseq/logseq), if you have one, or downloads it from GitHub, and keeps it in `~/.cache/logseq-gardener/`, so later runs skip that step.
	- ## Troubleshooting
		- If `lsg` reports an unresolved page reference, a page links a title that no page has. Correct the title or create the page.
		- If `docs:check` names a document with no `:hidden` entry, add a line for it to `:hidden`, written as its path from `docs/` with a leading `/`.
		- If `docs:check` reports that a README link names no page, correct the address to the page title, with each space written as `%20`, as the page's address on the site shows.
		- If `docs:check` names a page and its expected tag, change the page's `tags::` property to that tag, or rename the page to match its kind.
		- If `docs:check` names a URL that `lsg` help prints, help links a page title that the site no longer has, often because the page was renamed. Change the title in [src/help.rs](https://github.com/codekiln/logseq-gardener/blob/main/src/help.rs) to the page's new title, as [[lsg/How To/Link CLI Help to a Docs Page]] describes.
		- If `docs:check` reports that Logseq and the SDK disagree, the page uses syntax that the SDK reads differently from Logseq, such as a space inside the brackets of a page reference, `[[Release Notes ]]`. Write the reference without the space, such as `[[Release Notes]]`, or [open an issue](https://github.com/codekiln/logseq-gardener/issues) about the difference.
		- If Logseq misses a page or shows a deleted one, choose Re-index from the graph menu.
