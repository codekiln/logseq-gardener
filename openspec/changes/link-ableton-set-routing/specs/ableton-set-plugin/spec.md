## MODIFIED Requirements

### Requirement: Track settings from block properties

Each item of `tracks` SHALL hold `title`, the text of the track page's first heading, and `url`, the page's published path relative to the site root. It SHALL hold each of `type`, `midi-channel` and `monitor` that the heading states as a block property, as written; each of the routing settings `audio-from`, `audio-to`, `midi-from` and `midi-to` that the heading states; `arm` as `true` or `false` when the heading states `arm:: true` or `arm:: false`; and `devices` as the list of names in the heading's `devices::` property, split at commas, in order. A routing setting whose value starts with a reference to exactly one published page SHALL be held as that page's `title`, the text a column shows for it, its `url` and, when text follows the reference, that text as `detail`. A routing setting whose value starts with a reference to any other page SHALL be held as text: the last segment of the written name, then the text after the reference. Any other routing value SHALL be held as written. A setting the heading does not state, and a setting the garden hides with `:block-hidden-properties`, SHALL be left out.

#### Scenario: Audio track with devices

- **WHEN** a track page's heading states `type:: audio`, `audio-from:: AudioIn/External/S0`, `audio-to:: AudioOut/Master`, `monitor:: off`, `arm:: false` and `devices:: Compressor, EQ Eight, Limiter`
- **THEN** its item holds those values, `arm` is `false`, and `devices` is `Compressor`, `EQ Eight`, `Limiter` in that order

#### Scenario: Hidden setting

- **WHEN** the garden's `config.edn` holds `:block-hidden-properties #{:midi-to}`
- **THEN** no item holds `midi-to`

#### Scenario: Track page without a heading

- **WHEN** an embedded track page has no heading
- **THEN** its item holds `title`, the last segment of its page title, and `url`, and no setting

#### Scenario: Routing to another track

- **WHEN** a track page's heading states `audio-to:: [[Live/Set/Track/Synths]] Track In` and `Live/Set/Track/Synths` is a published page whose heading is `Synths`
- **THEN** its item's `audio-to` holds the title `Synths`, the URL of `Live/Set/Track/Synths` and the detail `Track In`

#### Scenario: Routing to a withheld page

- **WHEN** a track page's heading states `midi-from:: [[Live/Set/Track/Private]] Post Mixer` and `Live/Set/Track/Private` is marked `public:: false`
- **THEN** its item's `midi-from` is the text `Private Post Mixer`, and no generated file holds the withheld page's heading text or URL

### Requirement: Track view above the outline

On a page with an entry, the module SHALL show a track view above the page's outline, with the accessible name `Track view of <title>`. The view SHALL show one column per track in entry order, with a group's tracks as columns inside the group's column. Each column SHALL show the track's name as a link to its page, its type, the routing, MIDI channel and monitor state its item holds, a routing that names a page as that page's title linked to it and followed by its detail, `Armed` or `Off` for `arm`, and its devices in order. Every value SHALL appear as text, never as markup. The page's outline SHALL stay below the view, the same as without the plugin.

#### Scenario: Columns of the fixture Set

- **WHEN** a reader opens the Set page whose entry lists the group `Synths` holding `MicroFreak MIDI`, then `Commentary`
- **THEN** the view shows a `Synths` column that contains a `MicroFreak MIDI` column, and beside it a `Commentary` column, each name linking to its track page

#### Scenario: Device name with markup

- **WHEN** a track's `devices::` property is `Compressor, EQ <i>Eight</i>, Limiter`
- **THEN** the column lists `EQ <i>Eight</i>` as text and the view contains no `i` element

#### Scenario: Outline kept

- **WHEN** a reader opens a Set page with the plugin on
- **THEN** the outline below the view holds the same embedded group and track pages as the page built without the plugin

#### Scenario: Linked routing

- **WHEN** a reader opens a Set page on which `Commentary` routes its audio to the track `Synths` with the detail `Track In`
- **THEN** the `Commentary` column shows `Synths Track In` under Audio to, with `Synths` linking to that track's page
