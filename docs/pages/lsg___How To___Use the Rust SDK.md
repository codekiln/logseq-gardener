tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Garden Loading]], [[lsg/Ref/Page Titles]], [[lsg/Architecture]]

- # How To Use the Rust SDK
	- ## Overview
		- `logseq-gardener-sdk` is the Rust package that `lsg` is built on. Add it to a Rust project to load a garden's Markdown files and derive their page titles in your own program.
	- ## Prerequisites
		- Rust 1.97 or newer. The SDK uses the 2024 edition.
		- A Logseq Markdown garden: a folder that holds `pages/`, `journals/`, or both.
	- ## Steps
		- ### 1. Add the dependency
			- The SDK is not published on crates.io. Add it to your project's `Cargo.toml` as a git dependency on a release tag:
				- ```toml
				  [dependencies]
				  logseq-gardener-sdk = { git = "https://github.com/codekiln/logseq-gardener", tag = "v0.0.8" }
				  ```
			- To build against a checkout of the repository, use a path dependency on its `crates/logseq-gardener-sdk` folder:
				- ```toml
				  [dependencies]
				  logseq-gardener-sdk = { path = "../logseq-gardener/crates/logseq-gardener-sdk" }
				  ```
			- In a checkout, `mise exec -- cargo build -p logseq-gardener-sdk` builds the SDK without the command-line tool.
		- ### 2. Load a garden
			- `garden::load_garden` reads the garden's Markdown files, leaves them unchanged, and returns a `Garden`. Its `documents` field holds a `GardenDocument` for each file, with the file's original source, its path relative to the garden root, its `DocumentKind` (`Page` or `Journal`), its parsed blocks, and the page and block references the parser found. [[lsg/Ref/Garden Loading]] lists the files it reads.
			- Parsed blocks use the SDK's `ast` module, which re-exports the syntax tree of [lsdoc at its tested revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d). That syntax API follows the pinned parser, so a parser update can change it.
			- To see loading work, run the `load_garden` example from a checkout with the path of your garden:
				- ```sh
				  mise exec -- cargo run -p logseq-gardener-sdk --example load_garden -- /path/to/garden
				  ```
			- The example prints document and reference counts, such as `Loaded 89 Markdown documents: 89 pages, 0 journals; 120 page references, 0 block references.` It prints no note contents and leaves the garden unchanged.
		- ### 3. Derive page titles
			- `page_titles::page_title` derives a page's title from a loaded document and the garden's filename format. Use `FilenameFormat::TripleLowbar` when the garden's `logseq/config.edn` sets `:file/name-format :triple-lowbar`, and `FilenameFormat::Legacy` otherwise. [[lsg/Ref/Page Titles]] lists the rules.
			- This program prints each page's title and the source file it came from:
				- ```rust
				  use std::error::Error;
				  use logseq_gardener_sdk::garden::{DocumentKind, load_garden};
				  use logseq_gardener_sdk::page_titles::{FilenameFormat, page_title};
				  fn main() -> Result<(), Box<dyn Error>> {
				      let root = std::env::args().nth(1).ok_or("usage: garden-titles <garden-root>")?;
				      let garden = load_garden(&root)?;
				      for document in &garden.documents {
				          if document.kind == DocumentKind::Page {
				              let title = page_title(document, FilenameFormat::TripleLowbar)?;
				              println!("{title}\t{}", document.relative_path.display());
				          }
				      }
				      Ok(())
				  }
				  ```
			- The `page_titles` example in a checkout derives every page title of a garden and prints how many there are, how many contain `/`, and how many journals it skipped. It stops at the first title error and leaves the garden unchanged.
				- ```sh
				  mise exec -- cargo run -p logseq-gardener-sdk --example page_titles -- /path/to/garden triple-lowbar
				  ```
	- ## Troubleshooting
		- If `page_title` returns `PageTitleError::UnsupportedJournal`, the document is a journal. Check `document.kind` before you derive a title, as the program above does.
