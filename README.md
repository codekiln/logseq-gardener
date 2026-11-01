# Logseq Gardener

Logseq Gardener (`lsg`) is a command-line tool for people and coding agents who keep notes in Logseq Markdown gardens. Page aliases, references to individual blocks, and pages mentioned only through links make these gardens difficult to navigate with filename searches alone.

The goal is to help you find notes, follow relationships, and review changes from the terminal and Neovim using the notes saved in your Markdown files. This version publishes selected namespaces as local HTML pages, prints the source file of a page named by its title, alias or journal date, and provides offline help and version reporting.

## Project status

This is an initial development version. [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) carry `lsg` binaries for Apple silicon and Intel macOS, x86-64 Linux, and x86-64 Windows. The project checks run on Apple silicon macOS and Linux. See [Install with mise](docs/pages/lsg___How%20To___Install%20with%20mise.md) to install a release, or [Install a Release Archive](docs/pages/lsg___How%20To___Install%20a%20Release%20Archive.md) to download an archive and check its checksum.

[@codekiln](https://github.com/codekiln) maintains the project. See [CI runs](https://github.com/codekiln/logseq-gardener/actions/workflows/ci.yml) for check results and [GitHub Issues](https://github.com/codekiln/logseq-gardener/issues) for help or bug reports. The project uses [AGPLv3](LICENSE).

## Build from source

In a checkout, with [mise](https://mise.jdx.dev/) installed:

```sh
mise install
mise run build:release
```

This creates `target/release/lsg` (`lsg.exe` on Windows). The first build downloads development tools and dependencies. See [Build from Source](docs/pages/lsg___How%20To___Build%20from%20Source.md) for build requirements and PATH setup.

## First use

```sh
./target/release/lsg help outline
./target/release/lsg help section behavior --recursive
./target/release/lsg version --format json
```

The outline lists documentation headings. The section command explains what the commands do and which permissions they need. The final command returns one JSON document identifying your installed version. These commands work offline and leave garden files unchanged.

To publish your notes, follow [lsg/How To/Publish Selected Namespaces](docs/pages/lsg___How%20To___Publish%20Selected%20Namespaces.md). The page covers namespace selection and fresh output directories, and links the pages on rendering limits. Publishing includes journals only with `--journals` and withholds whole pages containing a parsed `public:: false` property. To open a page's source file by name, see [Look Up a Page File](docs/pages/lsg___How%20To___Look%20Up%20a%20Page%20File.md).

## Learn more

- [Navigate help and use JSON from scripts](docs/pages/lsg___Ref___Command%20Line.md)
- [Understand the SDK, CLI, and parser choice](docs/pages/lsg___Architecture.md)
- [Contribute and run the project checks](CONTRIBUTING.md)
- [Prepare and distribute releases](docs/pages/lsg___Releases.md)
- [Review dependency licenses and source notices](THIRD_PARTY_NOTICES.md)

## License

Copyright © 2026 codekiln. Logseq Gardener is licensed under the [GNU Affero General Public License, version 3 only](LICENSE), without warranty. See [dependency licensing](docs/pages/lsg___Licensing.md) for the notice and source-distribution process.
