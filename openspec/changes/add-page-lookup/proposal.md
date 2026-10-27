## Why

People and agents working in a garden know a page by its name, not by its file: `[[nvim]]` lives in `pages/nvim.md`, `[[Tools/Neovim]]` in `pages/Tools___Neovim.md`, and an alias or a journal date can point at either. Today the only way to find the file is to guess the encoding or `grep`. [Issue #4 — Add read-only page lookup through the SDK](https://github.com/codekiln/logseq-gardener/issues/4) asks for one command that resolves a name the way Logseq resolves a link and prints the source path, for the terminal, Neovim and agents.

The SDK already holds the parts: page titles, aliases, journal identities and the `PageNameIndex` that the publisher uses to resolve links. Only a whole-garden catalog and a command are missing.

## What Changes

- The SDK gains a page catalog: every page and journal in a garden, findable by title, leading `alias::` name or journal date, with the source path and how the name matched. It uses the same name index and matching rules as the publisher's link resolution.
- `lsg lookup NAME --graph PATH --filename-format legacy|triple-lowbar` prints the path of each page `NAME` reaches, one per line, or a JSON document with title, paths, kind and match type. Several matches all print; no match exits one with empty stdout.
- The CLI's exit-code requirement names a lookup with no match as a code-one outcome.
- Root help, `docs/cli.md`, `docs/architecture.md` and the README describe the command.

## Capabilities

### New Capabilities

- `page-lookup`: the SDK page catalog and the `lsg lookup` command, its matching rules, output and exits.

### Modified Capabilities

- `cli-contract`: exit one also covers a lookup that matches no page.

## Impact

- SDK: a new `page_lookup` module beside `page_names`; no change to existing modules.
- CLI: a `lookup` subcommand in `src/cli.rs`, its help in `src/help.rs`, and a `diagnostics` field on `Failure` so a failed lookup can list skipped files before the error.
- Help assessment: root help lists a new child command, so `tests/assessments/help.json` is reassessed.
- The publisher, the published site and the parity budgets are unchanged.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
