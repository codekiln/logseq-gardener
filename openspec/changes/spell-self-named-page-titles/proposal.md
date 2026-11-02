## Why

When a page's file refers to the page in another spelling, Logseq titles the page with that spelling, and `lsg` keeps the spelling of the file name ([Issue #444 — Title a page with its own alias's spelling, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/444)). The encode garden's `pages/Solvespace.md` holds `alias:: [[SolveSpace]]`: Logseq's published page is titled `SolveSpace`, and `lsg` titles it `Solvespace`.

An alias is one case of a wider rule in Logseq's file parser. Any reference in the file counts, including a tag, a namespace parent and a task marker. When a file Logseq reads earlier already refers to the page, the earlier spelling wins: a journal that writes `[[brew]]` makes `pages/Brew.md` the page `brew`.

## What Changes

- When the SDK loads a garden, it records the spelling Logseq gives each page that refers to itself in another spelling, and `page_title` returns that spelling. Every caller of `page_title` sees it: published titles, routes, links, search, the graph, namespace selection, `lsg lookup` and the parser check.
- In the encode garden, three pages change title, as in Logseq's build: `Solvespace` becomes `SolveSpace`, `Brew` becomes `brew`, and `Claude/Code/MCP/json/How To/Create Atlassian MCP Json and Use it` becomes `Claude/Code/MCP/json/How to/Create Atlassian MCP Json and Use it`.
- The parser check's deliberate mismatch moves from `fixtures/alias-title-case`, which now agrees, to `fixtures/alias-of-another-page`: a page that lists another page with a file as its alias, which Logseq records on both pages. [Issue #453](https://github.com/codekiln/logseq-gardener/issues/453) tracks that difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-page-titles`: a new requirement, "Spell a page that refers to itself as Logseq does", says which spelling a page takes when its file refers to it.

## Impact

- `crates/logseq-gardener-sdk/src/title_spellings.rs` (new): Logseq's rule, run once when the garden loads.
- `crates/logseq-gardener-sdk/src/garden.rs`: `GardenDocument` gains the public field `title_spellings`, which `load_garden` fills. Code that builds a `GardenDocument` itself sets the field, usually to an empty map.
- `crates/logseq-gardener-sdk/src/page_titles.rs`: `page_title` returns the recorded spelling.
- Namespace selection is case-sensitive, so `--include Solvespace` stops selecting `pages/Solvespace.md`; `--include SolveSpace` selects it. The encode garden's `mise run site:build` and this repository's `parity:build` take their namespace roots from file names, so both leave out `SolveSpace` until they include that root or [Issue #350 — Select every page and journal with one publish flag](https://github.com/codekiln/logseq-gardener/issues/350) lands.
- `scripts/logseq-parser/`: the fixture `alias-of-another-page` replaces `alias-title-case`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
