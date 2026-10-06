# Parser findings

Use lsdoc as the leading candidate for read-only Markdown parsing, with a separate SDK layer for Logseq's graph rules. The tested Markdown cases support that direction: the only real-garden syntax difference found affects math rendering inside emphasis, and its reference sets agree. Parser adoption remains a separate change; the current Logseq OG revision and Pengx's publishing garden still need coverage under [Issue #3 Compare Logseq parsers](https://github.com/codekiln/logseq-gardener/issues/3).

## Syntax

The shared baseline and focused graph fixtures produce equal normalized syntax and reference output. The comparison excludes `span`, `span_map`, and table `aligns`, matching lsdoc's upstream comparator. Source ranges are checked separately.

[Encode Garden's report](encode-report.md) finds a difference in dollar-delimited math inside bold text. [The synthetic reproduction](regressions.json) uses `- **$3$4/day**`: mldoc treats the bold contents as plain text; lsdoc treats `$3$` as inline math. lsdoc documents this intentional behavior as D48 in its [decisions](https://github.com/martinkoutecky/lsdoc/blob/32e63ef095c711d6d9947257bf5fd07d540fa59d/DECISIONS.md). Both parsers return the same references. A renderer would need to choose whether to preserve Logseq's behavior; a reference-only read command can avoid that decision.

[The corrected Logseq docs report](docs-report.md) finds equal normalized syntax and reference output throughout the pinned corpus. The original runner reported Org links such as `[[Guide/Page][Guide page]]` as disagreements because its mldoc worker called `extractRefs(ast)` with the default Markdown format. Experiment setup now repairs the worker to pass the note format, and each generated report identifies that repair. The synthetic runner garden exercises the actual worker with Org and Markdown links; removing the repair makes the regression check fail.

The reports contain no other findings on their pinned corpora. They are syntax comparisons; they do not compare a complete SDK graph with Logseq's graph.

## Graph meaning

[The fixture snapshot](fixtures.json) records Logseq's database output and graph-validator assertions. The relationships suggest the following SDK responsibilities:

| Input | Recorded behavior | SDK responsibility |
| --- | --- | --- |
| `Names___Nested.md` | Default legacy naming retains the underscores; triple-lowbar configuration produces `Names/Nested`. | Read graph configuration before deriving file identities. |
| `title:: Chosen Name` | Overrides `Title Override.md` as the page name. | Keep file location distinct from page identity. |
| Shared `Other Name` alias | Links the alias page to both `Alias Source` and `Chosen Name`. | Preserve all alias candidates and report ambiguity when a command requires a unique target. |
| `[[Missing Target]]` | Creates a page entity without a backing file. | Distinguish known names from notes stored in files. |
| Child with `id::` and `((uuid))` | UUID reference targets the nested note. | Resolve explicit UUIDs and retain parent and sibling relationships. |
| Missing UUID and embed | graph-validator fails both reference validations. | Retain unresolved references for diagnostics. |
| `public:: true/false` | Stored as page properties. | Keep publication policy separate from parsing properties. |
| Code fence | Reference-like text produces no graph references. | Respect literal syntax contexts. |

The missing sketch asset and fileless tag/page targets produce the other expected validator failures. The simple query macro is present in syntax output; the validator's advanced-query test reports no advanced queries. Publication filtering and query evaluation remain untested.

Logseq 0.9.8 is the relationship reference because the file-compatible graph-validator pins that version. Compatibility with the current Logseq OG parser needs a follow-up run. Generated entity UUIDs are normalized out of the saved graph output; explicit reference UUIDs remain visible.

## Source preservation

mldoc's Markdown exporter changes every tested input, including indentation, line endings, and trailing whitespace. Its output is unsuitable for an unchanged-note write contract under the tested settings.

lsdoc exposes source-oriented outline ranges. The probe slices those ranges and reassembles the retained original text unchanged for the fixture and line-ending cases. lsdoc supplies no Markdown serializer in its tested public API. The evidence supports storing source text alongside parsed structure and later making edits through source ranges. It does not establish formatting-preserving edits or validate every AST span.

## Performance

The public-garden reports measure parser-reported parse time, excluding process startup, with the best total from repeated runs. lsdoc is the native Rust release build; mldoc is the npm JavaScript build used by Logseq. These measurements favor lsdoc for parsing speed on both gardens. They do not measure SDK indexing, graph construction, or command latency.

[The memory probe](memory.json) measures separate processes parsing the complete docs corpus. lsdoc used about 23 MiB of peak resident memory and mldoc about 267 MiB. Runtime overhead and retained syntax output contribute to both totals; a production streaming adapter could have a different profile.

## Adoption work

The Rust SDK should retain each note's original text and file information, and provide parsed note structure to callers. A separate SDK module should apply Logseq's rules for page names, titles, aliases, namespaces, journals, properties, UUID references, and names with no note file. The command-line tool should use those SDK results.

The observed Markdown difference calls for a rendering-policy decision rather than a syntax repair for a reference-only command. The Org reference runner is repaired in the local experiment. No parser repair is indicated by these runs. Broader semantic compatibility still requires comparison of the proposed SDK graph against the relationship snapshots, current Logseq OG coverage, Pengx's publishing corpus, and separate editing tests before writes.
