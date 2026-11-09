tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Look Up a Page File]], [[lsg/Ref/Publish Options]]

- # Command Line Reference
	- ## Overview
		- `lsg` has three commands, `lookup`, `publish` and `version`, and embedded help for each of them.
		- Help works offline with closed standard input, without a pager or prompts. `--no-input` makes that default explicit.
	- ## Commands
		- ### help
			- `lsg`, `lsg help`, `lsg --help` and `lsg -h` show the same root help document.
		- ### lookup
			- `lsg lookup NAME --graph PATH --filename-format legacy|triple-lowbar` prints the source file of the page that `NAME` names, so an editor or agent can open it. [[lsg/How To/Look Up a Page File]] shows how to use it, and Lookup matching below gives its rules.
			- `lsg lookup help`, `lsg lookup --help` and `lsg lookup -h` show its documentation.
		- ### publish
			- `lsg publish` generates a static site from selected namespaces of a garden. [[lsg/Ref/Publish Options]] lists its options and its JSON report, and [[lsg/How To/Publish Selected Namespaces]] walks through a first site.
			- `lsg publish help`, `lsg publish --help` and `lsg publish -h` show its documentation without garden arguments.
		- ### version
			- `lsg version` identifies the build. `lsg version help`, `lsg version --help` and `lsg version -h` show its documentation.
			- `lsg --version` and `lsg -V` report the same version as `lsg version`.
	- ## Help navigation
		- These forms work after `lsg`, `lsg lookup`, `lsg publish` and `lsg version`:
			- | Form | Shows |
			  | --- | --- |
			  | `help --format json` | the help document as JSON, with the immediate `child_commands` |
			  | `help outline` | the ordered heading identifiers |
			  | `help outline --level 2` | the headings of one level |
			  | `help outline --max-level 2` | the headings of that level and every shallower level |
			  | `help section behavior` | the direct content of one section |
			  | `help section behavior --recursive` | one section with its descendants |
			  | `--programmatic` | the document with guidance for scripts and agents added |
		- Levels range from 1 through 6, and `--level` and `--max-level` are mutually exclusive. An unknown section or option is an error.
		- Publishing help names the pages of this site that hold more detail, each by its title and URL, such as `lsg/Ref/Static Search Data (https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/)`. Help prints these URLs and never fetches them. Offline, find the same title in the `docs` graph of a release archive, which [[lsg/Ref/Release Archives]] describes.
		- Help navigation follows the [codekiln-help standard](https://github.com/codekiln/clilint/blob/200e92a5d76420e8c47fd9e623da704b355dbc7d/docs/codekiln-help.md).
	- ## JSON format 1
		- `--format json` selects JSON output. Every successful JSON response includes `format_version: 1` and `command_path`, an array relative to `lsg`.
		- ### Help
			- Help also includes `programmatic` and `child_commands`. Each child has a `name`, and the list holds product commands such as `version`. Help is available at each command path.
			- Outlines contain ordered `headings`, each with `level`, `title` and `section`. Section responses contain `sections` with those fields plus `content`. Full help includes all sections.
			- Recursive section retrieval follows document order and includes descendants until the next heading at the same or a shallower level. Section identifiers are stable and can be copied directly from outlines.
		- ### Version
			- `lsg version --format json` returns `program: "lsg"`, the Cargo package `version`, `sdk_version` for the linked SDK, and `command_path: ["version"]`.
		- ### Lookup
			- `lsg lookup --format json` returns `command_path: ["lookup"]`, `query`, `diagnostic_count` and `pages`. Each page has `title`, `path`, `relative_path`, `kind` (`page` or `journal`) and `matched_by` (`title`, `alias` or `journal-date`).
		- ### Publish
			- [[lsg/Ref/Publish Options]] lists the fields of `lsg publish --format json`.
		- ### Compatibility
			- Fields can be added within format 1, so a consumer should ignore fields it does not know. Removing a field or changing its meaning requires a new format version.
	- ## Lookup matching
		- `NAME` matches a page title, a leading `alias::` name, or a journal date written in the garden's journal title format, `yyyy-MM-dd`, `yyyy_MM_dd` or `MMM do, yyyy`.
		- Matching ignores letter case and surrounding spaces, the way Logseq resolves a page reference.
		- Every page counts, including pages marked `public:: false`, because the command reads your own files rather than a published site.
		- Each printed path is joined to `--graph` as you spelled it, one path per line. When several pages share the name, every path prints in source-path order and stderr says how many matched.
		- No match exits 1 with empty stdout. Files whose title or journal date cannot be read are left out and listed on stderr.
		- Each call reads and parses the whole garden. On the 8,569-page encode garden that takes about 0.8 s and 263 MB of memory on an Apple silicon Mac.
	- ## Streams and exit codes
		- | Code | Meaning |
		  | --- | --- |
		  | 0 | Success, including early termination when a downstream pipe closes |
		  | 1 | Generation failed, a lookup matched no page or could not read the garden, or output could not be written |
		  | 2 | Invalid arguments, an unknown command, or an unknown section |
		- Successful results go to stdout and diagnostics go to stderr. Invalid invocations leave stdout empty, including with `--format json`, and an error writes no JSON document.
		- Output contains no terminal control sequences and never depends on a TTY.
		- Help and version never read stdin, garden files, configuration files or network resources.
		- Paths in arguments accept the operating system's native form. Paths in JSON output are display strings.
