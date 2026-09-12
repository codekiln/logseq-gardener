# Command-line interface

Run `lsg help` to read embedded documentation. Help works offline with closed standard input and without a pager or prompts. `--no-input` makes that default explicit. `lsg`, `lsg help`, `lsg --help`, and `lsg -h` show the same root document.

The current product command is `lsg version`. Its `help`, `--help`, and `-h` forms show equivalent command documentation. Root `--version` and `-V` report the same version as `lsg version`.

## Navigate documentation

At the root or after `version`, append:

- `help --format json` to discover immediate `child_commands`.
- `help outline` to list ordered heading identifiers.
- `help outline --level 2` to select one heading level, or `--max-level 2` to include shallower headings.
- `help section behavior` to read the section's direct content.
- `help section behavior --recursive` to include descendants.
- `--programmatic` to retain the documentation and add guidance for scripts and agents.

Level values range from 1 through 6. `--level` and `--max-level` are mutually exclusive. An unknown section or option is an error. Help navigation follows the [codekiln-help standard](https://github.com/codekiln/clilint/blob/200e92a5d76420e8c47fd9e623da704b355dbc7d/docs/codekiln-help.md).

## JSON format 1

Every successful JSON response includes `format_version: 1` and `command_path`, an array relative to `lsg`. Help also includes `programmatic` and `child_commands`. Each child has a `name`; help operations are navigation syntax and do not appear as recursively discoverable children.

Outlines contain ordered `headings` with `level`, `title`, and `section`. Section responses contain `sections` with those fields plus `content`. Full help includes all sections. Recursive section retrieval follows document order and includes descendants until the next heading at the same or shallower level. Stable section identifiers can be copied directly from outlines.

`lsg version --format json` returns `program: "lsg"`, the Cargo package `version`, and `command_path: ["version"]`. Additive fields can appear within format 1; consumers should ignore unknown fields. Removing a field or changing its meaning requires a new format version.

## Streams and exit codes

| Code | Meaning |
| --- | --- |
| 0 | Success, including early termination when a downstream pipe closes |
| 1 | Output could not be written |
| 2 | Invalid arguments, unknown command, or unknown section |

Successful results go to stdout; diagnostics go to stderr. Invalid invocations leave stdout empty, including with `--format json`. There is no JSON success document for an error. Output contains no terminal control sequences and never depends on a TTY. Help and version never read stdin, garden files, configuration files, or network resources at runtime.
