---
root: true
targets: ["*"]
description: "codekiln/logseq-gardener project instructions"
---

## OpenSpec

Use OpenSpec for planned changes. Before creating or editing OpenSpec artifacts, load the
matching OpenSpec skill from `.rulesync/skills/` and follow the artifact instructions
returned by the OpenSpec CLI. Do not invent the document format from memory.

Before using OpenSpec, resolve the local preferences repository with
`ghq list --full-path --exact github.com/codekiln/logseq-encode-garden`, then read and
follow `pages/My___Pref___Dev___AI___OpenSpec.md`. That page,
`My/Pref/Dev/AI/OpenSpec`, is the central source for the user's OpenSpec authoring
preferences. Follow the relevant preferences and principles it links to when drafting
and reviewing artifacts.

When archiving an OpenSpec change, always run the spec-sync workflow before moving the
change into the archive. Do not offer archive without syncing as a routine option.

Project-level AI configuration is generated from `.rulesync/`. Edit the RuleSync source,
then run `rulesync generate`; do not hand-edit generated tool files.

## Communication

Before writing or revising repository documentation or other project prose, read and
apply `.rulesync/rules/communication-style.md`. It defines plain language,
reader-centered information order, progressive disclosure for human and agent readers,
and the language to remove.

After changing `README.md`, run both `assess-readme-style` and
`assess-readme-purpose`. Resolve every reported problem or record the accepted
exception and its reason before considering the README change complete.

## mise

Use mise to manage project tools, environment settings, and tasks. Prefer executable file
tasks under `.mise/tasks/` to inline TOML tasks. Give each file task a `#MISE description`
and use `#USAGE` metadata for arguments and flags.

## ghq

Many CLI tools that are relevant to this project are likely installed locally; use `ghq list --full-path | rg <item>` to find where. Feel free to update the git references on any of them.

### example command lines to emulate or analyze
- `gh` 
- `mise` and `usage` (rg for `jdx` for all of Jeff Dickey's CLIs)
- `ripgrep`
- `rulesync`

### guidelines and standards relevant to ghq
- `agentskills`
- `cli-guidelines` 