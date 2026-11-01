tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Command Line]], [[lsg/Ref/Page Titles]]

- # How To Look Up a Page File
	- ## Overview
		- `lsg lookup` prints the source file of a page that you name by its title, one of its aliases, or a journal date, so an editor or a coding agent can open the file.
	- ## Prerequisites
		- `lsg`, installed as [[lsg/How To/Install with mise]] shows.
		- A Logseq Markdown garden, and its filename format: `triple-lowbar` when its `logseq/config.edn` sets `:file/name-format :triple-lowbar`, otherwise `legacy`.
	- ## Steps
		- ### 1. Print a page's file
			- Pass the name, the garden folder and the filename format:
				- ```sh
				  lsg lookup nvim --graph ~/garden --filename-format triple-lowbar
				  ```
			- `lsg` prints the path of the page's file joined to `--graph` as you spelled it, such as `/Users/you/garden/pages/nvim.md`.
			- The name can be a title, a leading `alias::` name, or a journal date in the garden's journal title format, such as `2025-02-22`. [[lsg/Ref/Command Line]] gives the matching rules.
		- ### 2. Open the file in an editor
			- Pass the printed path to your editor:
				- ```sh
				  nvim "$(lsg lookup nvim --graph ~/garden --filename-format triple-lowbar)"
				  ```
		- ### 3. Handle several matches
			- When several pages share the name, `lsg` prints every path, one per line in source-path order, and stderr says how many matched. Choose the file you want from the list.
		- ### 4. Look up a name that looks like an option
			- To look up a page named `help`, or a name that starts with `-`, put the options first and `--` just before the name:
				- ```sh
				  lsg lookup --graph ~/garden --filename-format triple-lowbar -- help
				  ```
		- ### 5. Read the result from a script
			- Add `--format json` for a JSON result with each matching page's `title`, `path`, `relative_path`, `kind` and `matched_by`. [[lsg/Ref/Command Line]] describes the fields.
	- ## Troubleshooting
		- If no page matches, `lsg` exits with status 1, prints nothing on stdout and says so on stderr. Check the spelling and `--filename-format`.
		- If stderr lists files that were left out, `lsg` could not read their title or journal date. Those files never match.
