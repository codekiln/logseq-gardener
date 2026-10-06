# Page-title validation

The SDK derived titles for every page in the local [codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden) checkout on October 6, 2026. The checkout was based on commit `e12f9568693662aee21c0c61b9787a767a56d4e9` and included user changes, so this result describes the local snapshot rather than that commit alone.

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example page_titles -- /path/to/logseq-encode-garden triple-lowbar
```

```text
Derived 6455 page titles: 5786 contain namespace separators; skipped 488 journals.
```

The graph sets `:file/name-format :triple-lowbar` in `logseq/config.edn`. SHA-256 hashes of all 6,943 visible Markdown files beneath `pages/` and `journals/` matched before and after the run. Garden files were read without modification.

Seven integration tests check the saved relationship fixtures' page titles, format-specific percent decoding, leading property placement and duplicate precedence, Contents, invalid paths, empty titles, non-UTF-8 stems, and document preservation. A Windows-specific native-path test is present but was not run on macOS.

This demonstrates page-title extraction on a real garden. It does not demonstrate unique lookup, alias resolution, journal date formatting, publication filtering, or a usable generated site. The next publishing step is to apply namespace selection to these titled documents and render a small local site, retaining explicit diagnostics for links whose destinations are excluded or unresolved.

The complete `mise run ci` checks passed locally. An independent functional review found no discrepancies within the documented title scope.
