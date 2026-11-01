# Verification: publish-unsafe-titles-at-digest-urls

A readable build of the encode garden no longer stops on titles with a literal slash. The three pages from issue #363 and their three unsafe namespace prefixes are published at their `p-<digest>.html` files, each page gets a diagnostic, and every other page keeps its folder.

Checked on 2026-11-01 against the local checkout of codekiln/logseq-encode-garden at `4d37b9ff`, with that garden's `mise-tasks/site/build` script from its `origin/main` (`7ed46a65`) plus `--readable-urls`. The script includes every namespace root and passes `--journals --search --timings`.

## Before: `lsg` built from logseq-gardener `origin/main` (`4573370`)

The build exits 1 and writes no output:

```
error: publication failed: page "Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?" cannot have a readable URL: a segment starts or ends with whitespace; use 'lsg publish help'
```

## After: `lsg` built from this branch

The unsafe-title failure is gone. The next failure is a letter-case collision between folders, such as `CLAUDE` and `Claude/Artifact`, which [Issue #364](https://github.com/codekiln/logseq-gardener/issues/364) tracks. With the 16 pages that #364 reports passed to `--exclude`, the build completes:

- 13,854 pages in `lsg-site.json`, 13,449 of them at folder URLs.
- Seven pages at digest files: the three pages below and four generated namespace pages whose titles end with a space:
  - `Claude/Code/Q/If I rename a CLI session with `
  - `Claude/Code/Q/Is there a Claude Code CLI slash command to see a diff of changed files, and why doesn't `
  - `Obsidian/Q/In Obsidian, is there a command that inserts a callout, and what are those colored Info `
  - `Obsidian/Q/In Obsidian, is there a command that inserts a callout, and what are those colored Info / Warning `
- Three diagnostics, one per page:

```
pages/Claude___Code___Q___If I rename a CLI session with %2Frename, can I resume it in Claude Desktop's Code tab%3F.md: page "Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?" cannot have a readable URL: a segment starts or ends with whitespace; it is published at p-2b9672f05aabc9af.html
pages/Claude___Code___Q___Is there a Claude Code CLI slash command to see a diff of changed files, and why doesn't %2Fdiff show the diff content%3F.md: page "Claude/Code/Q/Is there a Claude Code CLI slash command to see a diff of changed files, and why doesn't /diff show the diff content?" cannot have a readable URL: a segment starts or ends with whitespace; it is published at p-44728145061528c8.html
pages/Obsidian___Q___In Obsidian, is there a command that inserts a callout, and what are those colored Info %2F Warning %2F Danger blocks called%3F.md: page "Obsidian/Q/In Obsidian, is there a command that inserts a callout, and what are those colored Info / Warning / Danger blocks called?" cannot have a readable URL: a segment starts or ends with whitespace; it is published at p-475a82ff83a5dbda.html
```

## Browser check

`mise run readable:browser-test` publishes a fixture garden with the page `Docs/Q/Rename with /rename, then resume` and checks, in Chromium under a path prefix, that a link, a search result, the page graph, the graph view's route and plugin host navigation all reach its `p-<digest>.html` file:

```
{"prefix_links": true, "zoom": true, "search": "Docs/Guide/Start/#source-block-172", "sidebar": true, "digest_fallback": "p-c3b0ec0874e47d7e.html", "page_graph": true, "graph_route": true, "lists": true, "host": true, "disk": true}
```
