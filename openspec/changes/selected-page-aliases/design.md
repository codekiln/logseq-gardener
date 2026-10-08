## Context

The publisher currently resolves selected titles and outline UUIDs. The workshop summary also refers to the schedule and MCP note through aliases. [Recorded OG relationships](../current-og-relationships/experiments/file-relationships/findings.md) retain competing claims for `Other Name`.

The local Logseq source at `6efedb75588763af256bc7dfd0ed5526dc91fe7c`, the revision recorded by the comparison, extracts page aliases from the leading `alias` property. `extract.cljc` retains the last duplicate property; `text.cljs` combines comma-separated plain text and parsed page references, with quoted values preserved literally. `util.cljs` lowercases names, removes boundary slashes, and normalizes Unicode to NFC. The `aliases` spelling is parsed as linkable metadata but does not populate page aliases.

## Goals / Non-Goals

**Goals:** Resolve selected aliases to existing pages, preserve ambiguous candidate sets for SDK reuse, and demonstrate the workshop links locally.

**Non-Goals:** General garden lookup, fileless names, journal identity, and transclusion remain separate capabilities.

## Decisions

### Index visible pages after publication selection

A reusable `PageNameIndex` accepts caller-supplied page identifiers, titles and documents. Publishing inserts only selected pages that pass privacy and nesting checks. A name maps to a sorted set of page identifiers. Repeated claims from the same page remain unique; different pages remain ambiguous. Title and alias claims share the same index, so a real title does not silently defeat another page's alias. Excluded claims cannot change visible ambiguity or expose withheld names through diagnostics.

### Normalize lookup keys consistently

Titles, aliases and queries use trimmed lowercase NFC keys with one leading and trailing slash removed. Display titles, source labels, namespace selection and generated routes retain existing behavior. Normalized duplicate selected titles stop planning before output is created, matching the existing title-collision policy.

### Read leading alias syntax through the parser

Only the last case-insensitive `alias` property in the first parsed properties block supplies aliases. Plain text splits on ASCII or full-width commas. Direct page references contribute their complete names, including commas inside reference brackets. Code and other formatted spans do not contribute their inner text as aliases. A quoted value remains one literal name including its quotes. If parsing finds no names, the complete nonempty value remains a literal name, matching OG's property fallback. Nested properties and the `aliases` spelling do not supply page aliases. Blank and self aliases are omitted.

### Reuse local routes and source labels

A unique candidate resolves to the page's existing HTML route. Ambiguous references render the source label with a local ambiguity diagnostic that names only the source reference. Missing references retain the existing unavailable fallback. Index entries contain identifiers rather than copied content or asset requests.

## Risks / Trade-offs

- Alias and title collisions can make previously resolved links ambiguous → tests cover collisions and diagnostics preserve visible source labels.
- The pinned parser can differ from mldoc for advanced property syntax → the documented support is bounded to the tested syntax, with recorded OG fixtures and literal cases.
- Unicode normalization adds a dependency → inspect its cached source and run the complete repository checks.

## Migration Plan

Review this change above the weekend-trial branch. After integration into main and synchronization of the publishing, media and outline-navigation requirements, the cleanup child synchronizes this delta and archives the completed change in a separate PR. Existing CLI commands continue to work.

## Resolved Questions

### 1 - Which aliases affect publication lookup?

Only aliases declared by selected visible pages enter the publication index. Namespace selection uses logical page titles.

### 2 - How are ambiguous names handled?

All distinct visible candidates are retained. The renderer links only unique candidates, including when a title collides with an alias.

## Open Questions

None.
