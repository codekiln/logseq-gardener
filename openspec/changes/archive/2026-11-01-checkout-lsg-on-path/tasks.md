## 1. Checkout build on PATH

- [x] 1.1 Add an `[env]` table to `mise.toml` with `_.path = ["{{config_root}}/target/release"]` and a comment saying why
- [x] 1.2 Add the testing instruction to `.rulesync/rules/overview.md`, then run `mise exec -- rulesync generate` and `mise run rulesync:check`
- [x] 1.3 Say in `CONTRIBUTING.md` which binary a bare `lsg` runs in a checkout and how to test a change

## 2. Verification

- [x] 2.1 In a worktree, with and without `target/release/lsg`, record in `verification.md` what `mise exec -- lsg version`, a mise task, a shell with `mise activate` and the shim run
- [x] 2.2 Run `mise run openspec:validate` and `mise run ci`
