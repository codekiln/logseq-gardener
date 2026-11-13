## Why

The `ableton-set` track view shows a routing value as plain text. [Write Live's routing names on Ableton track pages · codekiln/logseq-encode-garden#397](https://github.com/codekiln/logseq-encode-garden/issues/397) changes track pages so a routing that points at another track of the Set is a page reference, such as `audio-to:: [[GitP/A/Session/26/11/06-Fri/Set/Track/1n2]] Track In`. Shown as text, that value fills a column with brackets and a long page name, and the reader cannot follow it. [Link a track's routing to the track it names in the ableton-set view · #633](https://github.com/codekiln/logseq-gardener/issues/633) asks for the link.

## What Changes

- A routing value (`audio-from`, `audio-to`, `midi-from`, `midi-to`) that starts with a reference to one published page reaches the entry as the page's title, its URL and the text after the reference.
- The view shows such a routing as the title, linked to the page, followed by that text.
- A reference to a withheld, missing or ambiguous page reaches the entry as text: the last segment of the name, then the text after it. No URL of a withheld page reaches a generated file.
- A value that does not start with a page reference is unchanged.

## Capabilities

### New Capabilities

### Modified Capabilities

- `ableton-set-plugin`: "Track settings from block properties" and "Track view above the outline" gain the linked routing.

## Impact

`crates/logseq-gardener-sdk/src/ableton_set.rs` and `first_party.rs`; `plugins/ableton-set/ableton-set.mjs` and `ableton-set.css`; `tests/site_ableton_set.rs`, `tests/ableton-set.test.cjs` and the Chromium check in `scripts/plugins/browser.py`; the `ableton-set` section of `lsg/Ref/First-Party Plugins`. An entry's routing value may now be an object, so a module older than this change would show nothing for a linked routing; the module and the build step ship together in one `lsg`.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
