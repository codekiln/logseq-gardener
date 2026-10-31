## ADDED Requirements

### Requirement: Check SDK relationships against recorded Logseq results

The SDK tests SHALL include the fixture notes from the current Logseq OG relationship comparison and expected results derived from that comparison's recorded output. The expected results SHALL name the Logseq source revision, the runtime versions and each note's SHA-256 hash, and a guide beside them SHALL state what the derivation leaves out. For the default and triple-lowbar filename formats, an SDK test SHALL compare each note's page name, title, file, aliases, namespace parent and journal date; the files each alias name finds; and the pages without a file that notes link to, together with their namespace parents. `mise run ci` SHALL run the test and a check that the copied notes and expected results match the recorded evidence, without a Logseq checkout or a whole garden. An update command SHALL regenerate the copies from the recorded evidence.

#### Scenario: A relationship regression fails CI

- **WHEN** an SDK change stops reading a page's `alias::` names, keeps one page per shared alias, or loses a namespace parent
- **THEN** the relationship test fails and names the differing pages

#### Scenario: Copies drift from the evidence

- **WHEN** a copied fixture note or the expected results differ from what the recorded evidence implies
- **THEN** the check fails, names the file and points to the update command

#### Scenario: Regenerate after an intended change

- **WHEN** a contributor updates the recorded Logseq evidence and runs the update command
- **THEN** the copied notes and expected results are rewritten from the evidence for review, with no network access or Logseq checkout
