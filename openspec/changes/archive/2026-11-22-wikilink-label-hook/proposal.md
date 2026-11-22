# Proposal

## Why

Logseq has no labeled-wikilink syntax. It reads `[[Target|Label]]` as a link to one page named `Target|Label`, so a labeled link in `logseq-gardener-docs/` creates a stub page instead of linking to `Target`. Agents trained on Obsidian Markdown write these links often. codekiln's dotfiles now install a global mise task, `logseq:fix-wikilink-labels`, that rewrites them ([codekiln/dotfiles#209](https://github.com/codekiln/dotfiles/issues/209)).

## What Changes

- A lefthook `pre-commit` command runs `mise run logseq:fix-wikilink-labels` on staged Markdown files and stages the fixed files, so a commit never carries `[[Target|Label]]` into a graph page.
- The task runs once over the whole repository to fix existing pages. As of 2026-11-22 it finds none.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docs-graph`: adds a requirement that commits rewrite labeled wikilinks in graph pages to plain page references.

## Impact

- `lefthook.yml` gains a `pre-commit` section. The `post-merge` and `post-checkout` commands stay as they are.
- A contributor needs the global `logseq:fix-wikilink-labels` task from codekiln's dotfiles. Without it, the pre-commit command fails.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
