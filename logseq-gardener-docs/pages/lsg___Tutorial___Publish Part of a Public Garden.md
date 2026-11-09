tags:: [[Diataxis/Tutorial]]
see-also:: [[lsg/How To/Publish Selected Namespaces]], [[lsg/Ref/Selection and Visibility]]

- # Tutorial: Publish Part of a Public Garden
	- ## What You'll Create
		- A website on your own computer that holds two namespaces of codekiln's public Logseq garden, with one workshop and one podcast session left out. You will follow references between workshop notes and listen to a GitP episode on it.
	- ## Prerequisites
		- mise, installed and activated in your shell, as mise's [Getting Started](https://mise.jdx.dev/getting-started.html) guide shows.
		- git and Python 3.
		- A network connection and about 400 MB of free disk space for the garden.
	- ## Learning Goals
		- Install `lsg` for one folder with mise.
		- Publish chosen namespaces of a garden with `--include`, and leave subtrees out with `--exclude`.
		- Read the publish report and its diagnostics.
		- Check what a published site shows and what it leaves out.
	- ## Steps
		- ### 1. Install lsg in a tutorial folder
			- We start with a folder for the tutorial and install `lsg` in it:
				- ```sh
				  mkdir lsg-tutorial
				  cd lsg-tutorial
				  mise use github:codekiln/logseq-gardener
				  ```
			- Notice the new `mise.toml` in the folder: it lists `lsg`, and mise puts `lsg` on your PATH whenever you work in this folder. At the next prompt, check the version:
				- ```sh
				  lsg --version
				  ```
			- You will see the version mise installed, such as `lsg 0.0.8`. [[lsg/How To/Install with mise]] describes the other ways to install it.
		- ### 2. Get the garden
			- Clone the public garden, [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden), into the folder:
				- ```sh
				  git clone --depth 1 https://github.com/codekiln/logseq-encode-garden.git garden
				  ```
			- `--depth 1` downloads the latest commit alone. The garden takes about 400 MB of disk space.
			- Notice the garden's file names, such as `garden/pages/GitP___A___Session.md`. Its `logseq/config.edn` sets `:file/name-format :triple-lowbar`, so each `___` in a file name stands for a `/` in the page's title.
		- ### 3. Publish two namespaces
			- Publish the workshop notes under `AI/ES/25/ws` and the podcast sessions under `GitP/A/Session`, leaving out the third workshop and the September 24 session:
				- ```sh
				  lsg publish --graph garden --output site \
				    --filename-format triple-lowbar \
				    --include AI/ES/25/ws --exclude AI/ES/25/ws/3 \
				    --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu \
				    --format json > result.json 2> report.txt
				  cat result.json
				  ```
			- You will see the report in `result.json`: the counts of published `pages` and `assets`, the `withheld_pages`, the `diagnostic_count`, and `index_path`, the site's front page.
			- `report.txt` lists the diagnostics, one per line. Most name a page reference or tag whose page lies outside the selection. One says that page dates come from a shallow clone, since the clone holds a single commit.
			- Notice how the selection works. `--include` and `--exclude` take a page title, and a title selects its descendants, the pages whose titles continue after a `/`. An exclusion wins over an inclusion, and matching is case-sensitive.
			- `lsg publish` writes into a folder that does not exist yet. To publish again, delete `site` or pass another `--output` folder.
		- ### 4. Serve the site
			- Serve the `site` folder to your own computer:
				- ```sh
				  python3 -m http.server 8000 --bind 127.0.0.1 --directory site
				  ```
			- Open `http://127.0.0.1:8000/` in a browser while the server runs. If port 8000 is in use, choose another port in both the command and the address.
			- The server serves only the generated site, and only on this computer's loopback interface. Hosted artwork and recordings still load from their own HTTPS hosts.
		- ### 5. Follow references between workshop notes
			- The front page lists all pages. Open `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`.
			- Below the heading `2/3 - Person/Mike Christensen`, follow the block reference that begins "On my left was". The page changes to `AI/ES/25/ws/1/Building Agents with Model Context Protocol` and scrolls to the note it references. Notice that the link's label is a short excerpt of that note.
			- Go back and follow `AI Engineer Summit 2025 NYC Workshops`, in the first heading, to the workshop schedule on `AI/ES/25/ws`. Go back again and follow `AIES 25 WS 1 - Building Agents with MCP - Mahesh Murag` to the MCP note. Notice that both names are aliases, and each link opens the selected page that declares the alias.
		- ### 6. Listen to an episode
			- Return to the front page and open `GitP/A/Session/24/11/19-Tue`. You will see the episode's artwork and audio controls. Play the recording, then pause it.
			- Listening needs access to the media host and a browser that plays the recording's format. "Open recording" follows the hosted file, and the host and browser decide whether it plays or downloads.
		- ### 7. Check what the selection left out
			- Search the front page's list for `AI/ES/25/ws/3` and for `GitP/A/Session/26/09/24-Thu`. Neither appears, and the site holds none of the assets that only those pages use.
			- Notice that leaving a namespace out removes its pages, while text on the published pages stays as written: a workshop summary can still name the excluded workshop. A link to a missing or excluded page keeps its label, and `report.txt` lists it. Read the pages you publish before you publish anything sensitive.
			- This garden sets `:publishing/all-pages-public? true`, so `lsg` publishes every selected page apart from those with a `public:: false` property anywhere in their outline. It withholds each of those pages whole, and `withheld_pages` counts them. [[lsg/Ref/Selection and Visibility]] gives the full rules.
			- Stop the server with Ctrl-C when you finish reading.
	- ## What You've Learned
		- You installed `lsg` for one folder with mise, published two namespaces of a garden with one workshop and one session left out, served the site on your own computer, and checked its references, its media and what it left out.
		- To publish your own garden, follow [[lsg/How To/Publish Selected Namespaces]], then [[lsg/How To/Publish a Garden to GitHub Pages]]. For another garden, pass `--filename-format triple-lowbar` when its `logseq/config.edn` sets `:file/name-format :triple-lowbar`, and `legacy` otherwise. A leading `title::` property can give a page its title, which `--include` and `--exclude` then match.
		- [[lsg/Ref/Logseq Compatibility]] shows what `lsg` publishes as Logseq does and where it differs. If a page lacks something a reader needs, [open an issue](https://github.com/codekiln/logseq-gardener/issues) with the source page's name, the syntax or reference, what the published page shows, what a reader should see instead, and the namespace selection and filename format you used.
