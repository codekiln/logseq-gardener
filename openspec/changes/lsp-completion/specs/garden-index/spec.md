## ADDED Requirements

### Requirement: Completion context from the text around the cursor

The SDK SHALL read a document's text and a UTF-8 offset and report what is being typed at the offset, using only the text of the offset's line and the lines above it. The context SHALL name its kind, the text typed so far, and the range of source that a chosen candidate replaces. The kinds are:

- a page link, when a `[[` before the offset on the same line has no `]]` between it and the offset; this includes `{{embed [[` and `#[[`;
- a block reference, when a `((` before the offset on the same line has no `))` between it and the offset; this includes `{{embed ((`;
- a tag, when a `#` at the start of the line or after a space, a tab or `(` is followed by text without a space up to the offset;
- a property value, when the line holds `key::` followed by a space before the offset;
- a property key, when the offset follows only the line's indentation and the characters of a property key, on a line where Logseq reads properties: before the page's first bullet, or beneath a bullet with only property lines between them.

When more than one kind applies, an open `[[` or `((` SHALL win, the later of the two when both are open. The replaced range SHALL start after the opening `[[`, `((` or `#`, or at the start of the key or value. It SHALL extend past the offset over the rest of the name and a closing `]]` or `))` when the closing pair follows on the same line with no bracket of its kind between. For a property key it SHALL extend over the rest of the key, the `::` and the spaces after it. The SDK SHALL report no context elsewhere.

#### Scenario: A half-typed page link

- **WHEN** the text is `- See [[Fru` and the offset is at its end
- **THEN** the context is a page link, the typed text is `Fru`, and the replaced range is `Fru`

#### Scenario: A closing pair after the cursor

- **WHEN** the text is `- See [[Fru]] today` and the offset is after `Fru`
- **THEN** the replaced range is `Fru]]`

#### Scenario: A tag

- **WHEN** the text is `- Cake #swe` and the offset is at its end
- **THEN** the context is a tag and the typed text is `swe`

#### Scenario: A block embed

- **WHEN** the text is `- {{embed ((pear))}}` and the offset is after `pear`
- **THEN** the context is a block reference, the typed text is `pear`, and the replaced range is `pear))`

#### Scenario: A property key and a property value

- **WHEN** the line beneath a bullet is `  ty` with the offset at its end, and another line is `  type:: rec` with the offset at its end
- **THEN** the contexts are a property key with the typed text `ty` and a value of the property `type` with the typed text `rec`

#### Scenario: Prose

- **WHEN** the text is `- An apple is red` and the offset is at its end
- **THEN** there is no context

### Requirement: Page name candidates

For a page link or a tag, the index SHALL offer the title of every page and journal, every alias as its page writes it, and every namespace parent of a title, including a parent that has no file. A journal's title is its date in the garden's `:journal/page-title-format`. When the typed text spells a date in a format Logseq accepts, the index SHALL also offer that date's journal title, whether or not the journal has a file. When the typed text holds a `/`, the index SHALL offer first the pages one level beneath the namespace written before the last `/`.

The index SHALL rank candidates in groups: a name equal to the typed text, then a name that starts with the typed text, then the other names. It SHALL order each group with Logseq's fuzzy page score, comparing without regard to letter case, and SHALL leave out a candidate that scores zero. An alias candidate SHALL name the page it belongs to and SHALL insert the alias.

A candidate's inserted text SHALL be the name followed by `]]` for a page link. For a tag it SHALL be the name when Logseq reads `#name` as that one tag, and `[[name]]` otherwise.

#### Scenario: A title and an alias

- **WHEN** the typed text after `[[` is `pom` in the fixture garden
- **THEN** the candidates include `Pomme`, an alias of `Fruit/Apple`, which inserts `Pomme]]`

#### Scenario: A name that starts with the typed text

- **WHEN** the typed text after `[[` is `log` and the garden has the pages `Blog`, `Log` and `Logbook`
- **THEN** the candidates are `Log`, `Logbook` and `Blog`, in that order

#### Scenario: A namespace level

- **WHEN** the typed text after `[[` is `Fruit/`
- **THEN** the leading candidates are `Fruit/Apple` and `Fruit/Pear`

#### Scenario: A journal by another date format

- **WHEN** the typed text after `[[` is `2026-01-04` and the journal title format is `MMM do, yyyy`
- **THEN** the candidates include `Jan 4th, 2026`

#### Scenario: A tag with a space

- **WHEN** the typed text after `#` is `mul` and the garden has a page `multi word`
- **THEN** the candidate `multi word` inserts `[[multi word]]`

### Requirement: Block candidates

For a block reference, the index SHALL offer each block id that a block writes, with the text of the block Logseq gives the id and the title of its page. A block SHALL match when its text holds every word of the typed text, without regard to letter case. A candidate SHALL insert the id followed by `))`.

#### Scenario: A block by its text

- **WHEN** the typed text after `((` is `pear ripens`
- **THEN** the candidate is the block `A pear ripens after picking.` on the page `Fruit/Pear`, and it inserts that block's id and `))`

### Requirement: Property candidates

For a property key, the index SHALL offer the keys written in the garden, ranked as page names are and then by how many lines write them, and SHALL leave out Logseq's card scheduling properties and the keys in `:block-hidden-properties`. A key candidate SHALL insert the key, `::` and a space.

For a property value, the index SHALL offer the values already written for that key, ranked the same way. For `tags`, `alias` and `aliases`, whose values Logseq reads as page names separated by commas, it SHALL offer page names for the name being typed after the last comma.

#### Scenario: Keys without hidden ones

- **WHEN** a garden writes `type::`, `card-last-score::` and a key listed in `:block-hidden-properties`
- **THEN** the key candidates include `type` and leave out the other two

#### Scenario: Values of a key

- **WHEN** the typed text after `type::` is `rec` and a block writes `type:: recipe`
- **THEN** the candidates include `recipe`

#### Scenario: A page name in `tags::`

- **WHEN** the typed text is `tags:: sweet, Fru`
- **THEN** the candidates are page names that match `Fru`, and the replaced range is `Fru`

### Requirement: A limit on candidates

The candidate query SHALL take a limit, return at most that many candidates in ranked order, and say whether it left any out. A list of page names SHALL always be reported as partial, because a date candidate appears only once the typed text spells a date.

#### Scenario: More matches than the limit

- **WHEN** 150 pages match the typed text and the limit is 100
- **THEN** the query returns the 100 best and reports that the list is partial
