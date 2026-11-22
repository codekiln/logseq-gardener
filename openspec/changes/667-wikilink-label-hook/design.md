# Design

## Context

The repository already uses lefthook for `post-merge` and `post-checkout` commands that regenerate RuleSync output. It has no `pre-commit` section. The fixer is a global mise file task from codekiln's dotfiles, not a project task.

## Goals / Non-Goals

**Goals:**

- Stop labeled wikilinks at commit time, before they reach main or the published site.

**Non-Goals:**

None.

## Decisions

- Run the fixer as a lefthook `pre-commit` command with `stage_fixed: true`. The command fixes the page and adds the fix to the same commit, so the contributor does not need a second commit.
- Pass `{staged_files}` filtered by `glob: "*.md"`. The task itself skips files outside a Logseq graph, so the README and OpenSpec files keep their own Markdown.
- Call the global task instead of copying it into `.mise/tasks/`. codekiln uses the same task in other gardens, and one copy is easier to keep correct.

## Risks / Trade-offs

- [A contributor without codekiln's dotfiles cannot commit Markdown] → The pre-commit command fails with mise's unknown-task error, which names the task. Today codekiln and their agents are the only contributors.
- [Test fixtures under `tests/fixtures/` and `scripts/logseq-parser/fixtures/` are Logseq graphs too, so the fixer would rewrite a fixture that needs a labeled wikilink] → None does today. A fixture that needs one can be committed after adding an `exclude` to the command.

## Resolved Questions

### 1 - Should the hook run only on `logseq-gardener-docs/`?

No. The task already limits itself to files inside a Logseq graph, and no fixture contains a labeled wikilink. Narrowing the glob now would add configuration for a case that does not exist.

## Open Questions

None.
