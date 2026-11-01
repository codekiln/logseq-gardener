# Logseq Gardener

Logseq Gardener (`lsg`) is a command-line tool for people and coding agents who keep notes in a Logseq Markdown garden: the folder of pages and journals that Logseq calls a graph. Page aliases, references to individual blocks, and pages mentioned only through links make a garden difficult to navigate with filename searches alone.

`lsg` resolves those relationships by Logseq's rules and leaves every file in the garden unchanged:

- `lsg publish` turns the namespaces you choose into a static website that reads like a site Logseq 0.10 publishes, with the same outline, linked references, sidebars, search and graph view. Logseq's published site loads the whole garden into the browser. `lsg` writes each page as its own HTML file and evaluates queries, lays out the graph view and splits the search index when it builds the site, so the browser loads more data only when a reader asks for it. On codekiln's [public garden](https://github.com/codekiln/logseq-encode-garden), a page of the `lsg` site shows its title in a desktop browser in tens of milliseconds, and the same page of Logseq's site in about 20 seconds ([Performance](https://codekiln.github.io/logseq-gardener/lsg/Ref/Logseq%20Compatibility/Performance/)). [Logseq Compatibility](https://codekiln.github.io/logseq-gardener/lsg/Ref/Logseq%20Compatibility/) lists what `lsg` publishes as Logseq does and where it differs.
- `lsg lookup` prints the path of the file that holds a page named by its title, alias or journal date.

The project aims to help you find notes, follow relationships and review changes to your notes from the terminal and Neovim. `lsg publish` and `lsg lookup` are the commands that read a garden so far.

## Project status

This is an initial development version. [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) carry `lsg` binaries for macOS on Apple silicon and Intel, Linux on x86-64, and Windows on x86-64. [CI runs](https://github.com/codekiln/logseq-gardener/actions/workflows/ci.yml) show the results of the project checks on Linux, and the maintainer also runs them on Apple silicon macOS.

[@codekiln](https://github.com/codekiln) maintains the project. Ask for help or report a bug in [GitHub Issues](https://github.com/codekiln/logseq-gardener/issues).

## Install

Install a released `lsg` with mise, for every project on your computer or for the project in the current folder. mise must be installed and activated in your shell, as mise's [Getting Started](https://mise.jdx.dev/getting-started.html) guide shows.

```bash
# Every project: adds lsg to your global mise config
mise use -g github:codekiln/logseq-gardener
# The project in the current folder: adds lsg to its mise.toml
mise use github:codekiln/logseq-gardener
```

Each command installs the newest release. If mise already installed a version of `lsg`, the command uses that version. Commit the project's `mise.toml`, and everyone who works on the project gets `lsg` from `mise install`. Check the install:

```sh
lsg --version
```

It prints the installed version, such as `lsg 0.0.8`.

[Install with mise](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20with%20mise/) shows how to pin a version or move to the newest release, what to do when the shell cannot find `lsg`, and how mise checks each download against the release's SHA-256 checksum. To download a release archive and check its checksum yourself, follow [Install a Release Archive](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20a%20Release%20Archive/). To build `lsg` for another platform or from a checkout, follow [Build from Source](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Build%20from%20Source/).

## First use

These commands read the documentation built into `lsg` and report its version. They work offline and change no files.

```sh
lsg help outline
lsg help section behavior --recursive
lsg version --format json
```

`lsg help outline` lists the headings, each followed by the name in parentheses that `lsg help section` takes. `lsg help section behavior --recursive` prints the Behavior heading with its subheadings: what the commands do and which permissions they need. `lsg version --format json` prints the version as one JSON document for scripts and agents.

To publish your notes, start with the tutorial [Publish Part of a Public Garden](https://codekiln.github.io/logseq-gardener/lsg/Tutorial/Publish%20Part%20of%20a%20Public%20Garden/), which builds a site on your computer from two namespaces of a public garden. [Publish Selected Namespaces](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Publish%20Selected%20Namespaces/) shows how to publish your own garden, and [Publish a Garden to GitHub Pages](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Publish%20a%20Garden%20to%20GitHub%20Pages/) puts the site on the web. `lsg publish` applies Logseq's publishing rules to the pages you select, so a page marked `public:: false` stays off the site; [Selection and Visibility](https://codekiln.github.io/logseq-gardener/lsg/Ref/Selection%20and%20Visibility/) describes the rules.

To find the file that holds a page, by the page's title, alias or journal date, follow [Look Up a Page File](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Look%20Up%20a%20Page%20File/).

## Documentation

The [documentation site](https://codekiln.github.io/logseq-gardener/) holds every guide and reference page. The `docs` folder of this repository holds the same pages as a Logseq graph, which you can open in Logseq.

- [Command Line](https://codekiln.github.io/logseq-gardener/lsg/Ref/Command%20Line/) lists the commands, their JSON output and their exit codes.
- [Architecture](https://codekiln.github.io/logseq-gardener/lsg/Architecture/) explains how the Rust SDK and the command line divide the work.
- [Releases](https://codekiln.github.io/logseq-gardener/lsg/Releases/) explains how a merged pull request becomes a release.
- [Contribute a Change](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Contribute%20a%20Change/) shows how to take a change from plan to pull request, and [CONTRIBUTING.md](CONTRIBUTING.md) shows how to set up a checkout and run the project checks.
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) records the third-party code the project adapts, with its licenses and notices.

## License

Copyright © 2026 codekiln. Logseq Gardener is licensed under the [GNU Affero General Public License, version 3 only](LICENSE), without warranty. [Licensing](https://codekiln.github.io/logseq-gardener/lsg/Licensing/) explains the dependency notices and how each distributed build carries the source its licenses require.
