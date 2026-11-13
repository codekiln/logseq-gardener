## Context

[Logseq/Entity/Ableton/Set](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Ableton___Set.md) defines how a garden writes a Live Set. The Set page embeds each top-level group and track page in Live's order, a group page embeds its tracks, and each of these pages has one heading whose block properties hold the settings: `type::`, `audio-from::`, `audio-to::`, `midi-from::`, `midi-to::`, `midi-channel::`, `monitor::`, `arm::` and `devices::` on a track, and `live-version::`, `tempo::` and `time-signature::` on the Set. The garden's `ableton:set:import` task writes these pages from a `.als` file. `GitP/A/Session/26/11/06-Fri/Set` is a real Set with 23 group and track pages.

`lsg` renders the embeds as nested outlines and shows no block property, so the published Set page names its tracks and shows none of their settings. A browser plugin receives only the page properties its manifest lists, for its own page, so it has no way to read a block property of an embedded page.

`podcast-episode` already solves the same problem for chapters: its build step reads the episode page's blocks and contributes the chapter list as the page's entry, and its browser module draws from the entry.

## Goals / Non-Goals

**Goals:**

- A reader sees a Set's tracks side by side, with each track's type, routing, MIDI channel, monitor and arm state and devices, and can open each track's page.
- The Set's pages keep the shape the entity page defines and the import task writes.
- Both sites that publish Set pages get the view by turning on one plugin.

**Non-Goals:**

- Show every block property on every published page, as Logseq's app does under a block. That is a core rendering change for all gardens and all pages.
- Show routing as Live's menus name it, such as `Ext. In 3` or the name of the track an output feeds. The pages hold the `.als` file's own target names, such as `AudioIn/External/M2` and `AudioOut/Track.63/TrackIn`, and the import task is where friendlier names would be written.
- Show clips, scenes, sends, volume, pan, crossfader assignment and track colour. The Set pages do not hold them.

## Decisions

**A first-party plugin with a build step.** `FirstParty::AbletonSet` joins `PodcastEpisode` and `SequenceHeader`. Its build step chooses pages by `logseq-entity::` type and contributes one metadata entry per Set page. The alternative of a general option that publishes chosen block properties in every page's HTML would add a new public surface to `lsg` and still leave the plugin to scrape rendered embeds for order and nesting. The alternative of moving the settings to page properties would change the page shape the entity page and the import task already share, and the plugin host passes a page only its own page properties, so the Set page would still not see its tracks' values.

**The Set's entry holds the whole track tree.** `tracks` is a list in page order. A track is an object with `title`, `url`, the settings its heading states, and, on a group page only, its own `tracks`. A group is therefore the presence of `tracks`; there is no separate flag that could disagree with it. Only a Set page gets an entry, so a track page loads no plugin code and renders as an ordinary page.

**Tracks come from page embeds, matched by type.** The build step reads `{{embed [[page]]}}` macros anywhere in the Set page, in page order, and keeps an embed when it resolves to exactly one published page typed `Logseq/Entity/Ableton/Track/Track` or `Logseq/Entity/Ableton/Track/Group`. Any other embed adds no column, so a Set page may embed a screenshot page or a note. The entity type decides whether a page is a group, not its `type::` value. A chain of the routes being read stops a page that embeds itself or its Set, and nesting stops at eight levels.

**Settings are read from the first heading's block properties.** Keys are compared as Logseq stores them, and the last value of a repeated key wins. Values stay as written, except `arm`, which becomes a boolean for `true` or `false` and is left out otherwise, and `devices`, which is split at commas into a list in chain order. A key the garden hides with `:block-hidden-properties` stays out of the entry, as `sequence-header` leaves out a hidden direction.

**No entry for a Set without tracks.** A Set page that embeds no published group or track page gets no entry and one diagnostic, as an episode without a recording does. An entry over the 64,000-byte limit of one page's metadata is dropped whole with a diagnostic; a partial track list would show a wrong Set. The page metadata of the real 23-track Set is about 5.5 KB.

**The view goes above the outline, inside `<main>`.** The module inserts a `<section>` before the page's outline. The host replaces `<main>` on navigation, so the view of the page before goes with it and the module draws again for the new page. Tracks are a flex row inside a horizontally scrolling box, so the page keeps its width on a phone. Columns are a list of list items; a group is an item that holds a nested list. Every value is set with `textContent`.

**Monitor and arm are drawn as Live draws them.** Monitor shows In, Auto and Off with the track's state marked; the two other states are hidden from assistive technology. Arm shows a filled or empty dot beside `Armed` or `Off`. A coloured top edge repeats the type, which the column also states in words.

## Risks / Trade-offs

[The entry repeats settings that the track pages hold, so a Set page's HTML grows with its tracks] → About 0.9 KB gzip for the 23-track Set; the `[plugins.ableton-set]` budget row allows 2 KB of page growth and `parity:budgets` checks it on the fixture Set.

[Routing values are Live's internal target names and read poorly] → The view shows what the page says. Friendlier names are a change to the import task, named in Non-Goals.

[A device name that contains a comma is split in two] → The import task writes `devices::` as a comma-separated list, so the page shape already has this limit.

[A reader without scripts sees no track view] → The outline with the embedded pages is unchanged, and the settings are missing there as they are today.

## Resolved Questions

### 1 - Where do the track settings come from, given that `lsg` publishes no block property?

Decided by the agent without codekiln: the plugin's build step reads them from the garden's source and contributes them as the Set page's entry. This is the path `podcast-episode` takes for chapters, it needs no new public option, and the pages keep their shape.

### 2 - Does the plugin live in `lsg` or in gitpa?

Decided by the agent without codekiln: in `lsg`, as a first-party plugin. The logseq-encode-garden site publishes the same Set pages as gitpa, and the plugin keys off entity types, which is the rule issue #484 set for behaviour that more than one garden uses.

### 3 - Is a group marked by its entity type or by `type:: group`?

Decided by the agent without codekiln: by its entity type, `Logseq/Entity/Ableton/Track/Group`. The entity type is what makes a page embed tracks; `type::` is a setting shown to the reader.

### 4 - Do track and group pages get their own view?

Decided by the agent without codekiln: no. gitpa#148 asks that track pages render as ordinary pages, and a track page's own settings would be the only content of such a view.

### 5 - Does the view replace the outline?

Decided by the agent without codekiln: no. The outline holds the Set's prose and the embedded pages, and search and readers without scripts depend on it.

### 6 - Can the change be proposed, implemented and synced in one pull request?

Decided by the agent without codekiln: proposed and implemented in one pull request; synced and archived in a follow-up pull request tracked by [Sync and archive add-ableton-set-plugin · #628](https://github.com/codekiln/logseq-gardener/issues/628), as `lsg/How To/Finish an OpenSpec Change After Merge` describes.

## Open Questions

None.
