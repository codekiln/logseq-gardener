# Parser adoption

**Decision:** Gardener adopts [lsdoc 0.5.8 at revision 32e63ef0](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d) as the parser inside the SDK, for reading Logseq Markdown. lsdoc turns each note into blocks, inline syntax, properties and the references written in the note. Logseq's rules for how notes relate to each other stay in SDK modules above the parser. The CLI, the publisher and any future language server use those SDK modules and do not call lsdoc directly.

The SDK is the shared core that codekiln's plan calls `garden-core`: one parser, graph model and index for every Gardener tool. The decision was recorded under [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3).

## What the parser covers

lsdoc is responsible for syntax:

- the outline: blocks, nesting, headings and source ranges, so the SDK can keep each note's original text;
- inline syntax: emphasis, links, images, macros, tags, code and math;
- page and block properties, including `public::`, `title::`, `alias::` and `tags::`, as written;
- embedded media as written: images, asset links, `{{embed}}`, `{{youtube}}`, `{{video}}`, `{{tweet}}` and other macros, and raw HTML; and
- the page and block references a note contains, following Logseq's reference extraction.

## Where relationship rules live

The SDK applies Logseq's graph rules to the parser's output. These rules depend on graph configuration and on other notes, so they do not belong in the parser:

| Rule | SDK module |
| --- | --- |
| Page names from file names, `title::` and `:file/name-format` | `page_titles`, `page_names` |
| Aliases and finding a page's file | `page_names`, `page_lookup` |
| Journal titles and links written as dates | `journals` |
| Namespace parents and hierarchy | `namespaces` |
| Linked references, tags and graph edges | `references`, `inline_references`, `graph` |
| Block references and embeds | `transclusion` |
| Video macros and timestamps | `media` |
| What gets published, including `public:: false` | `publishing`, `site` |

The corpus comparison below shows why this split works. lsdoc and mldoc, the parser inside Logseq, agree on every note's syntax and references. The extra links in Logseq's graph come from graph rules: task markers, property names and values, namespace parents and journal date names.

## Evidence

lsdoc's output matched mldoc 1.5.9 on three real gardens, ignoring source positions and table alignment. Source positions are checked separately. The comparison started a fresh mldoc process for each note.

| Garden | Notes | Syntax or reference differences | Parse time, lsdoc / mldoc |
| --- | --- | --- | --- |
| [Logseq docs](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/docs-report.md) | 333 | 0 | 10 ms / 778 ms |
| [Encode garden](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/encode-report.md) | 6,893 | 1, math inside bold text | 157 ms / 10,699 ms |
| [Pengx's knowledge garden](../experiments/parser-corpus/results.json) | 322 | 0 | 3 ms / 416 ms |

Parse time is the parser's own reported time, summed over the garden; it excludes process start-up. The [parser findings](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/findings.md) describe the fixtures, the math difference and source preservation. The [current Logseq OG comparison](../openspec/changes/current-og-relationships/experiments/file-relationships/findings.md) shows that current and historical Logseq build the same page and block relationships from the shared fixtures.

### Pengx's knowledge garden

[Pengx's knowledge garden](https://github.com/pengx17/knowledge-garden) is a published Logseq garden with no license. Gardener read it locally at revision `cdec71d8` and committed only [the script](../experiments/parser-corpus/README.md) and aggregate counts. The comparison ran once.

- **Syntax and references:** all 322 Markdown notes produced the same syntax and references in both parsers. There were no crashes or timeouts.
- **Embedded media:** both parsers found the same media in every note: 43 images, 40 of them local assets; 5 page or block embeds; 3 YouTube, 11 tweet and 6 renderer macros; and 6 HTML media elements.
- **Properties:** both parsers found the same 62 page property blocks, including 39 `title::`, 14 `tags::` and 6 `alias::`, and the same 170 block property blocks. The garden has no `public::` property. It publishes every page through the older graph-wide `:all-pages-public? true` setting.
- **Logseq relationships:** Logseq OG built 570 pages, 313 of them from files; it skips the 9 empty files. In 203 of the 313 files, Logseq linked exactly the pages that lsdoc reported as references. In the other files, Logseq added 172 links: 75 task markers such as `DONE` and `LATER`, 44 namespace parents, 31 property values, 15 property names, 4 block identifiers, 2 journal date names and 1 other. lsdoc reported 8 references that Logseq does not keep as written: 3 asset paths, 2 dates that Logseq renames to the garden's journal title format, 2 property values and 1 other.
- **Gardener end to end:** `lsg publish --journals` completed and published 170 pages, 144 journals and 40 assets, with 356 diagnostics. Most diagnostics are expected fallbacks: 207 links to pages without a file, 75 drawers and 33 HTML or Hiccup blocks kept as text, and 21 unsupported macros. The others include 1 unsupported journal title token, 3 journal files whose names are not dates, 4 missing block references and 2 advanced queries.

## Supported scope

- Logseq file gardens, the format of Logseq OG's `version/file` branch, read without changing any file.
- Markdown notes; see [Garden loading](garden-loading.md) for which files the SDK reads.
- Syntax and references equal to mldoc 1.5.9 on the gardens above, apart from the math difference below.
- Relationship rules checked against Logseq 0.9.8 and current Logseq OG on the shared fixtures.

## Limitations

- **Math inside bold or italic text.** lsdoc reads `$…$` inside emphasis as math; mldoc does not. lsdoc records this as an intentional difference, D48 in its [decisions](https://github.com/martinkoutecky/lsdoc/blob/32e63ef095c711d6d9947257bf5fd07d540fa59d/DECISIONS.md). A renderer that must match Logseq exactly needs its own handling. References are unaffected.
- **Editing.** The SDK keeps source text and ranges, but no test covers formatting-preserving edits. Writing notes needs separate evidence.
- **Org notes.** Only the Logseq docs garden contains Org notes, and the SDK reads Markdown only.
- **Logseq's database version** is out of scope.
- **Journal title tokens.** The SDK does not support the weekday token `E` in `:journal/page-title-format`, which Pengx's garden uses.
- **Relationships on whole gardens.** The relationship comparison above compares reference sets per file. It does not compare the SDK's own graph with Logseq's graph. Query results, graph-validator checks and publication filtering on real gardens remain untested.
- **Current Logseq revision.** The fixture comparison used current Logseq OG at `6efedb75`. Pengx's garden ran against `6e7afa8e`, the `version/file` branch of the renamed [logseq/og](https://github.com/logseq/og) repository in the local checkout. Both declare mldoc 1.5.7.

mldoc and the Logseq graph implementations remain research references with their own licenses, documented in the experiments. Copying any of their code into Gardener needs its own source and license review.

## License

lsdoc 0.5.8 declares AGPL-3.0-only and supplies the complete license text, matching Gardener's license. The [license audit](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/README.md) records its locked package graph and actual license and attribution files.

Its dependencies use MIT, MIT/Apache alternatives, Unlicense/MIT alternatives, and additional Unicode data terms. The planned permissive-license choice is MIT wherever offered; unicode-ident also requires preservation of its Unicode-3.0 terms. [The inventory](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/inventory.json) identifies each package, version, source checksum, declaration, and license-file hash. [The supplied texts](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/license-texts.txt) preserve copyright attribution and every supplied license alternative.

Treat lsdoc as an unmodified dependency at its tested revision. Preserve its AGPL text and identify its upstream repository in notices. If Gardener changes upstream code, record the modification date, changes, and attribution under the existing [licensing process](licensing.md). The project's corresponding-source materials must include the exact dependency sources and build scripts for distributed builds; AGPL sections 4–6 govern source and binary conveyance, and section 13 covers a modified program's remote network interaction. See [the AGPL text](https://www.gnu.org/licenses/agpl.en.html) for those requirements.

The full MIT and Unicode notices must accompany relevant distributed code or data. Preserve the supplied copyright files, including rand's separate attribution files. Keep all license alternatives in the generated collection so reviewers can inspect the original terms.

## Change the parser revision

The SDK manifest pins lsdoc, and [Third-party notices](../THIRD_PARTY_NOTICES.md) record it. To move to another revision:

1. Rerun the [fixture comparison](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/README.md) and the public-garden comparisons, and update this page's evidence.
2. Pin the new revision in the SDK manifest, update the [dependency guide](dependencies.md), and commit the resulting Cargo.lock.
3. Run `mise run notices:update` and inspect the package inventory and license collection. Gardener's resolved versions can differ from lsdoc's lockfile.
4. Record the upstream revision, attribution, and modification status in THIRD_PARTY_NOTICES.md, and check that the dependency's license and attribution files are preserved.
5. Run `mise run notices:check` and `mise run ci`. Follow the existing source-material process whenever distributing builds.
