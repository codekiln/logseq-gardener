# Dependencies

The CLI depends on Clap to parse arguments and report usage errors. It uses Serde to serialize help sections and serde_json to produce JSON help and version responses. The CLI also depends on the local `logseq-gardener-sdk` package to report the version of the SDK it contains. The SDK uses [lsdoc at its tested revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d) to parse Markdown source and extract references. See [Garden loading](garden-loading.md) for its supported scope.

The SDK also uses Serde and serde_json to serialize its versioned selected-source search artifacts. These reuse the already locked serialization packages.

The publisher colours fenced code during the build, so published pages carry coloured HTML instead of an editor script. [syntect](https://github.com/trishume/syntect) reads TextMate grammars, with only its parsing, pure-Rust `fancy-regex` engine and grammar-dump features enabled, so the build needs no C compiler. [two-face](https://github.com/CosmicHorrorDev/two-face) supplies the grammar set `bat` uses, which adds TOML, TypeScript, Dockerfile and other languages the encode garden writes that syntect's defaults lack. The grammars add about 3.3 MB to the `lsg` binary.

The SDK's reference index finds every page's unlinked mentions in one pass over the garden's block text with [aho-corasick](https://github.com/BurntSushi/aho-corasick), which matches all page names and aliases at once instead of scanning the garden once per page. The package was already locked through syntect's regular-expression engine.

Those are the project's direct dependencies. Cargo.lock records the packages they bring in, including packages used only on some platforms. To trace why a locked package is present, run `cargo tree -i <package>` from the repository root. For the exact versions, sources, checksums, and declared licenses used in distribution, see the [dependency source inventory](dependency-sources.json). The [licensing guide](licensing.md) explains how the inventory and license files are maintained.
