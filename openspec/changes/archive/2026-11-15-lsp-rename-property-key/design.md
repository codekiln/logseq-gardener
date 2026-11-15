## Context

[PR #655 — rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/pull/655) added the rename plumbing: `GardenIndex::rename_target_at` says what a rename at an offset is about, a `RenamePlan` lists text replacements and file renames without Language Server Protocol (LSP) types, and `features::rename::workspace_edit` turns any plan into a `WorkspaceEdit`. A property key is the one subject `rename_target_at` still refuses. The index already records each `key:: value` line as a `PropertyLine` with the key in lower case, the range of the key and the start of the block the line belongs to.

Logseq has no command that renames a property key, so this rename follows how Logseq reads keys. `extract-properties` in `deps/graph-parser/src/logseq/graph_parser/block.cljs` of the local [logseq/og](https://github.com/logseq/og) checkout, read at commit `6e7afa8eb`, lower-cases a key and replaces each space and `_` with `-` before it stores the key. `deps/graph-parser/src/logseq/graph_parser/property.cljs` lists the keys Logseq uses for its own features in `editable-built-in-properties` and `hidden-built-in-properties`, and `src/main/frontend/extensions/srs.cljs` registers the card scheduling keys as built-in.

[Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) asks the change that finishes the MVP to record the server's speed on the encode garden in the docs graph. This change is that one.

## Goals / Non-Goals

**Goals:**

- `:lua vim.lsp.buf.rename()` on a property key renames the key on every `key::` line of the garden.
- A rename never changes what Logseq reads from a block apart from the key's name: values stay, and no value becomes hidden behind another line of the same key.
- Keys that Logseq gives a meaning keep their names.
- The epic's performance numbers come from one release build, a script in the repository produces them, and a docs page records them.

**Non-Goals:**

- Rewriting the key where a query or a setting names it: `(property type book)` in a `{{query}}`, `query-properties:: [:type]`, `:block-hidden-properties` and `:property-pages/excludelist` in `logseq/config.edn`.
- Renaming keys written in YAML front matter (`key: value` between `---` lines) and in Org-mode drawers.
- Merging the two values when a block writes both the old and the new key.
- An `lsg` command that renames a property key from a terminal.

## Decisions

### The plan

`GardenIndex::rename_property(key, new_key)` returns `Result<RenamePlan, RenameRefusal>`. The plan holds one `DocumentChange` per file that writes the key, in path order, each with one replacement per `key::` line: the range of the key and the new key. It holds no file rename and no entry in `pages`. The handler passes the plan to `workspace_edit` as it is, so the edit of an open buffer carries its version and a client without the `rename` resource operation gets a working edit.

### Which lines the rename replaces

A line is replaced when its key equals the old key after Logseq's normalization: lower case, with `_` as `-`. `property_names::logseq_property_key` does the normalization. The index keeps keys in lower case only, so `rename_property` compares the normalized form of each line's key itself.

The new key is written as typed, after trimming spaces and one `::` from its end, which a person who reads `type::` on the line may type. A line that already spells the new key gets no replacement, so a rename that changes only letter case, or only `_` to `-`, touches the lines that differ.

Lines of a properties block that the index does not record as properties stay: a line whose key Logseq's parser drops, and front matter.

### Keys with a built-in meaning

`property_names::is_built_in_property(key)` is true for a key that, normalized, is in Logseq's `editable-built-in-properties` or `hidden-built-in-properties`, is a card scheduling key, or starts with `logseq.`. The list covers `title`, `alias`, `aliases`, `tags`, `icon`, `template`, `template-including-parent`, `public`, `filters`, `exclude-from-graph-view`, `macro`, `filetags`, `id`, `custom-id`, `collapsed`, `heading`, `background-color`, the `created-at`, `updated-at` and `last-modified-at` timestamps, the `query-*` keys, `ls-type`, the `hl-*` keys, the task markers `todo`, `doing`, `now`, `later` and `done`, and the six `card-*` keys.

Renaming from such a key would remove a feature from every block that uses it, and renaming to one would give the values a meaning their author did not write. Both are refused. Prepare rename refuses too, so Neovim shows the reason before it asks for a name.

Keys in the garden's `:block-hidden-properties` setting are the person's own and can be renamed.

### The new key is already in use

Other blocks may already write the new key. The rename goes ahead and the two keys become one, which is what a person wants who settles on `kind` after writing both `type` and `kind`. Each block keeps its value.

A block that writes both the old and the new key would end with two lines of one key. Logseq keeps the last line's value and the other value disappears from queries and the properties box. The rename is refused with the file and line of the first such block, and the person decides which value stays.

A block that writes the old key more than once has that state before the rename and keeps it after.

### Refusals

`RenameRefusal` gains:

- `BuiltInKey { key }`: the old or the new key has a built-in meaning;
- `InvalidKey { key }`: the new key is empty, holds white space or a colon, or `property_names::valid_property_name` rejects it;
- `KeyOnSameBlock { key, new_key, path, line }`: a block writes both keys.

`RenameRefusal::PropertyKey` goes away. A new key that normalizes and spells as every line already does gives an empty plan, as a page rename to the same title does.

### What a rename at a position is about

On a property key `rename_target_at` returns `Subject::Property` with the key in lower case, the range of the key and the key as written on that line, which prepare rename returns as the placeholder. It returns `BuiltInKey` for a built-in key.

### Pages and property keys stay apart

Logseq treats a property key as a reference to a page of the same name, and its page rename rewrites `old::` lines in blocks that refer to the page. `lsg lsp` keeps the two renames separate, as [PR #655](https://github.com/codekiln/logseq-gardener/pull/655) left them: a page rename leaves keys as written and a key rename leaves the page, its file and its links as written. A person who wants both runs both renames. Find references on a key lists `key::` lines only, and rename changes the same places.

### Measuring

`scripts/lsp/perf.py` starts `lsg lsp` on a garden and speaks JSON-RPC over the server's standard input and output, with the Python standard library alone. It reads the garden's files to pick samples, sends an unsaved buffer for the typing cases, and asks for renames without applying the answer, so the garden's files stay as they are. `mise run lsp:perf <garden>` builds the release binary and runs the script; `--record` writes the docs page `lsg/Ref/Language Server/Performance`. The task is separate from `mise run ci`, because CI has no encode garden.

The script measures:

- loading: the `loadMilliseconds` of `lsg/status`, and the time from starting the process to the answer to `initialize`;
- completion: typed text for each trigger in an unsaved buffer, repeated;
- definition, hover and references: at page links, tags and block references sampled from the garden's files with a fixed seed;
- reindexing: the time from sending `textDocument/didChange` to the answer to an `lsg/status` request sent right after, for a small page and for the garden's largest page. The server handles messages in order, so the answer arrives after the reindex;
- rename: `textDocument/rename` for the most-linked pages and the most-written property key, with the count of text edits and file renames in the answer.

It prints the median and the 95th percentile of each group and compares them with the epic's targets: completion and definition within 100 ms at the 95th percentile, reindexing under 50 ms, loading under 5 seconds.

## Risks / Trade-offs

- [A query or `:block-hidden-properties` still names the old key after a rename] → The how-to page and the reference say that the rename covers `key::` lines, and `lsg/Ref/Command Line` lists the other places to change by hand.
- [Two keys that become one cannot be told apart again] → The edit is an ordinary set of buffer changes in the editor, and the garden is a git repository; undo or `git checkout` restores the lines.
- [The measurements depend on the machine and the garden's state] → The page records the machine, the garden commit, the `lsg` version and the date, and the task reproduces them.

## Resolved Questions

### 1 - Does a rename of `due_date` also rewrite `due-date::` lines?

Yes. Logseq stores both as `due-date`, so they are one property in queries and in the properties box. A rename that left one spelling behind would split one property in two.

### 2 - What happens when other blocks already write the new key?

The rename is accepted and the keys become one. No value is lost, and joining two spellings of one idea is a common reason to rename a key.

### 3 - What happens when one block writes both the old and the new key?

The rename is refused, with the file and line of the first such block. Logseq reads one value per key on a block, so the rename would hide a value. Dropping a line or joining the values would change values, which the issue says to leave alone.

### 4 - Which keys count as having a built-in meaning?

Logseq's `editable-built-in-properties` and `hidden-built-in-properties`, the card scheduling keys that its flashcards register, and every key that starts with `logseq.`, compared after normalization. The `logseq.` prefix is Logseq's own namespace for properties such as `logseq.order-list-type`, and a garden may hold keys of that namespace from a newer Logseq than the list was read from.

### 5 - Does the page rename start to rewrite property keys that share the page's name?

No. It stays as [PR #655](https://github.com/codekiln/logseq-gardener/pull/655) left it. Logseq's `replace-property-ref!` writes a lower-case, dash-joined form of the new page name as the key, which gives `sweet-produce::` for a page renamed to `Sweet Produce` and a key Logseq drops for a name with a bracket. With a rename for keys available, a person renames the key on purpose and chooses its spelling.

### 6 - Does prepare rename refuse a built-in key, or only rename?

Both. Neovim calls prepare rename first and shows the error in place of the prompt, so the person learns the reason before typing a name.

### 7 - What does the new key look like after trimming?

Spaces around it are dropped, and so is one `::` at its end. The key is otherwise written as typed, in the letter case typed.

### 8 - How is reindexing after an edit measured from end to end?

By the time from sending `textDocument/didChange` to receiving the answer to a request sent right after it. The server reads messages in order on one thread, so the answer cannot come before the index holds the changed text. The request is `lsg/status`, whose own work is small.

### 9 - Where does the fixture for the end-to-end test live?

In `tests/fixtures/lsp-rename-property-garden/` with a `before/` and an `after/` garden, beside the page rename's fixture and apart from `tests/fixtures/lsp-garden`, whose document count other tests assert.

## Open Questions

None.
