# Logseq Gardener

Logseq Gardener (`lsg`) is a command-line tool being built for people and coding agents who keep notes in Logseq Markdown gardens. Page aliases, references to individual blocks, and pages mentioned only through links make these gardens difficult to navigate with filename searches alone.

The goal is to help you find notes, follow relationships, and review changes from the terminal and Neovim while keeping your Markdown files authoritative. This foundation provides offline help and version reporting. Garden lookup commands will follow a parser compatibility comparison.

## Project status

This is an initial development version, with no published binary release yet. The distribution configuration targets Apple silicon and Intel macOS, x86-64 Linux, and x86-64 Windows. Local foundation verification uses Apple silicon macOS; CI checks the project on Linux. These are development targets, not a compatibility guarantee for garden operations.

[@codekiln](https://github.com/codekiln) maintains the project. See [CI runs](https://github.com/codekiln/logseq-gardener/actions/workflows/ci.yml) for check results and [GitHub Issues](https://github.com/codekiln/logseq-gardener/issues) for help or bug reports. The project uses [AGPLv3](LICENSE).

## Build from source

In a checkout, with [mise](https://mise.jdx.dev/) installed:

```sh
mise install
mise run build:release
```

This creates `target/release/lsg` (`lsg.exe` on Windows). The first build downloads development tools and dependencies. See [Installation](docs/installation.md) for PATH setup and future binary verification.

## First use

```sh
./target/release/lsg help outline
./target/release/lsg help section behavior --recursive
./target/release/lsg version --format json
```

The outline lists documentation headings. The section command explains what the commands do and which permissions they need. The final command returns one JSON document identifying your installed version. These commands work offline and leave garden files unchanged.

## Learn more

- [Navigate help and use JSON from scripts](docs/cli.md)
- [Understand the parser comparison and planned architecture](docs/architecture.md)
- [Contribute and run the project checks](CONTRIBUTING.md)
- [Prepare and distribute releases](docs/releases.md)
- [Review dependency licenses and source notices](THIRD_PARTY_NOTICES.md)

## License

Copyright © 2026 codekiln. Logseq Gardener is licensed under the [GNU Affero General Public License, version 3 only](LICENSE), without warranty. See [dependency licensing](docs/licensing.md) for the notice and source-distribution process.
