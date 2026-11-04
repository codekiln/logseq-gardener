## MODIFIED Requirements

### Requirement: Preserve page name candidates

The SDK SHALL provide reusable deterministic candidate lookup for caller-supplied page identifiers. Titles, aliases and lookup queries SHALL share trimmed lowercase NFC keys with one leading and trailing slash removed. Repeated claims from one page SHALL be deduplicated. A name that is the title of an inserted page SHALL name that page alone, although other pages list the name as an alias, because Logseq keeps one page per name. When that page holds no text, meaning its file is empty or holds one bullet with nothing after the dash, and exactly one other page lists the name as an alias, the name SHALL name that other page, as Logseq's links open the alias's source page. Pages that share a title, and pages that share an alias no page has as its title, SHALL remain distinguishable and ambiguous.

#### Scenario: Duplicate alias claims
- **WHEN** Alias Source and Chosen Name both claim Other Name
- **THEN** lookup retains both page identifiers and reports an ambiguous result

#### Scenario: Unicode and title collision
- **WHEN** a title and another page's alias have equivalent normalized spelling
- **THEN** lookup returns the titled page for either spelling

#### Scenario: Empty titled page
- **WHEN** `pages/brew.md` lists `Kegs` as an alias and `pages/Kegs.md` holds only `-`
- **THEN** lookup of `Kegs` returns `brew`

## ADDED Requirements

### Requirement: Record an alias on the page it names

The SDK SHALL give each inserted page the aliases Logseq's file parser records for it (`:block/alias`, from `extract-page-alias-and-tags`): the names of the page's own `alias::` value, and, for each page whose `alias::` value names this page, that page and the other names of its value. Aliases SHALL be lookup keys, without the page's own key.

#### Scenario: A page lists another page as its alias
- **WHEN** `pages/brew.md` holds `alias:: [[Homebrew]], [[Package Manager]], [[Bottles]]`, `pages/Homebrew.md` holds `alias:: [[Formulae]]`, and `pages/Package Manager.md` has no aliases
- **THEN** `brew` has the aliases `bottles`, `homebrew` and `package manager`; `Homebrew` has `bottles`, `brew`, `formulae` and `package manager`; and `Package Manager` has `bottles`, `brew` and `homebrew`

#### Scenario: Parser check
- **WHEN** `scripts/logseq-parser/check.py` runs on that garden
- **THEN** Logseq and the SDK agree
