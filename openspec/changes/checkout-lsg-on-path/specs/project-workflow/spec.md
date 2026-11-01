## ADDED Requirements

### Requirement: Checkout build runs as lsg in its checkout

The repository's `mise.toml` SHALL put the checkout's `target/release` directory first on PATH for shells with `mise activate`, `mise exec` and mise tasks. The agent instructions and `CONTRIBUTING.md` SHALL say to test a change with `./target/release/lsg`, and that `command -v lsg` shows which binary a bare `lsg` runs.

#### Scenario: Built checkout

- **WHEN** a contributor runs `mise run build:release` in a checkout and then `mise exec -- lsg version` there
- **THEN** the checkout's `target/release/lsg` runs and reports the version in `Cargo.toml`

#### Scenario: Unbuilt checkout

- **WHEN** a checkout has no `target/release/lsg` and a contributor runs `mise exec -- lsg version` there
- **THEN** the installed `lsg` runs, and `mise install`, `mise run` and `mise exec` succeed

#### Scenario: Shell that uses mise shims

- **WHEN** an agent's shell reaches `lsg` through mise's shims directory in a checkout
- **THEN** the agent instructions direct it to `./target/release/lsg` for testing a change
