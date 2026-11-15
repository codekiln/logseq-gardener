## ADDED Requirements

### Requirement: What a rename at a position is about

`GardenIndex::rename_target_at(path, offset)` SHALL return the subject of a rename, the source range of the name and the current name. On a page link, tag, page embed or page name in a property value the subject SHALL be the page the name leads to, following an alias to its page, the range SHALL be the written name, and the name SHALL be the page's title, or the name as written for a page with no file. Anywhere else in a document the subject SHALL be the document's own page, with the range of its leading `title::` value when the offset is in it and an empty range at the offset otherwise. It SHALL return a refusal for a path the index holds no document for, a document with no title, a block reference, a block id, a property key, a name several files answer to and a journal date with no file.

#### Scenario: An alias leads to its page

- **WHEN** `pages/Fruit___Apple.md` declares `alias:: Malus` and a caller asks at `[[malus]]` in another page
- **THEN** the subject is the page of `pages/Fruit___Apple.md`, the range holds `malus` and the name is `Fruit/Apple`

#### Scenario: Ordinary text names the document's own page

- **WHEN** a caller asks at an offset inside ordinary words of a page titled `Shown`
- **THEN** the subject is that page, the range is empty at the offset and the name is `Shown`

#### Scenario: A block reference

- **WHEN** a caller asks at a `((uuid))`
- **THEN** the answer is a refusal

### Requirement: Plan the rename of a page

`GardenIndex::rename_page(page, new_name)` SHALL return a `RenamePlan` and SHALL change no file. The plan SHALL list, per file in path order, the text replacements with their source ranges in source order and the file renames to apply after them. It SHALL use no Language Server Protocol type.

The plan SHALL replace every page link, tag, page embed and page name in a property value that names the page by its title, in any letter case and Unicode normalization form, with the new name as typed and trimmed. It SHALL keep a name written with one of the page's aliases and a journal date written in a format other than the journal's title. It SHALL replace `#old` with `#new` when `#new` reads back as one tag of that name and with `#[[new]]` otherwise, keep the brackets of `#[[old]]`, and write `[[new]]` for a bare name in a property value when the new name holds a comma. Text outside the replaced ranges SHALL stay as written.

#### Scenario: Every way of naming the page

- **WHEN** a page `Fruit` with `alias:: Produce` is renamed to `Sweet Food` and another page writes `[[Fruit]]`, `[[fruit]]`, `#Fruit`, `#[[FRUIT]]`, `{{embed [[Fruit]]}}`, `[label]([[Fruit]])`, `tags:: fruit`, `[[Produce]]`, `[[Fruity]]` and `[[Fruit]]` inside inline code
- **THEN** the first seven become `[[Sweet Food]]`, `[[Sweet Food]]`, `#[[Sweet Food]]`, `#[[Sweet Food]]`, `{{embed [[Sweet Food]]}}`, `[label]([[Sweet Food]])` and `tags:: Sweet Food`, and the last three stay as written

#### Scenario: A tag with no file

- **WHEN** the name `todo`, which no file answers to, is renamed to `task`
- **THEN** `#todo` becomes `#task`, `#[[todo]]` becomes `#[[task]]`, `tags:: todo` becomes `tags:: task`, and the plan renames no file

### Requirement: Namespace children follow a rename

The plan SHALL rename every page whose title starts with the old title and a `/`, replacing that start with the new name and keeping the rest of the title, together with the names that lead to those pages and the names with that start that no file answers to. It SHALL rename a page whose title holds `[[old]]` or `[[old/x]]`, as Logseq's `rename-nested-pages` does. A page whose title only starts with the same letters, such as `workshop` beside `work`, SHALL stay.

#### Scenario: Children, their links and a child with no file

- **WHEN** `work` is renamed to `Job` in a garden with the pages `work/Log`, `work/Log/2026`, `workshop` and `sub/work___Plan.md`, and a page writes `[[Work/log]]`, `#work/Log/2026` and `[[work/unwritten]]`
- **THEN** the pages become `Job/Log`, `Job/Log/2026` and `Job/Plan`, the names become `[[Job/log]]`, `#Job/Log/2026` and `[[Job/unwritten]]`, and `workshop` stays

### Requirement: File names and titles after a rename

The plan SHALL rename each renamed page's file to the name `file_names::page_file_name` gives its new title under the garden's `:file/name-format`, in the folder the file is in. It SHALL keep a journal's file name. It SHALL write the new title to the `title::` line of the page's leading properties when the page has one, and SHALL add a `title::` line at the top of the file when the page has none and either the page is a journal or `file_names::title_needs_property` reports the new title. When the old and the new path are equal after lowercasing and Unicode NFC, the plan SHALL rename the file to a temporary name that starts with a dot, in the same folder, and from there to the new path. The plan SHALL replace the string of `:default-home {:page "…"}` in `logseq/config.edn` when it names a renamed page.

#### Scenario: A title the legacy file name cannot carry

- **WHEN** `logseq/config.edn` sets no `:file/name-format` and the page of `pages/Node.md` is renamed to `Node.js`
- **THEN** the file becomes `pages/Node.js.md` and its text starts with `title:: Node.js` and an empty line

#### Scenario: A journal

- **WHEN** the journal of `journals/2026_01_02.md`, titled `Jan 2nd, 2026`, is renamed to `Bake Day` and a page writes `[[Jan 2nd, 2026]]` and `[[2026-01-02]]`
- **THEN** the plan renames no file, the journal's text starts with `title:: Bake Day`, the first link becomes `[[Bake Day]]` and the second stays

#### Scenario: A change of letter case only

- **WHEN** the page of `pages/Foo.md` is renamed to `foo`
- **THEN** the plan renames `pages/Foo.md` to a `pages/.lsg-rename-<n>.tmp` that does not exist and then that file to `pages/foo.md`

#### Scenario: The home page

- **WHEN** `logseq/config.edn` holds `:default-home {:page "home/Start"}` and the page `Home` is renamed to `Base`
- **THEN** the plan replaces the string with `"Base/Start"`

### Requirement: Refused renames

`GardenIndex::rename_page` SHALL return a `RenameRefusal` whose text states the reason, and no plan, when the new name is empty after trimming or holds a line break; when a new title of the page or of a namespace child belongs to another page by title or alias, or spells a journal date; when a name that only references carry would become the name of a page with a file; when several files answer to the old name; when the old name is a journal date with no file; when the page's title is written outside a `title::` line; when a new title gives an empty file name; and when a file exists at a new path. A new name equal to the page's title SHALL give an empty plan. A new name that only references carry, or that is one of the page's own aliases, SHALL be accepted.

#### Scenario: The new name belongs to another page

- **WHEN** page `A` is renamed to `b` or to `bee` in a garden where `pages/B.md` declares `alias:: Bee`
- **THEN** the answer is a refusal that names `pages/B.md`

#### Scenario: An empty name

- **WHEN** a page is renamed to a name of spaces, or to `/`
- **THEN** the answer is a refusal

#### Scenario: A name only references carry

- **WHEN** page `A` is renamed to `Ghost`, which a link names and no file answers to
- **THEN** the answer is a plan
