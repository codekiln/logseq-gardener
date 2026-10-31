## Context

The [current Logseq OG comparison](../current-og-relationships/experiments/file-relationships/findings.md) ran Logseq's `version/file` branch at `6efedb75` and Logseq 0.9.8 on eight synthetic notes, under the default and triple-lowbar filename formats, and saved the results in `relationships.json`. Both versions agree. The comparison needs a local Logseq checkout and npm packages, so it runs only on a contributor's machine.

The SDK already answers the same questions through separate modules: `page_titles` and `journals` derive titles, `page_names` reads aliases, `PageCatalog` finds files by name, and `GraphIndex` builds Logseq's graph from a `ReferenceIndex`, including pages without a file. One existing test, `candidates_reproduce_shared_og_alias_ambiguity`, reads two of the notes straight from OpenSpec change folders.

## Goals / Non-Goals

**Goals:**

- Fail CI when an SDK change alters page identity, aliases, alias ambiguity, namespace parents, journal identity or pages without a file on the recorded notes.
- Keep CI free of Logseq source, npm installs and whole gardens.
- Make regenerating the expected results one command whose diff a reviewer can read.

**Non-Goals:**

- Compare block relationships: parents, sibling order, block references and per-block page references, which `relationships.json` also records.
- Compare the SDK with Logseq on a whole public garden in CI.

## Decisions

**Copy the notes and a derived expected file into the SDK tests.** `crates/logseq-gardener-sdk/tests/fixtures/og-relationships/` holds the eight notes and `expected.json`. Reading `relationships.json` directly from the change folder would break when the change is archived and would make the test parse Logseq's full output. The alternative of hand-writing expected values would lose the link to Logseq's own results.

**Derive the expected file with a script.** `scripts/og_relationship_fixtures.py` reads `relationships.json`, which it finds by name under `openspec/changes/`, active or archived. It keeps Logseq's results after these steps: it drops built-in pages, identified as pages without a file that no note links to; it drops pages named by a block UUID; it lists each page without a file whose name is a stored page's alias under `aliases` with the files that declare it; and it lists the remaining linked pages without a file under `fileless`. `--check` compares the copies and `expected.json` with that derivation and the recorded hashes, and `--update` rewrites them, taking each note from the committed OpenSpec fixture with the matching hash. Python is already pinned by mise and used by other project checks.

**Test through SDK APIs only.** The test loads the notes with `load_garden` and `PageCatalog::load`, derives titles with `page_titles` and `journals`, aliases with `page_aliases`, and alias matches with `PageCatalog::find`. It builds a `ReferenceIndex` and `GraphIndex` with the publisher's rule that a written name refers to a page only when exactly one page answers to it. Pages without a file are graph nodes without a page that the catalog cannot find; an ambiguous alias such as `Other Name` is also such a node, and the alias comparison covers it.

**Add `GraphIndex::namespace_parent`.** The graph index already stores each node's namespace parent but keeps it private. Exposing it lets the test, and later the CLI, read Logseq's namespace field from the SDK instead of splitting titles in the caller.

**Point the existing alias test at the copies.** `candidates_reproduce_shared_og_alias_ambiguity` keeps its narrower check but reads the copied notes, so archiving the parser-comparison changes cannot break it.

## Risks / Trade-offs

[The derivation could hide a real difference] → Each dropped category is listed in the fixture guide, and the dropped pages are visible in `relationships.json`. Block relationships remain a non-goal.

[The evidence path could change] → The script searches for the `current-og-relationships` evidence wherever it sits under `openspec/changes/` and fails when it finds none or several.

[The fixture guide links into the active change folder] → The cleanup after archiving updates links, as it does for `docs/parser-adoption.md`.

## Resolved Questions

### 1 - Should the test read `relationships.json` directly instead of a copy?

Decided by the agent without codekiln: no. The issue asks for checked-in fixtures in the SDK tests, and the change folder moves when it is archived. A copy that a CI check ties to the evidence keeps the test self-contained without letting the two drift.

### 2 - Which Logseq pages should the comparison leave out?

Decided by the agent without codekiln: built-in pages that no note links to, and pages named by a block UUID. The SDK models neither as a page, and including them would make the test compare Logseq's database setup rather than relationship rules.

### 3 - How should alias names that Logseq keeps as separate pages be compared?

Decided by the agent without codekiln: as lookups. Logseq stores `Other Name` and `Second Name` as pages without a file whose aliases point back to the declaring pages. The SDK answers the same question by returning the declaring files from `PageCatalog::find`, so the expected results list those files and the test checks every match is an alias match.

### 4 - Should block relationships be part of this change?

Decided by the agent without codekiln: no. The issue lists page identities, titles, aliases, namespaces, journal identities, ambiguity and pages without a file. The recorded block fields can be added when a garden command needs them, as the issue's completion note allows.

### 5 - Is adding `GraphIndex::namespace_parent` within scope?

Decided by the agent without codekiln: yes. Relationship logic belongs in the SDK, and the index already computes the parent. The alternative, splitting titles in the test, would check the test's own rule rather than the SDK's.

### 6 - Should the new check run in `mise run ci`?

Decided by the agent without codekiln: yes. It reads only committed files and takes under a second, and without it the copies could drift from the evidence unnoticed.

## Open Questions

None.
