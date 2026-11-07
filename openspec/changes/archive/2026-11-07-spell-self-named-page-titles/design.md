## Context

Logseq's file parser titles a page in two steps. `build-page-map` in `deps/graph-parser/src/logseq/graph_parser/extract.cljc` gives the page the title of its file name or `title::` property. `with-ref-pages` in the same file then merges that page with every page the file's blocks refer to, grouped by lower-cased name, and the later maps win. Each reference's map comes from `page-name->map` in `deps/graph-parser/src/logseq/graph_parser/block.cljs`, which keeps the spelling of a page already in the database, or else uses the reference's own spelling. `parse-graph` transacts the files one at a time in the order `filter-files` gives, so a page is in the database when Logseq creates it at start, such as `TODO`, or when a file read earlier refers to it.

Within a block, `with-page-refs` adds each name it finds to the front of the block's list, so the names written in the text come last to first, followed by the namespace parents. The last reference to the page in the file therefore comes from the last block that names it, and within that block from a namespace parent, or else from the name written first.

`extract-page-alias-and-tags` drops an alias whose lower-cased name is the page's own, so the page gets no alias. The alias value is still a page reference of the properties block, and that reference gives the title.

Logseq's build of the encode garden, the `gh-pages` commit `8132c95` from garden commit `083106d`, titles `pages/Solvespace.md` `SolveSpace`, `pages/Brew.md` `brew`, and `pages/Claude___Code___MCP___json___How To___Create Atlassian MCP Json and Use it.md` `Claude/Code/MCP/json/How to/Create Atlassian MCP Json and Use it`. The SDK titled all three from their file names or `title::` properties.

## Goals / Non-Goals

**Goals:**

- Every caller of `page_title` gets the title Logseq gives a page that refers to itself in another spelling.
- `mise run docs:parser-check` reports no title difference for such a page.

**Non-Goals:**

- Titles of pages without a file. The parser check reports 14 such differences in the encode garden, from alias declarations and tags, which follow other parts of Logseq's parser.

## Decisions

- `title_spellings::spell_titles_as_logseq` runs once in `load_garden`, after `keep_each_block_id_once`, and stores each changed title in the new public field `GardenDocument::title_spellings`, keyed by the lookup key of the title the file name or `title::` property gives. `page_title` returns the stored spelling for that key. `load_garden` does not know the filename format, so the pass records a spelling for the title under each format; the two titles usually share one key.
- The relationship and naming rule lives in the SDK, as `block_ids` does for block ids. The publisher's code reads titles through `page_title` and needs no change.
- The pass reads a file's references with `Contribution::from_document`, the same reader the reference index and the parser check use. That reader re-parses each block's text, so the pass calls it only where a title can change. It first lists the names each file writes from the parse that loading already did: page references and tags, names in `alias::`, `aliases::` and `tags::` values, task markers and priorities, and the namespace parents of each. A title can change only when some file writes it in another spelling, or when it differs from Logseq's spelling of one of its own pages. On the encode garden, the pass reads the references of 41 files, and loading takes about 0.15 seconds longer: `lsg lookup`, which loads the whole garden, took a median 1.17 seconds against 1.02 before.
- `parse_order` from `block_ids` gives Logseq's file order, so both rules read the files in the same order.
- The parser check's deliberate mismatch is the fixture `scripts/logseq-parser/fixtures/alias-of-another-page`. Resolved Question 2 records why.

## Risks / Trade-offs

- [The listed names miss a name that `Contribution::from_document` reads] → The pass would then keep the title from the file name. On the encode garden and the docs graph, every name the reader returns is among the listed names.
- [Namespace selection is case-sensitive, so a page whose title changes case leaves a selection that names its old spelling] → `--include Solvespace` stops selecting `pages/Solvespace.md`. The encode garden's `site:build` and this repository's `parity:build` take their roots from file names and so leave out `SolveSpace`. Issue #350 would let both select every page with one flag.
- [A page that refers to itself in two spellings within one block, through two namespace parents] → The SDK takes the parent that sorts last; Logseq takes the last one it lists. Neither graph has such a block.

## Resolved Questions

### 1 - Does the rule cover only aliases, as the issue's title says?

Decided by the agent without codekiln: every reference, as Logseq's parser applies it. The alias is one case of `with-ref-pages`. The encode garden's two other title differences, `brew` and `How to`, follow the same rule, and Logseq's parser gives the same result for a tag, a namespace parent and a task marker in a test graph.

### 2 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: a page that lists another page with a file as its alias. `pages/brew.md` holds `alias:: [[Homebrew]]`, and Logseq also gives `Homebrew` the alias `brew`, which the SDK does not. The check reports one line about one file, the difference is current, and its cause lies in how Logseq records aliases, apart from titles. It is filed as [Issue #453](https://github.com/codekiln/logseq-gardener/issues/453). The other candidates were the title of an alias name without a file, which is close enough to this change that a later fix could fold it in, and journal references spelled with different weekdays, which depend on the garden's journal format.

### 3 - Should namespace selection follow the new titles or the file names?

Decided by the agent without codekiln: the new titles. Selection already matches the title `page_title` returns, and keeping one title for every purpose avoids a page that is selected under one spelling and published under another. The cost is that a selection written with the old spelling misses the page, which the reference page for titles now states.

## Open Questions

None.
