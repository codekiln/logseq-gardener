tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Review Help Examples]], [[lsg/How To/Edit the Documentation]], [[lsg/Ref/Command Line]]

- # How To Link CLI Help to a Docs Page
	- ## Overview
		- `lsg` help sends a reader to this site by naming a page's title and printing its URL, such as `See lsg/Ref/Static Search Data (https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/) for the data contract.` Follow these steps when a help section needs to point at a page. A reader online opens the URL, and a reader offline finds the same title in the `docs` graph of a release archive.
		- `mise run docs:check` fails when help links a URL that the built site does not have, so a renamed or missing page shows up in CI.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
		- The page you want to link, in `docs/pages/`. [[lsg/How To/Edit the Documentation]] shows how to write one.
	- ## Steps
		- ### 1. Choose the page
			- Pick the page that holds the detail the help section leaves out, and copy its title exactly, such as `lsg/Ref/Rendering and Assets`. The title decides the URL.
		- ### 2. Add the link to the help text
			- In [src/help.rs](https://github.com/codekiln/logseq-gardener/blob/main/src/help.rs), build the section's content with `format!` and a named argument set to `docs::page_link` of the title:
			- ```rust
			  content: format!(
			      "... See {rendering} for supported types and rendering limits.",
			      rendering = docs::page_link("lsg/Ref/Rendering and Assets")
			  ),
			  ```
			- `docs::page_link` prints the title followed by its URL in parentheses, and `docs::page_url` gives the URL alone. Both are in [src/docs.rs](https://github.com/codekiln/logseq-gardener/blob/main/src/docs.rs). They join the site's base URL, `https://codekiln.github.io/logseq-gardener/`, and the route that the SDK's `page_urls::readable_url` gives the title, which is the folder `lsg publish --readable-urls` writes the page to.
			- Build every documentation link with these functions. `cargo test` fails when `src/` holds the base URL more than once or when help names a file under `docs/`.
		- ### 3. Test the help text
			- Assert the new link in the test that covers the command, as `cli_plugin_adds_files_and_invalid_manifests_fail_without_output` in [tests/publish.rs](https://github.com/codekiln/logseq-gardener/blob/main/tests/publish.rs) does for the plugins section.
			- Run `cargo test`.
		- ### 4. Check the link against the built site
			- Run `mise run docs:check`. It builds the docs site, then runs the ignored test in [tests/docs_links.rs](https://github.com/codekiln/logseq-gardener/blob/main/tests/docs_links.rs). The test reads the help of every command in human and JSON output and fails for each documentation URL that matches no page `url` in the site's `lsg-site.json`, which [[lsg/Ref/Site Manifest]] describes. The base URL alone stands for the homepage, `index.html`.
			- To run the test against a site you already built, pass the site folder in `LSG_DOCS_SITE`:
			- ```sh
			  mise run docs:site
			  LSG_DOCS_SITE=target/docs-site/<UTC time>/site cargo test --locked --test docs_links -- --ignored
			  ```
		- ### 5. Refresh the help assessment
			- Follow [[lsg/How To/Review Help Examples]], then run `mise run clilint:check`.
	- ## Troubleshooting
		- If the test names a URL, the site has no page with that title. Correct the title in help to the page's current title, or create the page.
		- If help stops with `help links a page with no readable URL`, the title cannot be a folder name, for example because a segment is empty or the title holds a backslash. [[lsg/Ref/Publish Options]] lists the title rules.
		- If the test stops with `set LSG_DOCS_SITE`, it ran without a built site. Run it through `mise run docs:check`, or set `LSG_DOCS_SITE` to a site folder.
