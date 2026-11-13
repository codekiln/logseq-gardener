## Purpose

A garden that writes an Ableton Live Set as pages, a Set page that embeds one page per group and track, can turn on `ableton-set`, so a reader sees the Set's tracks side by side with each track's type, routing, monitor and arm state and devices.

## Requirements

### Requirement: Entry for an Ableton Set page

The plugin SHALL apply to a published page whose `logseq-entity::` value names `Logseq/Entity/Ableton/Set` and which embeds at least one published group or track page. The page's entry SHALL hold `title`, the text of the page's first heading, the `live-version`, `tempo` and `time-signature` block properties that heading states, and `tracks`. `tracks` SHALL list, in page order, each page the Set embeds with `{{embed [[page]]}}` that resolves to exactly one published page typed `Logseq/Entity/Ableton/Track/Track` or `Logseq/Entity/Ableton/Track/Group`. A group's item SHALL hold its own `tracks`, read from the group page in the same way, and a track's item SHALL hold no `tracks` key. A page of another type SHALL get no entry.

#### Scenario: A group, a track inside it and a track outside any group

- **WHEN** a Set page embeds the group page `Live/Set/Track/Synths`, which embeds the track page `Live/Set/Track/Synths/MicroFreak MIDI`, and then embeds the track page `Live/Set/Track/Commentary`
- **THEN** the Set's entry lists `Synths` with `MicroFreak MIDI` in its `tracks`, followed by `Commentary` without a `tracks` key

#### Scenario: Track page

- **WHEN** a reader opens `Live/Set/Track/Commentary` directly on a site with the plugin on
- **THEN** the page has no entry, the browser requests no file under `plugins/ableton-set/`, and the page shows its outline as it does without the plugin

#### Scenario: Set without tracks

- **WHEN** a Set page embeds no published group or track page
- **THEN** the page has no entry and the build reports one diagnostic that names the page

### Requirement: Track settings from block properties

Each item of `tracks` SHALL hold `title`, the text of the track page's first heading, and `url`, the page's published path relative to the site root. It SHALL hold each of `type`, `audio-from`, `audio-to`, `midi-from`, `midi-to`, `midi-channel` and `monitor` that the heading states as a block property, as written; `arm` as `true` or `false` when the heading states `arm:: true` or `arm:: false`; and `devices` as the list of names in the heading's `devices::` property, split at commas, in order. A setting the heading does not state, and a setting the garden hides with `:block-hidden-properties`, SHALL be left out.

#### Scenario: Audio track with devices

- **WHEN** a track page's heading states `type:: audio`, `audio-from:: AudioIn/External/S0`, `audio-to:: AudioOut/Master`, `monitor:: off`, `arm:: false` and `devices:: Compressor, EQ Eight, Limiter`
- **THEN** its item holds those values, `arm` is `false`, and `devices` is `Compressor`, `EQ Eight`, `Limiter` in that order

#### Scenario: Hidden setting

- **WHEN** the garden's `config.edn` holds `:block-hidden-properties #{:midi-to}`
- **THEN** no item holds `midi-to`

#### Scenario: Track page without a heading

- **WHEN** an embedded track page has no heading
- **THEN** its item holds `title`, the last segment of its page title, and `url`, and no setting

### Requirement: Only published group and track pages become columns

An embed SHALL add no item when it names a withheld, excluded, missing or ambiguous page, a page that is neither a group nor a track, the Set page itself, or a page already being read as an enclosing group. The title and URL of a withheld page SHALL reach no generated file through the plugin. A page the Set links to without embedding SHALL add no item.

#### Scenario: Withheld track

- **WHEN** a Set page embeds a track page marked `public:: false`
- **THEN** the entry has no item for it and no generated file holds its heading text

#### Scenario: Group that embeds itself

- **WHEN** a group page embeds itself
- **THEN** the group's `tracks` holds no item for the group

### Requirement: Track view above the outline

On a page with an entry, the module SHALL show a track view above the page's outline, with the accessible name `Track view of <title>`. The view SHALL show one column per track in entry order, with a group's tracks as columns inside the group's column. Each column SHALL show the track's name as a link to its page, its type, the routing, MIDI channel and monitor state its item holds, `Armed` or `Off` for `arm`, and its devices in order. Every value SHALL appear as text, never as markup. The page's outline SHALL stay below the view, the same as without the plugin.

#### Scenario: Columns of the fixture Set

- **WHEN** a reader opens the Set page whose entry lists the group `Synths` holding `MicroFreak MIDI`, then `Commentary`
- **THEN** the view shows a `Synths` column that contains a `MicroFreak MIDI` column, and beside it a `Commentary` column, each name linking to its track page

#### Scenario: Device name with markup

- **WHEN** a track's `devices::` property is `Compressor, EQ <i>Eight</i>, Limiter`
- **THEN** the column lists `EQ <i>Eight</i>` as text and the view contains no `i` element

#### Scenario: Outline kept

- **WHEN** a reader opens a Set page with the plugin on
- **THEN** the outline below the view holds the same embedded group and track pages as the page built without the plugin

### Requirement: View on narrow screens, in both themes and with the keyboard

The page SHALL keep its width at any viewport width: the row of tracks SHALL scroll sideways inside the view. The view SHALL take its colours from the site's theme in light and dark. Keyboard focus SHALL reach each track link and show a visible outline.

#### Scenario: Phone

- **WHEN** a reader opens a Set page with more tracks than fit on a 375 pixel wide screen
- **THEN** the page does not scroll sideways and the row of tracks does

#### Scenario: Dark theme

- **WHEN** the site's theme is dark
- **THEN** each column has a dark background and light text

#### Scenario: Keyboard

- **WHEN** a reader presses Tab through a Set page
- **THEN** each track link receives focus in column order and shows a focus outline

### Requirement: View follows navigation

After a same-document navigation, the page SHALL show the track view of the adopted page, or none when that page has no entry, and never two views.

#### Scenario: Open a track and come back

- **WHEN** a reader follows a column's link to its track page and then goes Back
- **THEN** the track page shows no track view, and the Set page shows one track view again
