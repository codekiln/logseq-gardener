tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Look Up a Page File]], [[lsg/How To/Edit a Garden in Neovim]], [[lsg/Ref/Publish Options]]

- # Command Line Reference
	- ## Overview
		- `lsg` has four commands, `lookup`, `lsp`, `publish` and `version`, and embedded help for each of them.
		- Help works offline with closed standard input, without a pager or prompts. `--no-input` makes that default explicit.
	- ## Commands
		- ### help
			- `lsg`, `lsg help`, `lsg --help` and `lsg -h` show the same root help document.
		- ### lookup
			- `lsg lookup NAME --graph PATH --filename-format legacy|triple-lowbar` prints the source file of the page that `NAME` names, so an editor or agent can open it. [[lsg/How To/Look Up a Page File]] shows how to use it, and Lookup matching below gives its rules.
			- `lsg lookup help`, `lsg lookup --help` and `lsg lookup -h` show its documentation.
		- ### lsp
			- `lsg lsp` serves the Language Server Protocol (LSP) for a garden on standard input and output. An editor starts it; [[lsg/How To/Edit a Garden in Neovim]] gives the Neovim configuration. Language server below describes what it reads and answers.
			- `lsg lsp help`, `lsg lsp --help` and `lsg lsp -h` show its documentation without starting a server.
		- ### publish
			- `lsg publish` generates a static site from selected namespaces of a garden. [[lsg/Ref/Publish Options]] lists its options and its JSON report, and [[lsg/How To/Publish Selected Namespaces]] walks through a first site.
			- `lsg publish help`, `lsg publish --help` and `lsg publish -h` show its documentation without garden arguments.
		- ### version
			- `lsg version` identifies the build. `lsg version help`, `lsg version --help` and `lsg version -h` show its documentation.
			- `lsg --version` and `lsg -V` report the same version as `lsg version`.
	- ## Help navigation
		- These forms work after `lsg`, `lsg lookup`, `lsg lsp`, `lsg publish` and `lsg version`:
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
	- ## Language server
		- `lsg lsp` takes no options. The garden root is the nearest folder, at or above the workspace folder or root URI the editor sends, that holds `logseq/config.edn`. When there is none, the server answers `initialize` with an error that names the folder, and exits with status 1 when the editor closes the connection.
		- The server reads `:file/name-format`, the journal formats and `:hidden` from `logseq/config.edn`, then every Markdown file beneath `pages/` and `journals/`, skipping names that start with a dot, `node_modules`, symlinks and `:hidden` entries. It reports the reading as work done progress. Markdown files in other folders are left out.
		- The server asks for incremental text synchronization. The text of an open buffer takes the place of its file until the buffer closes.
		- For changes made outside the editor, the server registers `workspace/didChangeWatchedFiles` for `pages/**/*.md`, `journals/**/*.md` and `logseq/config.edn` when the editor supports dynamic registration, and watches those folders itself otherwise: through the operating system's file events on Linux and Windows, and by comparing modification times every two seconds on macOS. A change to `logseq/config.edn` reads the whole garden again.
		- Positions use `utf-8` when the editor lists it in `general.positionEncodings`, and `utf-16` otherwise.
		- The server advertises no formatting and changes no file.
		- ### Navigation
			- | Request | Answer |
			  | --- | --- |
			  | `textDocument/definition` | The start of the page's file for a page link, tag, page embed or page name in a property value, and the first line of the block for a block reference, block embed or `id::` value. `null` in ordinary text |
			  | `textDocument/references` | Every range that names the page or block under the cursor. In ordinary text, every range that names the page of the document |
			  | `textDocument/hover` | Markdown: a page's title, aliases, reference count and first five lines, or a block's first five lines below its page title and parent blocks. `null` in ordinary text and on a property key |
			- A name resolves as Logseq resolves it: by title, then by alias, then by the journal date it spells in any format Logseq accepts, ignoring letter case. A name that several pages answer to gives one definition location for each page.
			- A page that has no file gets the location `pages/<file name>.md`, where the file name follows Logseq's rules for the garden's `:file/name-format`: `Fruit/Quince` gives `Fruit___Quince` under `:triple-lowbar` and `Fruit%2FQuince` under the legacy format. A journal that has no file gets `journals/<date>.md` with the date in `:journal/file-name-format`. The server creates neither file.
			- References count a page's aliases as the page. With `includeDeclaration`, the list starts with the start of the page's file and each name in its own `alias::` property, or with the `id::` value of the block. A page without a file has no declaration. On a property key, references list every line that writes the key.
		- ### Requests
			- | Request | Parameters | Result |
			  | --- | --- | --- |
			  | `lsg/status` | none | `root`, `filenameFormat` (`triple-lowbar` or `legacy`), `journalTitleFormat`, `journalFileNameFormat`, `positionEncoding`, `fileWatching` (`client`, `server` or `none`), `documents`, `pages`, `journals`, `openDocuments` and `loadMilliseconds` |
			  | `lsg/occurrence` | `textDocument` and `position` | `null` when nothing is at the position, otherwise `kind`, `range`, `nameRange`, `target` and `resolved` |
			- `kind` is `pageLink`, `tag`, `blockReference`, `pageEmbed`, `blockEmbed`, `propertyKey`, `propertyValue` or `blockId`. `range` covers the whole syntax, such as `[[Page]]`, and `nameRange` the name inside it.
			- `target` has a `type` of `page` with the `name` as written, `block` with the `id`, or `property` with the `key`.
			- `resolved` has a `type` of `document` with the page's `uri` and `title`; `journalWithoutFile` with the `date`; `pageWithoutFile` with the `name`; `ambiguous` with the `uris` of the pages that share the name; `block` with the `uri` and `range` of the block's first line; `blockNotFound`; or `property` with the `count` of lines that write the key.
			- A block id that several blocks write resolves to the block Logseq keeps: the one in the file Logseq reads first.
	- ## Streams and exit codes
		- | Code | Meaning |
		  | --- | --- |
		  | 0 | Success, including early termination when a downstream pipe closes |
		  | 1 | Generation failed, a lookup matched no page or could not read the garden, output could not be written, or an `lsg lsp` session ended without `shutdown` |
		  | 2 | Invalid arguments, an unknown command, or an unknown section |
		- Successful results go to stdout and diagnostics go to stderr. `lsg lsp` writes only LSP messages to stdout. Invalid invocations leave stdout empty, including with `--format json`, and an error writes no JSON document.
		- Output contains no terminal control sequences and never depends on a TTY.
		- Help and version never read stdin, garden files, configuration files or network resources.
		- Paths in arguments accept the operating system's native form. Paths in JSON output are display strings.
