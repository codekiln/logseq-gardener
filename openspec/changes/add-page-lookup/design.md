## Context

`PageNameIndex` in `page_names.rs` resolves a name to stored pages by lookup key (trimmed, one boundary slash removed, lower-cased, NFC) and falls back to a journal date in the garden's title format or Logseq's reference formats. The publisher fills it with the selected, visible pages and uses it for links, homepage selection and favorites. `page_title` and `journal_identity` supply titles. Nothing yet fills the index with the whole garden or reports source paths.

The Project Brief puts relationship logic in one shared SDK used by the CLI, the language server and the publisher. A name-to-file lookup is the first piece the language server will also need, for go-to-definition on `[[links]]`.

## Goals / Non-Goals

**Goals:**

- Resolve a name to its source file exactly as a published link resolves, so the command and the site never disagree about which page a name means.
- Give scripts and editors a path on stdout and a stable JSON document.

**Non-Goals:**

- A persistent cache so repeated lookups skip parsing. The [agent CLI cache analysis](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Analysis___Fable___26___09___12___0933%20ET%20Agents%20calling%20the%20CLI%20justify%20a%20persistent%20cache%20on%20a%20small%20garden.md) argues for one; it belongs with the incremental index and serves every command, not only lookup.
- Prefix or fuzzy matching for completion in an editor.
- Fileless pages: namespace parents and pages that exist only as link targets.

## Decisions

The SDK module `page_lookup` owns the catalog. `PageCatalog::build` takes loaded documents, a filename format and journal formats, and has no filesystem access; `PageCatalog::load` reads the garden and `logseq/config.edn` and calls it. Each page or journal enters the existing `PageNameIndex<usize>` under its title and aliases, and journals under their date. `find` returns a `PageLocation` per candidate with title, relative path, kind and `MatchedBy` (`Title`, `Alias` or `JournalDate`), in source-path order because ids follow the loader's sorted documents. Reusing the publisher's index keeps one set of matching rules; a second implementation would drift.

The catalog covers every page, with no namespace selection and no `public:: false` check. Those rules decide what a site shows; lookup answers where the author's own file is.

A page whose title cannot be derived, or a journal whose file name is not a date, is left out with a diagnostic and the rest stay findable. The publisher fails on a bad page title because it cannot produce a correct site; a lookup can still answer for every other page.

The CLI is a thin caller: `lsg lookup NAME --graph PATH --filename-format F`. `--filename-format` is required, as for `publish`, so the two commands read file names the same way. Human output is one path per line, `--graph` joined with the relative path, so `nvim "$(lsg lookup …)"` works. Several matches all print, with a count on stderr; the caller can see the ambiguity and choose. No match exits one with empty stdout and an error naming the query.

`NAME` is a positional argument and `help` is a subcommand. clap reads `lsg lookup help` as the subcommand; a page named `help` is found with the options first and `-- help` last. `args_conflicts_with_subcommands` makes a stray option after `help` an error rather than a lookup.

Budgets: the command adds nothing to published pages. On the whole encode garden (8,569 pages plus journals) a release build takes 0.75 s warm and 263 MB peak RSS on an Apple silicon Mac, almost all of it loading and parsing. The command's budget is 1.5 s and 400 MB on that garden. No CI row checks it, because CI has no copy of the encode garden and wall time on shared runners is not stable; the measurement is recorded in `docs/cli.md`.

What lives where: matching, titles, aliases, journal dates and diagnostics are in the SDK; the CLI parses arguments, joins paths and formats output.

## Risks / Trade-offs

- [Parsing the whole garden on each call is slower than a lookup needs to be] → Acceptable for an interactive command at under a second; the persistent cache in the non-goals removes it later without changing the interface.
- [A positional `NAME` shadows a page named `help`] → Documented `--` form, covered by a test.
- [Two pages claiming one alias print two paths, which breaks `nvim "$(…)"`] → The count on stderr says why; the JSON lists both with `matched_by`.

## Resolved Questions

### 1 - What is the command called and how does it take the name?

Decided by the agent without codekiln: `lsg lookup NAME` with `NAME` positional. One verb keeps the help tree flat beside `publish` and `version`, and a positional name is what `nvim "$(lsg lookup nvim …)"` wants. A `page` command group can come later if more page commands appear.

### 2 - Does lookup include pages marked `public:: false` or outside a namespace selection?

Decided by the agent without codekiln: yes, every page. Visibility and selection decide what a published site shows; lookup runs on the author's own files, and hiding a private page from its own author would only send them back to `grep`.

### 3 - Is `--filename-format` read from `logseq/config.edn` or passed?

Decided by the agent without codekiln: passed and required, as for `publish`. The two commands then read file names identically, and the existing help already explains which value to choose. Reading `:file/name-format` from the config can come to both commands together.

### 4 - What happens with no match, and with several?

Decided by the agent without codekiln: no match exits one with empty stdout and an error naming the query; several matches all print in source-path order with a count on stderr and exit zero. Exit one lets `nvim "$(…)"` and shell conditionals fail cleanly; printing every match shows an ambiguity rather than hiding it behind an arbitrary pick, as Logseq's own link resolution would.

### 5 - What does a file with an unreadable title or journal date do to the lookup?

Decided by the agent without codekiln: it is left out and listed on stderr, and the lookup continues. One bad file name should not stop the author finding every other page.
