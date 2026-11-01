tags:: [[Diataxis/Explanation]]
see-also:: [[lsg/Parser Adoption]], [[lsg/Static Publishing]], [[lsg/Ref/Dependencies]]

- # Architecture
	- Logseq Gardener keeps its Rust API apart from its command-line interface. The SDK package, `logseq-gardener-sdk`, reads, looks up and publishes a garden, and the `lsg` command line calls it. Other Rust applications can depend on the SDK without installing `lsg`; [[lsg/How To/Use the Rust SDK]] shows how.
	- ## Packages and modules
		- | Component | Responsibility |
		  | --- | --- |
		  | `logseq-gardener-sdk` in `crates/logseq-gardener-sdk/` | The reusable Rust API. It exposes the SDK version, a logical namespace-selection policy, a read-only Markdown loader and configured page-title extraction. It publishes selected pages as a static HTML site. Its page catalog finds a page's source file by title, alias or journal date for `lsg lookup`. |
		  | `logseq-gardener` at the repository root | The CLI package, which builds `lsg` and depends on the SDK. |
		  | `src/cli.rs` | Validates arguments and dispatches help, version, lookup and publish requests. |
		  | `src/help.rs` | Holds the help document and selects its sections for text or JSON output. |
		  | `src/output.rs` | Writes results and diagnostics to their output streams and handles write failures. |
		- The CLI handles terminal input and output. Garden loading belongs in the SDK, so the CLI and other Rust applications call the same code.
		- The source is in [crates/logseq-gardener-sdk](https://github.com/codekiln/logseq-gardener/tree/main/crates/logseq-gardener-sdk) and [src](https://github.com/codekiln/logseq-gardener/tree/main/src).
	- ## Garden files and parsing
		- Markdown files hold the garden's notes. Any future index must be rebuildable from those files, and an editing command should be able to change a note without reformatting unrelated text.
		- The Markdown reader uses the tested lsdoc revision. Official mldoc provides the syntax reference, and Logseq OG's graph parser and graph-validator provide the references for relationships between notes. The [parser comparison findings](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/parser-comparison-evidence/experiments/parser-comparison/findings.md) favor lsdoc for read-only Markdown parsing on the tested fixtures and public gardens.
		- SDK modules above the parser turn its output into pages and relationships:
			- `page_titles` derives titles with an explicit filename format, as [[lsg/Ref/Page Titles]] describes.
			- `page_names` keeps title and alias candidates, `page_lookup` finds source files, and `journals` interprets journal dates.
			- `outline`, `references` and `transclusion` supply outline identities, relationships and embedded content to the publisher.
		- The loader keeps source text and ranges alongside the parsed structure.
		- The [current Logseq OG relationship comparison](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/current-og-relationships/experiments/file-relationships/findings.md) reproduces the earlier fixture relationships under default and configured filename interpretation. The results preserve the recorded page lookup rules for titles, aliases, namespaces, journals and pages without a file.
		- [[lsg/Parser Adoption]] records the decision: lsdoc parses syntax in the SDK, and the SDK modules above it apply Logseq's relationship rules for every caller.
	- ## Publishing
		- The SDK owns namespace selection, visibility, reference resolution and HTML generation.
			- `publication` chooses the published pages, assigns their routes and builds the read-only view that build plugins receive.
			- `contributions` checks what build plugins return and resolves their links to routes and anchors.
			- The publisher in `site` renders what those modules decide.
		- The CLI validates the options of `lsg publish` and writes results and diagnostics to their output streams.
		- [[lsg/Static Publishing]] explains what `lsg publish` does with a garden, and [[lsg/How To/Publish Selected Namespaces]] runs it on a real garden.
