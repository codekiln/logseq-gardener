## Context

Logseq's file parser gives each reference the spelling of a page that already exists (`page-name->map` in `deps/graph-parser/src/logseq/graph_parser/block.cljs`). A name in an `alias::` value is such a reference. So the first file Logseq reads that refers to a name, in any way, spells it. `title_spellings.rs` already applies this rule to a page's own title. An alias name that is no page's title kept the spelling of its `alias::` value, which gives a different spelling when another file refers to the name first.

`scripts/logseq-parser/check.py` showed the rule on small gardens. Each case below differs from the SDK before the change and agrees after it.

| Files | Logseq's spelling |
| --- | --- |
| `Run` lists `Runtime`; `Notes` refers to `[[runtime]]` | `runtime` |
| `Run` lists `Runtime`; `Zed` refers to `[[runtime]]` | `Runtime` (`Run` is read first) |
| `Run` lists `Runtime`; `Notes` refers to `[[runtime]]`, `Other` to `[[RUNTIME]]` | `runtime` (the first file) |
| One file lists `Runtime`, then refers to `[[runtime]]` and `[[RUNTIME]]` in two blocks | `RUNTIME` (the last block) |
| `Notes` refers to `[[runtime]], [[RUNTIME]]` in one block | `runtime` (the first name in a block) |
| `Brew` lists `homebrew, Homebrew`; `Zed` refers to `[[HomeBrew]]` | `Homebrew` (the last name in the properties) |

## Goals / Non-Goals

**Goals:**

- Every alias name that is no page's title takes the spelling Logseq gives it.
- `check.py` reports no difference on `fixtures/alias-spelling-reference`.

**Non-Goals:**

- Alias names that are also a page's title. The page's title spelling already covers them.
- Property names in the spelling of page titles. Logseq spells a page with the lower-case property name when a file writes the property, as the filed [Issue #521](https://github.com/codekiln/logseq-gardener/issues/521) shows for a block that also writes the page in another case.

## Decisions

- Extend `spell_titles_as_logseq`, which already finds the first file Logseq reads that refers to a name, and record the alias spellings in each document's `title_spellings` map under the alias's lookup key. A page title's lookup key never equals one of its own aliases' keys, so the map serves both without a new field in every `GardenDocument`.
- Do the work at load time, and read a file's references only for an alias name that some file writes in two spellings. A garden whose alias names have one spelling each costs one pass over the names the parse already holds.
- `page_alias_spellings` replaces each alias spelling with the recorded one, so the publisher, the search index and the comparison harness all get Logseq's spelling.
- A property name, such as `runtime::`, counts as a reference to the page `runtime` when the alias pass decides which file refers to the name first. A file with the property `runtime::` is why Logseq spells the encode garden's alias `runtime` in lower case.
- In the properties before the first bullet, the name written last wins. This keeps the rule of the earlier alias fix (`alias:: [[homebrew]], [[Homebrew]]` gives `Homebrew`).

## Risks / Trade-offs

- [The search data of a page whose alias a block refers to in another case changes spelling] → It is the spelling Logseq shows.

## Resolved Questions

### 1 - Which spelling does Logseq take when several references disagree?

The first file in Logseq's read order that refers to the name. Within that file, the last block that refers to it, and within one block the first name written. The cases are in the table under Context.

### 2 - Do the aliases `developer` and `repo` in the encode garden have the same cause?

Yes. `developer` is listed as an alias by `Programming/Programmer`, and `OpenVAS` has the property `developer::`, which Logseq reads as a reference in lower case. `repo`, `codespace` and `obsidian/feature/task lists` come from pages that spell the same alias in different case: Logseq takes the spelling of the file it reads first, and the SDK took the first in path order. This change fixes all four. `verification.md` has the numbers.

## Open Questions

None.
