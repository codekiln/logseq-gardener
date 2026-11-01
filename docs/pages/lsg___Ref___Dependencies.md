tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Update Dependencies]], [[lsg/Licensing]], [[lsg/Architecture]]

- # Dependencies Reference
	- ## Overview
		- The direct Cargo dependencies of the `lsg` CLI and the SDK, and what each one does. Each manifest, [Cargo.toml](https://github.com/codekiln/logseq-gardener/blob/main/Cargo.toml) for the CLI and [crates/logseq-gardener-sdk/Cargo.toml](https://github.com/codekiln/logseq-gardener/blob/main/crates/logseq-gardener-sdk/Cargo.toml) for the SDK, repeats the reason in a comment beside each dependency.
	- ## CLI
		- | Package | Use |
		  | --- | --- |
		  | Clap | Parses arguments and reports usage errors. |
		  | Serde | Serializes the help sections. |
		  | serde_json | Produces the JSON help and version responses. |
		  | `logseq-gardener-sdk` | The local SDK package. It does the work of `lsg lookup` and `lsg publish`, and the CLI reports the version of the SDK it contains. |
	- ## SDK
		- | Package | Use |
		  | --- | --- |
		  | [lsdoc at its tested revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d) | Parses Markdown source and extracts references. [[lsg/Ref/Garden Loading]] gives its supported scope, and [[lsg/Parser Adoption]] records the decision to use it. |
		  | Serde and serde_json | Serialize the versioned selected-source search artifacts. They reuse the serialization packages the CLI already locks. |
		  | [flate2](https://github.com/rust-lang/flate2-rs) | Measures each search word shard's gzip size, so a shard splits before it exceeds its download budget. The package was already locked through syntect's grammar dumps. |
		  | [syntect](https://github.com/trishume/syntect) | Colours fenced code during the build, so published pages carry coloured HTML instead of an editor script. It reads TextMate grammars, with only its parsing, pure-Rust `fancy-regex` engine and grammar-dump features enabled, so the build needs no C compiler. |
		  | [two-face](https://github.com/CosmicHorrorDev/two-face) | Supplies the grammar set `bat` uses, which adds TOML, TypeScript, Dockerfile and other languages that the encode garden writes and syntect's defaults lack. The grammars add about 3.3 MB to the `lsg` binary. |
		  | [aho-corasick](https://github.com/BurntSushi/aho-corasick) | Finds every page's unlinked mentions in one pass over the garden's block text, matching all page names and aliases at once instead of scanning the garden once per page. The package was already locked through syntect's regular-expression engine. |
		  | unicode-normalization | Brings page names and search text to Unicode NFC, so names that differ only in how their characters are composed match. It also finds readable page folders that a case-insensitive file system would treat as one. |
	- ## Locked packages
		- `Cargo.lock` records the packages that the direct dependencies bring in, including packages used only on some platforms. To trace why a locked package is present, run `cargo tree -i <package>` from the repository root.
		- The dependency source inventory, [docs/dependency-sources.json](https://github.com/codekiln/logseq-gardener/blob/main/docs/dependency-sources.json), records the exact version, source, checksum and declared license of every locked package used in distribution, transitive and platform-specific packages included.
		- [[lsg/Licensing]] explains the inventory and the license files, and [[lsg/How To/Update Dependencies]] keeps them current.
