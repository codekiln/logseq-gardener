## Context

`load_garden` walks `pages/` and `journals/` only, so `lsg` never needed `:hidden`: the garden's `:hidden` entries name files at the root, such as `README.md` and `CLAUDE.md`. Logseq builds its file list from the whole garden folder with `get-files`, drops paths `ignored-path?` rejects, then drops `:hidden` matches with `remove-hidden-files` ([config.cljs](https://github.com/logseq/logseq/blob/6e7afa8eb/deps/common/src/logseq/common/config.cljs)). `hidden?` prefixes both the path and the pattern with `/` and tests whether the path starts with the pattern, so `README.md` hides only the root file and `../README.md` hides nothing.

Logseq's `gh-pages` build runs `publish-spa` on a fresh checkout, so it never sees ignored files. A local `lsg` build runs in a working folder that holds them: the encode garden's ignored `tools/` folder has 10 Markdown files from other repositories.

## Goals / Non-Goals

**Goals:**

- The whole-garden comparison shows no page that Logseq publishes from the garden's files and `lsg` does not.
- A local build and a build from a fresh checkout publish the same pages.

**Non-Goals:**

- Load `.org` files, which Logseq also reads. The encode garden has none, and `lsg` parses Markdown only.

## Decisions

- **One walk from the root.** `load_garden` walks the root once. A path under `journals/` loads as a journal; every other path loads as a page. The existing rule that the root holds `pages/` or `journals/` stays, so pointing `lsg` at an ordinary folder still fails early. The alternative, keeping the two note-folder walks and adding a third walk for the rest, reads the same files with more code.
- **Logseq's skip rules, then `:hidden`.** Skip any path segment starting with `.`, any `node_modules/` segment, and `logseq/bak/`, `logseq/.recycle/` and `logseq/version-files/`; then drop paths that `:hidden` hides with Logseq's prefix rule. `:hidden` applies in `pages/` and `journals/` too, because Logseq applies it there.
- **Git-ignored files outside the note folders.** When the root is in a git work tree, one `git ls-files --cached --others --exclude-standard` call lists the files a checkout would hold, and the walk keeps only those outside `pages/` and `journals/`. The note folders keep today's behaviour, so an untracked draft page still publishes as before. Outside a git work tree no filter applies. `git_dates.rs` already runs `git` this way, so this adds no dependency.
- **Symlinks and special files outside the note folders are skipped, not errors.** Inside `pages/` and `journals/` they still stop the build, as the spec requires. A tool folder elsewhere with a symlink should not break publishing.
- **Duplicate titles.** When a file outside the note folders derives the same title as a page in `pages/` or `journals/`, the note-folder page wins and the build prints the skipped file's path. Logseq would report a file conflict; `lsg` keeps the page the author maintains and says what it dropped.
- **Title collisions are resolved after loading.** `load_garden` does not derive titles, because titles depend on the filename format the caller passes. `Garden::remove_shadowed_documents` takes each document's name from the caller and drops the files a note-folder document shadows; `publish_site` and `PageCatalog::load` call it and report each dropped path as a diagnostic.
- **Page titles outside `pages/`.** `page_title` accepts any normal Markdown path outside `journals/`. The `Contents` special case stays limited to `pages/contents.*`.
- **SDK owns it.** File discovery, `:hidden` and the git filter live in the SDK loader, where the CLI, the LSP and the publisher share them. The publisher renders the new pages like any other.

## Budgets

- First-paint bytes, script bytes, per-keystroke search latency and peak phone memory per page: unchanged, since no page's markup or the search format changes. The existing `parity:budgets` rows cover them in CI.
- Build time: a new `[loading]` row in `scripts/parity/budgets.toml` with `build_slowdown_pct = 5`; `parity:perf --baseline-lsg` marks a whole-garden build more than 5% slower than the baseline binary's.
- A loader test in CI builds a fixture garden with a page in another folder, a `:hidden` file, a `node_modules/` file, a `logseq/bak/` file and a git-ignored file, and checks the exact page list.

## Risks / Trade-offs

- A garden with Markdown notes for other purposes (READMEs, tool docs) at the root gains pages it did not publish before. → Logseq publishes them too; `:hidden` and `.gitignore` remove them.
- A garden that is not a git repository publishes ignored-looking folders such as `tools/`. → Logseq does the same; `:hidden` covers it.

## Resolved Questions

### 1 - Should `lsg` publish Markdown files outside `pages/` and `journals/`?

Decided by the agent without codekiln: yes, because Logseq does. Issue #252 first guessed that links ending in `.md` create pages; Logseq's `gh-pages` database shows `PARAMETERS` has content blocks, so the page comes from the file `mise-tasks/microfreak/PARAMETERS.md`, and none of the garden's other links to `.md` addresses created a page.

### 2 - Should the loader skip files git ignores?

Decided by the agent without codekiln: yes, outside `pages/` and `journals/`. Logseq's published build comes from a fresh checkout, which has no ignored files, so skipping them makes a local build match it. Inside the note folders the loader keeps reading every file, so untracked drafts publish as they do today.

### 3 - Should `:hidden` apply inside `pages/` and `journals/`?

Decided by the agent without codekiln: yes, with Logseq's prefix rule, because Logseq applies it to every file. No encode garden page matches today, so this removes nothing now.

### 4 - What happens when two files derive the same title?

Decided by the agent without codekiln: the file in `pages/` or `journals/` wins and the build names the other file. The note folders hold the pages the author maintains, and naming the dropped file makes the choice visible.

### 5 - Should symlinks outside the note folders stop the build?

Decided by the agent without codekiln: no, the loader skips them. They stop the build inside the note folders because a garden page should never be silently missing; elsewhere they belong to tooling the garden does not publish.

### 6 - Where does the title-collision check run?

Decided by the agent during Apply without codekiln: after loading, in an SDK method that the publisher and the lookup catalog both call. The loader cannot derive titles without the filename format, and keeping `load_garden` free of title rules keeps its contract unchanged for other callers.

### 7 - Does the page-title spec change?

Decided by the agent during Apply without codekiln: yes. `markdown-page-titles` rejected paths outside `pages/`, which would have stopped every build of the encode garden. The change now carries a delta that accepts any path outside `journals/`.

## Open Questions

None.
