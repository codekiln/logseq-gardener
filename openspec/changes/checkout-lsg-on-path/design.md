## Context

mise can reach `lsg` in a checkout three ways. A shell with `mise activate` puts the installed release's directory on PATH. `mise exec` and mise tasks build PATH from the checkout's `mise.toml` and the global config. A shell without `mise activate` finds the `lsg` shim in mise's shims directory, and the shim picks a binary from the configured tools, falling back to the shell's own PATH. Coding agents' shells usually take the shim route, because they start without an interactive profile.

`mise run build:release` writes `target/release/lsg`.

## Goals / Non-Goals

**Goals:**

- A bare `lsg` under `mise activate`, `mise exec` or a mise task runs the checkout's build once one exists.
- Every agent knows how to run the checkout's build, whatever its shell does with a bare `lsg`.

**Non-Goals:**

- Give each `lsg` version its own cache directory once `lsg` keeps a cache, so a release and a checkout build never read each other's entries. `lsg` has no cache today.

## Decisions

- Prepend `{{config_root}}/target/release` with mise's `_.path` in `mise.toml`. `{{config_root}}` is the checkout that holds the `mise.toml`, so each worktree finds its own build. mise skips the entry while the directory or binary is missing, so an unbuilt checkout runs the installed release.
- Use `target/release` because `mise run build:release`, the README and the parity scripts all use the release build.
- Tell agents and contributors to test a change with `./target/release/lsg` and to check `command -v lsg` when unsure, because a shim ignores `_.path`. Release preparation raises the version in `Cargo.toml` after each release, so `lsg version` usually tells a checkout build from the installed release too.

## Risks / Trade-offs

- [A shell with `mise activate` and an agent's shell run different binaries for the same `lsg` command in a checkout] → The agent instructions name `./target/release/lsg` for testing a change.
- [A bare `lsg` runs a checkout build older than the source after an edit without a rebuild] → The same holds for `./target/release/lsg`; contributors rebuild with `mise run build:release` before checking a change.

## Resolved Questions

### 1 - Can the shim run the checkout's build?

Decided by the agent without codekiln: no, without breaking a fresh clone. Declaring `"github:codekiln/logseq-gardener" = "path:./target/release"` in `mise.toml` makes the shim run the checkout build once it exists, but in a clone without a build mise tries to download that version from GitHub, and `mise install`, `mise run` and `mise exec` fail. Adding the tool to `disable_tools` makes the shim stop with "lsg is not a valid shim" even after a build, because the shim's fallback searches only the shell's own PATH, which lacks `_.path`.

### 2 - Should a bare `lsg` in an unbuilt checkout run the installed release?

Decided by the agent without codekiln: yes. mise skips a missing `_.path` directory. The configurations that would stop instead, a `path:` version and `disable_tools`, break a fresh clone and the shim.

## Open Questions

None.
