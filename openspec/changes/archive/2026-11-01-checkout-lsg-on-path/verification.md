# Verification: checkout-lsg-on-path

Under `mise exec`, a mise task and a shell with `mise activate`, a bare `lsg` in a checkout now runs that checkout's `target/release/lsg` once it is built, and the globally installed `lsg` before then. A shell that reaches `lsg` through mise's shims runs the installed `lsg` either way.

Checked on 2026-11-01 on Apple silicon macOS with mise 2026.10.25, in a worktree of this branch. The global mise config installs `"github:codekiln/logseq-gardener" = "latest"`, which resolved to 0.0.6. The checkout's `Cargo.toml` is at 0.0.7, so `lsg version` shows which binary ran. The mise task was a temporary file task running `command -v lsg` and `lsg version`. The activated shell was `zsh -i` with `eval "$(mise activate zsh)"` in its profile.

| How `lsg` was run | With `target/release/lsg` | Without `target/release/lsg` |
| --- | --- | --- |
| `mise exec -- lsg version` | checkout build, `lsg 0.0.7` | installed release, `lsg 0.0.6` |
| mise task | checkout build, `lsg 0.0.7` | installed release, `lsg 0.0.6` |
| Shell with `mise activate` | checkout build, `lsg 0.0.7` | installed release, `lsg 0.0.6` |
| Shell that uses mise shims | installed release, `lsg 0.0.6` | installed release, `lsg 0.0.6` |

Without `target/release/lsg`, `mise install` and `mise exec -- true` exited 0.

The alternatives in the design's resolved questions were checked in a scratch project with the same global config. With `"github:codekiln/logseq-gardener" = "path:./target/release"`, the shim ran the checkout build, but with no build `mise install`, `mise run` and `mise exec` failed while mise looked for a GitHub release named after the path. With `disable_tools = ["github:codekiln/logseq-gardener"]`, the shim stopped with "lsg is not a valid shim" whether or not the build existed.

After the merge, the cleanup worktree `.worktrees/345-sync-archive-checkout-lsg-on-path`, which sits inside the main checkout, had no build while the main checkout had one at 0.0.8. There `mise exec -- lsg version` ran the main checkout's `target/release/lsg` and printed `lsg 0.0.8`, because mise also reads the main checkout's `mise.toml`. With a stand-in `target/release/lsg` in the worktree, the worktree's own binary ran first. The scenario "Unbuilt checkout" now names the enclosing checkout's build.
