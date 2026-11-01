tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Publish Options]], [[lsg/Ref/Selection and Visibility]], [[lsg/Tutorial/Publish Part of a Public Garden]]

- # How To Publish Selected Namespaces
	- ## Overview
		- `lsg publish` turns chosen namespaces of a Logseq Markdown garden into a static website: an index, one HTML page for each published page with relative links between them, and the supported assets those pages use. Open `index.html` in a browser to read it. The garden's files stay unchanged.
		- For a complete walkthrough that adds workshop navigation and podcast listening checkpoints, follow [[lsg/Tutorial/Publish Part of a Public Garden]]. To put the site on the web, see [[lsg/How To/Publish a Garden to GitHub Pages]].
	- ## Prerequisites
		- `lsg`, installed as [[lsg/How To/Install with mise]] shows. In a checkout of this repository, `mise exec -- cargo run --bin lsg -- publish` runs the same command from source.
		- A Logseq Markdown garden. The examples publish parts of the public [encode garden](https://github.com/codekiln/logseq-encode-garden), whose path is in `garden_root`. When ghq manages your clones, `garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"` sets it.
	- ## Steps
		- ### 1. Find the garden's file name format
			- Open the garden's `logseq/config.edn`. If it sets `:file/name-format :triple-lowbar`, use `--filename-format triple-lowbar`. Otherwise use `--filename-format legacy`, for Logseq's default format.
		- ### 2. Choose the namespaces
			- Pass `--include` with each namespace to publish, and `--exclude` with each namespace to leave out. Each option selects the namespace and its descendants, and you can repeat either one.
			- Selection is case-sensitive, and an exclusion always wins. With no `--include`, no page is selected; only journals are published, with `--journals`.
			- Within the selection, the garden's publishing rules decide which pages are published: a page with `public:: false` is withheld, and so is a page without `public:: true`, unless `logseq/config.edn` makes every page public or the garden has no `logseq/config.edn`. [[lsg/Ref/Selection and Visibility]] describes the rules.
		- ### 3. Publish into a new directory
			- Choose an output directory that does not exist yet, whose parent exists, outside the garden. Each run needs a fresh directory.
			- ```sh
			  preview_root="$(mktemp -d "${TMPDIR:-/tmp}/lsg-publishing.XXXXXX")"
			  lsg publish --graph "$garden_root" --output "$preview_root/site" \
			    --filename-format triple-lowbar \
			    --include Logseq/Frontmatter --exclude Logseq/Frontmatter/tags
			  ```
			- `lsg publish` prints the counts and the path of `index.html` on standard output, and a diagnostic for each unsupported construct or unresolved reference on standard error. To keep the diagnostics, append `2> /private/tmp/garden-site-report.txt` to the command. [[lsg/Ref/Publish Options]] describes the report, its JSON form and the exit codes.
		- ### 4. Open the site
			- Open `index.html` in the output directory with a browser.
			- To serve the site over HTTP instead, which search needs, run a static file server and open [http://127.0.0.1:8000/](http://127.0.0.1:8000/):
			- ```sh
			  python3 -m http.server 8000 --bind 127.0.0.1 --directory "$preview_root/site"
			  ```
			- Follow the frontmatter page's selected embeds. The excluded tags explanation stays unavailable.
			- Stop the server with Ctrl-C. If port 8000 is taken, choose another port.
	- ## Examples
		- ### A namespace without one of its branches
			- ```sh
			  lsg publish --graph "$garden_root" --output /private/tmp/garden-site-preview \
			    --filename-format triple-lowbar \
			    --include My/AI --exclude My/AI/Agent
			  ```
			- The command selects `My/AI` and its descendants, and leaves out `My/AI/Agent` and its descendants. Open `/private/tmp/garden-site-preview/index.html`.
		- ### Podcast sessions
			- ```sh
			  lsg publish --graph "$garden_root" --output /private/tmp/gitp-session-preview \
			    --filename-format triple-lowbar \
			    --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu
			  ```
			- Open the generated index, then `GitP/A/Session/24/11/19-Tue`. Its artwork and recording are HTTPS media, so viewing the artwork and listening need network access.
			- The exclusion removes the September 24 session and its asset namespace. Retained production notes stay visible unless their page is withheld. Selecting parts of a page's outline is not supported yet. Simple queries run when the site is built, and a Datalog query shows Logseq's query error.
		- ### From Rust
			- Rust programs call `site::publish_site` or `site::publish_site_with_options` from `logseq-gardener-sdk`. The [publish_site example](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/examples/publish_site.rs) shows the call, and accepts `--include`, `--exclude`, `--stylesheet` and `--homepage`. [[lsg/How To/Use the Rust SDK]] shows how to add the SDK to a project.
	- ## Troubleshooting
		- If `lsg publish` reports that the output already exists, choose a new directory. Refusing existing output also keeps the files of pages excluded since an earlier run out of the new site.
		- If a write fails part way, the new directory may hold partial output. Fix the error, then publish into a fresh directory.
		- If the index is empty or a page is missing, check that an `--include` selects the page with the same letter case, and that the report does not name it as withheld.
		- If a page shows literal text where you expected a link, an embed or other syntax, the report names the construct or reference behind it. Use the diagnostics to choose the syntax and reference gaps that matter for your garden. [[lsg/Ref/Rendering and Assets]] and [[lsg/Ref/Logseq Compatibility]] describe what `lsg` renders.
