## Why

codekiln's dotfiles now install the released `lsg` globally through mise ([codekiln/dotfiles#183 Install Logseq Gardener (lsg) globally with mise](https://github.com/codekiln/dotfiles/issues/183)), so a bare `lsg` works in every directory, including logseq-gardener checkouts. In a checkout it runs the release, while the code under test builds to that checkout's `target/release/lsg`. A contributor or agent who runs `lsg` to check a change sees the release's behavior ([Issue #337 — Keep checkout builds of lsg separate from the global install](https://github.com/codekiln/logseq-gardener/issues/337)).

## What Changes

- `mise.toml` puts the checkout's `target/release` directory first on PATH for shells with `mise activate`, for `mise exec` and for mise tasks. After `mise run build:release`, a bare `lsg` in those shells runs the checkout's build. Before the first build it runs the installed release.
- A shell that reaches tools through mise shims still runs the installed release, because a shim searches only that shell's own PATH. Coding agents' shells usually work this way. The agent instructions and `CONTRIBUTING.md` say to test a change with `./target/release/lsg`, and that `command -v lsg` shows which binary a bare `lsg` runs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-workflow`: adds the requirement "Checkout build runs as lsg in its checkout".

## Impact

- `mise.toml`: one `[env]` entry.
- `.rulesync/rules/overview.md` and the agent files RuleSync generates from it, and `CONTRIBUTING.md`.
- Tests, CI and the parity scripts keep their current binaries: the integration tests run `CARGO_BIN_EXE_lsg`, and `scripts/parity/common.py` builds its path from `CARGO_TARGET_DIR`.
- The README and `docs/installation.md` still say no binary release exists. The `modernize-docs` change rewrites both ([Issue #333 Rewrite the README for released binaries and mise installs](https://github.com/codekiln/logseq-gardener/issues/333), [Issue #327 Move install, CLI and tutorial docs into the docs graph](https://github.com/codekiln/logseq-gardener/issues/327)).

## Citations

- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Pref/Dev/mise](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___mise.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
