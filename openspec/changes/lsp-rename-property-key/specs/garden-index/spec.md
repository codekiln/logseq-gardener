## ADDED Requirements

### Requirement: Keys with a built-in meaning

`property_names::logseq_property_key` SHALL return a property key as Logseq stores it: trimmed, in lower case, with each space and `_` replaced by `-`. `property_names::is_built_in_property` SHALL be true for a key whose stored form is in Logseq's `editable-built-in-properties` or `hidden-built-in-properties`, is one of the six card scheduling keys, or starts with `logseq.`, and false for every other key.

#### Scenario: Built-in keys in any spelling

- **WHEN** a caller asks about `id`, `Title`, `alias`, `tags`, `collapsed`, `custom_id`, `Created_At`, `card-last-score` and `logseq.order-list-type`
- **THEN** each answer is true

#### Scenario: A person's own keys

- **WHEN** a caller asks about `type`, `status`, `due-date` and `identity`
- **THEN** each answer is false

### Requirement: Plan the rename of a property key

`GardenIndex::rename_property(key, new_key)` SHALL return a `RenamePlan` and SHALL change no file. The plan SHALL hold, per file in path order, one replacement for each `key::` line the index records whose key equals `key` as `logseq_property_key` compares them, in a page's leading properties and in block properties, with the range of the key and the new key as typed. The new key SHALL be trimmed of surrounding white space and of one `::` at its end. A line that already spells the new key SHALL get no replacement. The plan SHALL rename no file and SHALL leave values and all other text as written. A key no line writes SHALL give an empty plan.

When blocks other than the ones that write the old key already write the new key, the rename SHALL be planned as for an unused key.

#### Scenario: Page properties and block properties in several files

- **WHEN** `pages/A.md` starts with `Type:: book`, a block of `pages/B.md` writes `type:: film` after a tab, a journal block writes `TYPE:: note`, and `type` is renamed to `kind`
- **THEN** the plan replaces `Type`, `type` and `TYPE` with `kind` in the three files, and each file's text is otherwise as written

#### Scenario: Underscores count as hyphens

- **WHEN** lines write `due_date::`, `Due-Date::` and `duedate::`, and `due-date` is renamed to `deadline`
- **THEN** the first two keys become `deadline` and `duedate::` stays

#### Scenario: A change of letter case only

- **WHEN** lines write `Type::` and `type::`, and `type` is renamed to `Type`
- **THEN** the plan replaces the key of the `type::` line alone

#### Scenario: The new key is in use on other blocks

- **WHEN** one block writes `type:: book`, another writes `kind:: film`, and `type` is renamed to `kind`
- **THEN** the plan replaces `type` with `kind` and leaves the other block as written

#### Scenario: The key is named in a value and in text

- **WHEN** a block writes `type:: type` and its text holds `type:: in code` inside inline code, and `type` is renamed to `kind`
- **THEN** the line becomes `kind:: type` and the block's text stays

### Requirement: Refused renames of a property key

`GardenIndex::rename_property` SHALL return a `RenameRefusal` whose text states the reason, and no plan, when `is_built_in_property` is true for the old or the new key; when the new key is empty after trimming, holds white space or a colon, or is rejected by `property_names::valid_property_name`; and when a block writes both the old and the new key, in which case the refusal SHALL name the file and the line of the first such block.

#### Scenario: A built-in key

- **WHEN** `collapsed` is renamed to `folded`, or `type` is renamed to `Tags` or to `custom_id`
- **THEN** each answer is a refusal that names the built-in key

#### Scenario: A key Logseq would not read

- **WHEN** `type` is renamed to an empty name, to `my key`, to `a:b`, to `#kind` or to `kind(x)`
- **THEN** each answer is a refusal

#### Scenario: A block writes both keys

- **WHEN** a block that starts on line 3 of `pages/A.md` writes `type:: book` and `kind:: novel`, and `type` is renamed to `kind`
- **THEN** the answer is a refusal that names `pages/A.md` and line 3

## MODIFIED Requirements

### Requirement: What a rename at a position is about

`GardenIndex::rename_target_at(path, offset)` SHALL return the subject of a rename, the source range of the name and the current name. On a page link, tag, page embed or page name in a property value the subject SHALL be the page the name leads to, following an alias to its page, the range SHALL be the written name, and the name SHALL be the page's title, or the name as written for a page with no file. On the key of a `key::` line the subject SHALL be the property key in lower case, the range SHALL be the key and the name SHALL be the key as written. Anywhere else in a document the subject SHALL be the document's own page, with the range of its leading `title::` value when the offset is in it and an empty range at the offset otherwise. It SHALL return a refusal for a path the index holds no document for, a document with no title, a block reference, a block id, a property key with a built-in meaning, a name several files answer to and a journal date with no file.

#### Scenario: An alias leads to its page

- **WHEN** `pages/Fruit___Apple.md` declares `alias:: Malus` and a caller asks at `[[malus]]` in another page
- **THEN** the subject is the page of `pages/Fruit___Apple.md`, the range holds `malus` and the name is `Fruit/Apple`

#### Scenario: Ordinary text names the document's own page

- **WHEN** a caller asks at an offset inside ordinary words of a page titled `Shown`
- **THEN** the subject is that page, the range is empty at the offset and the name is `Shown`

#### Scenario: A block reference

- **WHEN** a caller asks at a `((uuid))`
- **THEN** the answer is a refusal

#### Scenario: A property key

- **WHEN** a caller asks at an offset inside `Type` of a line `Type:: book`
- **THEN** the subject is the property key `type`, the range holds `Type` and the name is `Type`

#### Scenario: A built-in property key

- **WHEN** a caller asks at an offset inside `tags` of a line `tags:: fruit`
- **THEN** the answer is a refusal that names `tags`
