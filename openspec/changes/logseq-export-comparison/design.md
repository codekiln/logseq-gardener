## Context

The `parity:` tasks compare `lsg` with the encode garden's `gh-pages` build, which a GitHub workflow produces with `logseq/publish-spa`. They cannot build Logseq's site of any other garden, so the docs graph has had no repeatable comparison with what Logseq shows. The parser check (`docs:parser-check`) compares parser output, not rendered pages.

For [Issue #369](https://github.com/codekiln/logseq-gardener/issues/369#issuecomment-6561215901) the export was built and compared by hand. That run settled how to build Logseq's export without Logseq desktop, and which page artifacts a text comparison has to handle.

- `logseq/publish-spa` wraps `logseq.publishing/export` from `deps/publishing`. Logseq's repository runs the same function from the command line with `scripts/src/logseq/tasks/dev/publishing.cljs`: `nbb-logseq -cp <publishing/src>:<graph-parser/src>:<db/src>:<common/src>:<scripts/src> -m logseq.tasks.dev.publishing STATIC_DIR GRAPH_DIR OUT_DIR`. The script resolves relative paths against `..`, so every path is passed absolute. It reads the graph and writes `index.html`, `404.html`, `static/` and `assets/` into OUT_DIR, with the graph's DataScript data inline in `index.html`.
- STATIC_DIR is a built publishing frontend: `404.html` and the `css`, `fonts`, `icons`, `img` and `js` folders, with `js/publishing/`. Logseq.app ships it as `Contents/Resources/app`. The `app` folders of Logseq.app 0.10.15 and of the official `Logseq-linux-x64-0.10.15.zip` hold the same files: a digest over `404.html`, `VERSION` and those folders, 400 files, is identical.
- The export logs `Failed to copy` for each asset a page names that the graph lacks. In the docs graph these are image names inside code examples.
- In Logseq's single-page app, `.page-title` also matches a favorite in the left sidebar; `document.title` names the open page. Changing only the URL hash leaves the old page's blocks on screen for a while. Long pages render lazily as they scroll. Logseq shows a count beside a referenced block (`a.open-block-ref-link`, the `block-refs-count` component in `frontend/components/block.cljs`), which `lsg` leaves out, as `lsg/Ref/Logseq Compatibility/Page Outline` records. Joining text nodes with spaces split `2px` where syntax highlighting puts `2` and `px` in separate spans.

## Goals / Non-Goals

**Goals:**

- One command that builds Logseq 0.10.15's export and the `lsg` site of a garden and reports, per page, the words and property rows that differ.
- Pinned, verified inputs: the same Logseq source, frontend and npm packages on every machine.
- Unit tests for the comparison that run in `mise run test`, and a browser check of the page reading on small fixtures.

**Non-Goals:**

- Comparing layout and styling. `parity:screenshots` shows both builds side by side for that.
- Comparing the Linked References, Unlinked References and Hierarchy sections below the blocks.
- Running the comparison in CI.

## Decisions

- **Task and files.** The task is `parity:logseq-export`, named for the build it adds; the other parity tasks read the `gh-pages` build. `scripts/parity/logseq_export.py` holds the work, with uv inline dependency `playwright==1.60.0` as `screenshots.py` has. `scripts/parity/logseq-export/` holds `source.json`, `package.json` and `package-lock.json`, and `npm ci` installs `node_modules` there, as `scripts/logseq-parser/` does.
- **Pinned source.** `source.json` names tag `0.10.15`, its commit `03bcefbd`, the five paths the export needs and the digest of those files, computed as `scripts/logseq-parser/check.py` computes its digest: the SHA-256 of a `shasum -a 256` listing sorted by path. The cache is `~/.cache/logseq-gardener/logseq-export-source/<commit>`, apart from the parser check's `logseq-source/`, which holds other paths of another commit. A cached copy is checked on every run. Without a local checkout the task fetches the commit with `--depth 1 --filter=blob:none` and a sparse checkout of the pinned paths, as check.py does.
- **Pinned frontend.** `source.json` also holds the frontend's version, the digest of its 400 files, and the release archive's URL, its SHA-256 as GitHub's release API reports it, and the folder inside it. Because Logseq.app and the archive hold the same files, one digest checks every source of the frontend. The task uses `--static-dir` when given, then `/Applications/Logseq.app/Contents/Resources/app` when its digest matches, then the cache, and downloads the archive only when none of these match. It unpacks only the files the export reads into `~/.cache/logseq-gardener/logseq-frontend/0.10.15` and deletes the archive. The export copies the frontend and leaves the source folder unchanged.
- **Garden and options.** `--garden` defaults to `docs`, and the docs graph gets `docs:site`'s options. Options for another garden follow `--`, as in `mise run parity:logseq-export --garden ~/garden -- --include notes --journals`. The bash task passes them from usage's `usage_lsg_option`, which usage shell-quotes, through `eval` into an array.
- **One server.** The run directory holds `logseq/` and `lsg/`, and one `serve.Server` serves the run directory, so one browser origin reads both builds.
- **Page list.** The pages to compare are the `lsg-site.json` entries of kind `page` or `journal`. Generated namespace pages have no blocks on either side, and the `home` entry repeats a page. Logseq's pages come from the DataScript data inline in the export's `index.html`, decoded with `logseq_db`; a Logseq page with blocks and no `lsg` entry is listed for information, since the `lsg` options decide what `lsg` selects.
- **Reading a page.** Each Logseq page loads from `about:blank`, waits until `document.title` equals the title, then scrolls `#main-content-container` and the window one screen at a time until both are at the bottom and the blocks' text length has stayed the same for three more reads, up to 400 screens. Scrolling one screen at a time brings every lazily rendered element, such as a code block, into view. The `lsg` page loads in the same browser and waits for `main h1`. Both reads run one script in the live page. On the Logseq side it collects the properties box rows, then removes the properties block (`.pre-block`), the badges (`.open-block-ref-link`) and CodeMirror's line numbers, and returns the `innerText` of `.page-blocks-inner`. On the `lsg` side it returns the rows of `.page-properties` and the `innerText` of `main > ul.outline`. `innerText` follows the layout, so a word split across highlight spans stays one word, and hidden text such as a collapsed block's children is left out on both sides.
- **Words.** Text is lower-cased and split into runs of letters, digits and underscores; `[[` and `]]` around Logseq's page references disappear with the split. Each side's words are counted, and the report lists each word one side shows more often than the other, with the difference when it is above one.
- **Property rows.** A row is its key and its value text, with `[[` and `]]` removed and spaces collapsed. The report lists rows only one build shows.
- **Report.** The run directory, `target/parity/<UTC time>/`, holds `logseq-export.json`, `logseq-export.md`, `logseq-export.log`, `lsg-diagnostics.txt`, and the two builds. It has no `settings.json`, so `parity:compare`, `parity:screenshots` and `parity:perf` keep using the latest `parity:build` run. The Markdown starts with the commits, versions and counts, then lists each differing page with its words and rows, the pages missing from Logseq, the Logseq pages outside the `lsg` build, and the files the export failed to copy.
- **Garden unchanged.** For a garden in a Git checkout, the task compares `git status --porcelain --ignored` for the garden folder before and after both builds and stops when it changed.
- **Tests.** `scripts/parity/test_logseq_export.py` tests word counting, property rows, the comparison, the export log reading, the page list, the report and the scrolling loop with a fake page whose text grows as it scrolls. `mise run test` runs them with the other parity tests. A browser test class in the same file loads fixture pages from `scripts/parity/testdata/logseq-export/`: a Logseq-shaped page with a badge, CodeMirror line numbers, a properties box, a word split across spans and blocks added on scroll, and an `lsg`-shaped page. It is skipped when Playwright is not installed, and the task runs it with Playwright before it builds.

## Risks / Trade-offs

- [The release archive is 202 MB] → It is downloaded only when no matching Logseq.app, cached frontend or `--static-dir` exists, and only the 130 MB the export reads is kept.
- [A page longer than 400 screens never reaches the bottom] → The loop stops at 400 screens, the page's entry notes that Logseq's page was still growing, and the words Logseq had not rendered show up as differences.
- [Comparing words hides word order and block structure] → A moved or nested block reads the same. The Markdown names the page, and `parity:screenshots` or a manual look covers structure.
- [Each Logseq page reloads the whole app] → A docs graph run reads 88 pages in about four minutes, which suits a task run by hand.

## Resolved Questions

### 1 - Where does the built publishing frontend come from when Logseq.app is missing?

From the official 0.10.15 release. GitHub's release API reports each asset's SHA-256, so the archive's digest is pinned without downloading it, and the frontend inside `Logseq-linux-x64-0.10.15.zip` matches Logseq.app's file for file. The fallback was run end to end on 2026-11-01.

### 2 - Which `lsg` options does the comparison use?

The options `docs:site` uses for the docs graph. Another garden takes its options as task arguments.

### 3 - How does the comparison treat the property rows `lsg` leaves out?

It reports them, and the task exits 1. Before [PR #389](https://github.com/codekiln/logseq-gardener/pull/389) fixed [Issue #385](https://github.com/codekiln/logseq-gardener/issues/385), the task reported the `alias` rows of three docs pages, such as `alias: Logseq Gardener` on the `lsg` page.

## Open Questions

None.
