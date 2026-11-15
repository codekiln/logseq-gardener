## Context

[PR #650 — follow links, list references and hover in lsg lsp](https://github.com/codekiln/logseq-gardener/pull/650) gave the SDK the title-to-file-name rules in `file_names` and `GardenIndex::page_references`, which says for each place that names a page whether the place writes the title, an alias or a journal date. [Issue #643 — Rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/643) adds rename on top of both.

Logseq's rename is `rename!` in `src/main/frontend/handler/page.cljs` of the local [logseq/og](https://github.com/logseq/og) checkout, read at commit `6e7afa8eb`. It calls `rename-page-aux` for the page and each namespace child (`rename-namespace-pages!`), rewrites the text of every block that refers to the page (`rename-update-refs!`, `replace-old-page!`), and renames pages whose title holds a link to the page (`rename-nested-pages`). `rename-page-aux` renames the file unless the page is a journal, writes a `title::` property when the file name cannot carry the title, and updates `:default-home`.

The server writes no file. A rename answers with a `WorkspaceEdit`, and the editor applies it. Neovim 0.12 applies `documentChanges` in order with `vim.lsp.util.apply_workspace_edit`, and renames a file with `vim.lsp.util.rename`.

## Goals / Non-Goals

**Goals:**

- `:lua vim.lsp.buf.rename()` on a page link, tag or page name in a property value, or in the ordinary text of a page, renames the page across the garden as Logseq would.
- Text outside the replaced names stays as written, including tab indentation.
- The rename is computed in the SDK as a plan without LSP types, so a later `lsg` command can print or apply the same plan.
- The handler's conversion of a plan to a `WorkspaceEdit` serves [Issue #644 — Rename a property key across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/644) unchanged.

**Non-Goals:**

- Merging two pages when the new name belongs to another page, as Logseq's `merge-pages!` does. Issue #638 lists it among the later features.
- Renaming a property key. Issue #644 covers it.
- Renaming the property keys that share the page's name. Logseq's `replace-property-ref!` rewrites `old::` to `new::` in blocks that refer to the page; issue #644 decides how property keys are renamed.
- An `lsg` command that renames a page from a terminal.

## Decisions

### The plan

`GardenIndex::rename_page(&PageId, new_name)` returns `Result<RenamePlan, RenameRefusal>`. A `RenamePlan` holds `pages`, the pages with a file that get a new title, and `documents`, one `DocumentChange` per file to change, in path order. A `DocumentChange` holds the file's current path, the text its ranges count in, `replacements` (a source range and the new text, in source order, none overlapping another) and `moves` (file renames to apply in order after the replacements). `DocumentChange::text` returns the file's text after the replacements, which the unit tests compare and a later command can write.

`rename_page_with_config` takes the text of `logseq/config.edn` from the caller, so the server passes an open buffer's text. `rename_page` reads the file.

### Which names the rename replaces

The rename works on written names. A name is renamed when its lookup key equals the old title's key (`old` gives `new`), when a prefix of it up to a `/` does (`old/x` gives `new/x`, with the rest kept as written), or when it holds `[[old]]` or `[[old/x]]` inside (`[[old]] is a tool` gives `[[new]] is a tool`). The same function renames page titles and the names in references, so a child page, the links to it and a link to a child that has no file all follow.

A written name is replaced only when it names a page by its title or names a page with no file. A name that reaches a page through an alias, or through a journal date in a format other than the journal's title, stays: it still leads to the same page after the rename. The index decides with the lookup that go to definition uses, so letter case and Unicode normalization match the way links resolve.

Per kind of place:

- `[[old]]`, `{{embed [[old]]}}` and `[label]([[old]])`: the name inside the brackets.
- `#[[old]]`: the name inside the brackets.
- `#old`: the whole tag becomes `#new` when `#new` reads back as one tag of that name, and `#[[new]]` otherwise. The check parses `#new` with the outline parser, so a name with a space or a trailing dot gets brackets.
- A bare name in `tags::`, `alias::` or `aliases::`: the name, or `[[new]]` when the new name holds a comma, which would split a bare name in two.

### Files and titles

A page file gets the name `file_names::page_file_name` gives the new title, in the folder the file is in, as Logseq's `compute-new-file-path` keeps the folder. A journal keeps its file name.

A page with a `title::` line in its leading properties gets the new title there. A page without one gets a `title::` line at the top when `file_names::title_needs_property` says the file name reads back as another title, and a journal always gets one, since its file name is its date. The line goes ahead of existing leading properties, or ahead of the page's first line with an empty line between.

### Renames that change only letter case

When the old and the new path are equal after lowercasing and Unicode NFC, the plan renames the file twice: to `.lsg-rename-<n>.tmp` in the same folder, then to the new name. The index skips names that start with a dot, so the temporary file is never a page.

A single `RenameFile` from `Foo.md` to `foo.md` fails in Neovim 0.12.6 on macOS. `vim.lsp.util.rename` finds the target with `uv.fs_stat` and skips the rename; with `options.overwrite` it deletes the buffer it is renaming, because `bufexists` matches the names without regard to case. The two renames need no option and no knowledge of the client's file system. A temporary name that holds the new name, such as `Foo.md.lsg-rename`, also failed in the headless trial: Neovim reported that the new name was already in its buffer list and skipped the second rename.

### Refusals

`RenameRefusal` says why a rename cannot be done, and its `Display` text is the error the person reads. The new name is trimmed, and one slash is dropped from each end, as Logseq does. The rename is refused when:

- the new name is empty or holds a line break;
- a new title, of the page or of a child, belongs to another page by title or alias, or spells a journal date;
- a name that only references carry would become the name of a page with a file;
- several files answer to the old name, or the old name is a journal date with no file;
- the page's title is written in a form the rename cannot rewrite, such as front matter;
- a file already exists at a new path, or the new title gives an empty file name.

A new name equal to the page's title gives an empty plan.

### What a rename at a position is about

`GardenIndex::rename_target_at(path, offset)` returns a `RenameTarget { subject, range, name }`. On a page link, tag, page embed or page name in a property value the subject is the page the name leads to, following an alias to its page; the range is the written name and `name` is the page's title, or the name as written for a page with no file. Anywhere else the subject is the document's own page, with the range of its `title::` value when the cursor is in it and an empty range at the cursor otherwise. A block reference and a property key are refused. `subject` is the `Subject` type of find references, so issue #644 adds property keys by answering for `Subject::Property`.

### The workspace edit

`features::rename::workspace_edit(server, &plan)` turns any plan into a `WorkspaceEdit`. For each file it writes one `TextDocumentEdit` and then the file's `RenameFile` operations. Neovim applies the text edit to the buffer of the old name and then renames the file, and `vim.lsp.util.rename` writes the edited buffer under the new name. The other order, rename and then edit under the new name, leaves the renamed files with unsaved edits and gives the server no known version for the new name.

A `TextDocumentEdit` for an open buffer carries the version the client last sent, which the server now keeps per URI, and the URI the client opened the buffer under. Other edits carry a null version.

The handler uses `documentChanges` when the client advertises `workspace.workspaceEdit.documentChanges` or lists any resource operation, and `changes` otherwise. Neovim lists `resourceOperations` and leaves `documentChanges` out, and reads `documentChanges`. A plan that renames a file needs `rename` among the resource operations; without it the answer is an error that names the capability.

A refusal is an error response with code `-32803` (`RequestFailed`) and the text `cannot rename: <reason>`. Neovim shows it as a notification.

### Neovim check

`mise run lsp:nvim-check` renames the page of `pages/Fruit.md` to `Orchard` with `vim.lsp.buf.rename`, waits for the three renamed files, writes all buffers and reads the links in other files from disk. It then renames `Orchard` to `orchard`, which covers the rename through a temporary name on a file system that ignores letter case.

## Risks / Trade-offs

- [Neovim leaves the edits to files that keep their name as unsaved buffers] → The how-to page says to run `:wall`. Renamed files are written at once, because `vim.lsp.util.rename` saves them.
- [The index holds a file's text from disk while another program is changing it] → The file watcher refreshes the index, and an edit computed for an old text of an open buffer carries its version, so the client drops it.
- [A rename of a page with thousands of namespace children returns a large edit] → On the encode garden, renaming `Microfreak` returns 20,209 text edits and 2,797 file renames in 4.2 MB, in about 270 ms. The PR records the numbers.
- [A crash of the editor between the two renames of a letter-case rename leaves `.lsg-rename-<n>.tmp`] → The file holds the page's text, and renaming it by hand finishes the rename.

## Resolved Questions

### 1 - Does the rename follow Logseq where Logseq matches text, or the index where the index resolves names?

The index. Logseq's `replace-page-ref!` replaces the string `[[old]]` with the page's stored spelling, so `[[OLD]]` stays when the page is `Old`, and its tag pattern misses a tag followed by `)`. The issue asks for names matched the way links resolve, and the index already resolves each written name with source ranges. Every place that leads to the page by its title is rewritten, whatever its letter case.

### 2 - Does a rename that changes only letter case rename the namespace children?

Yes. Logseq calls `rename-page-aux` for the page alone in that case, which leaves `Foo/Bar` beside the renamed `foo`. A child's title starts with its parent's title in Logseq's namespace display, so the plan renames children on every rename.

### 3 - What does renaming a journal do?

The journal keeps its file name, as the issue asks and as Logseq does, and gets the new title in a `title::` property, which Logseq's `get-page-name` and the SDK's `journal_identity` read ahead of the file name. Links written with the old title are rewritten. Links written as the date in another format stay, and still lead to the journal by its date.

### 4 - Is a name that only references carry free to take?

Yes. Renaming `Fruit` to `Ghost` when `[[Ghost]]` is written and has no file is allowed: the existing links then lead to the renamed page, which is what a person who links first and creates the page later expects. The other direction, renaming a name with no file onto a page with a file, is refused as a merge.

### 5 - Does every page rename need the `rename` resource operation?

Only a rename that gives a file a new name. The issue says to require the capability; a tag with no file is renamed with text edits alone, and issue #644 renames property keys the same way, so the handler checks the capability when the plan holds a file rename and returns the error then.

### 6 - What does prepare rename return in ordinary text?

An empty range at the cursor and the page title as `placeholder`. Neovim shows the placeholder as the default of its prompt and uses nothing else from the answer. A range around the word under the cursor would suggest that the word is what gets renamed.

### 7 - Does the rename rewrite property keys that share the page's name?

No. Logseq's `replace-property-ref!` renames `old::` to a lower-case, dash-joined form of the new name in blocks that refer to the page. Property keys get their own rename in issue #644, with the cursor on the key.

### 8 - Where does the `title::` line go when a page has none?

At the top of the file: ahead of the leading properties when there are some, and ahead of the first line with an empty line between otherwise. Logseq's `add-property!` writes page properties in the page's first block, which is the top of the file.

### 9 - Does the fixture garden of the other tests change?

No. The rename tests use `tests/fixtures/lsp-rename-garden/` with a `before/` and an `after/` garden, and the Neovim check renames in its temporary copy of `tests/fixtures/lsp-garden` after the other checks ran.

## Open Questions

None.
