# Command-line interface

Run `lsg help` to read embedded documentation. Help works offline with closed standard input and without a pager or prompts. `--no-input` makes that default explicit. `lsg`, `lsg help`, `lsg --help`, and `lsg -h` show the same root document.

Use `lsg publish` to generate selected local HTML pages; see [lsg/How To/Publish Selected Namespaces](pages/lsg___How%20To___Publish%20Selected%20Namespaces.md) for a copyable trial and [lsg/Ref/Rendering and Assets](pages/lsg___Ref___Rendering%20and%20Assets.md) for rendering limitations. `lsg publish help`, `lsg publish --help`, and `lsg publish -h` show equivalent documentation without requiring garden arguments.

Use `lsg lookup NAME --graph PATH --filename-format legacy|triple-lowbar` to print the source file of the page that `NAME` names, so an editor or agent can open it; see [Find a page's source file](#find-a-pages-source-file). `lsg lookup help`, `lsg lookup --help`, and `lsg lookup -h` show its documentation.

Use `lsg version` to identify the build. Run `lsg version help`, `lsg version --help`, or `lsg version -h` to read its documentation. Root `--version` and `-V` report the same version as `lsg version`.

## Navigate documentation

Use these forms with `lsg`, `lsg lookup`, `lsg publish`, or `lsg version`:

- `help --format json` to discover immediate `child_commands`.
- `help outline` to list ordered heading identifiers.
- `help outline --level 2` to select one heading level, or `--max-level 2` to include shallower headings.
- `help section behavior` to read the section's direct content.
- `help section behavior --recursive` to include descendants.
- `--programmatic` to include guidance for scripts and agents.

Level values range from 1 through 6. `--level` and `--max-level` are mutually exclusive. An unknown section or option is an error. Help navigation follows the [codekiln-help standard](https://github.com/codekiln/clilint/blob/200e92a5d76420e8c47fd9e623da704b355dbc7d/docs/codekiln-help.md).

## JSON format 1

Every successful JSON response includes `format_version: 1` and `command_path`, an array relative to `lsg`. Help also includes `programmatic` and `child_commands`. Each child has a `name`; the list contains product commands such as version. Help is available at each command path.

Outlines contain ordered `headings` with `level`, `title`, and `section`. Section responses contain `sections` with those fields plus `content`. Full help includes all sections. Recursive section retrieval follows document order and includes descendants until the next heading at the same or shallower level. Stable section identifiers can be copied directly from outlines.

`lsg version --format json` returns `program: "lsg"`, the Cargo package `version`, `sdk_version` for the linked SDK, and `command_path: ["version"]`. Additive fields can appear within format 1; consumers should ignore unknown fields. Removing a field or changing its meaning requires a new format version.

## Streams and exit codes

| Code | Meaning |
| --- | --- |
| 0 | Success, including early termination when a downstream pipe closes |
| 1 | Generation failed, a lookup matched no page, or output could not be written |
| 2 | Invalid arguments, unknown command, or unknown section |

Successful results go to stdout; diagnostics go to stderr. Invalid invocations leave stdout empty, including with `--format json`. There is no JSON success document for an error. Output contains no terminal control sequences and never depends on a TTY. Help and version never read stdin, garden files, configuration files, or network resources at runtime.

## Publishing reports

`lsg publish --graph PATH --output PATH --filename-format legacy|triple-lowbar` accepts repeated `--include NAMESPACE` and `--exclude NAMESPACE`. An empty include list selects nothing. Paths accept native OS arguments; JSON paths are display strings.

Successful JSON has `command_path: ["publish"]`, `pages`, `journals`, `assets`, `withheld_pages`, `skipped_journals`, `diagnostic_count`, `output_directory`, and `index_path`. Selected unsupported content produces a completed site and exit zero with local diagnostics on stderr. Fatal SDK errors leave stdout empty and exit one. Invalid selectors and arguments exit two.

The optional `publish --stylesheet PATH` selects a local CSS file loaded after built-in styles on every generated page. The file must be readable, regular, have a `.css` extension, and use a path without symlink components. Invalid stylesheet input returns one before output creation. See [lsg/How To/Customize a Published Site](pages/lsg___How%20To___Customize%20a%20Published%20Site.md) for copying and browser resource behavior.

The optional `publish --journals` publishes journal pages and Logseq's journals feed; see [lsg/How To/Customize a Published Site](pages/lsg___How%20To___Customize%20a%20Published%20Site.md). The optional `publish --search` emits a versioned external manifest, corpus and pure evaluator. Generated pages advertise the manifest without loading the corpus. See [lsg/Ref/Static Search Data](pages/lsg___Ref___Static%20Search%20Data.md) for supported content, exclusions and the consumer API.

## Find a page's source file

`lsg lookup NAME --graph PATH --filename-format legacy|triple-lowbar` prints the path of each file whose page `NAME` names, one per line, joined to `--graph` as you spelled it:

```sh
nvim "$(lsg lookup nvim --graph ~/garden --filename-format triple-lowbar)"
```

`NAME` matches a page title, a leading `alias::` name, or a journal date written in the garden's journal title format, `yyyy-MM-dd`, `yyyy_MM_dd` or `MMM do, yyyy`. Matching ignores case and surrounding spaces, the way Logseq resolves `[[links]]`. Every page counts, including pages marked `public:: false`, because the command reads your own files rather than a published site.

When several pages share the name, every path prints in source-path order and stderr says how many matched. No match exits one with empty stdout. Files whose title or journal date cannot be read are left out and listed on stderr. To look up a page named `help` or a name starting with a dash, put the options first and `--` just before the name.

Successful JSON has `command_path: ["lookup"]`, `query`, `diagnostic_count`, and `pages`, each with `title`, `path`, `relative_path`, `kind` (`page` or `journal`) and `matched_by` (`title`, `alias` or `journal-date`).

Each call reads and parses the whole garden. On the 8,569-page encode garden that takes about 0.8 s and 263 MB of memory on an Apple silicon Mac.
