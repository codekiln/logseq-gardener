## Why

A garden can describe an Ableton Live Set as pages: a Set page that embeds one page per group and track, each with its type, routing, monitor and arm state and devices as block properties ([Logseq/Entity/Ableton/Set](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Ableton___Set.md)). Published, such a Set is a long outline of embeds, and `lsg` shows no block property, so a reader sees none of the settings. [Add the ableton-set plugin · #627](https://github.com/codekiln/logseq-gardener/issues/627) asks for the Set laid out as Live shows it, for [Render Ableton Set pages as a track view · codekiln/gitpa#148](https://github.com/codekiln/gitpa/issues/148).

Two sites publish these pages, the Ghost in the Patch site and the logseq-encode-garden site, so the behaviour belongs in a first-party plugin that both turn on, as [Render the GitP/A namespace the same on both sites · #484](https://github.com/codekiln/logseq-gardener/issues/484) decided for the episode player.

## What Changes

- `lsg` ships a third first-party plugin, `ableton-set`. A garden turns it on with `:lsg/plugins [:ableton-set]`.
- At build time the plugin gives each published Set page an entry that lists the group and track pages the Set embeds, in page order, with the settings each page's heading states as block properties.
- In the browser the plugin draws a track view above the Set page's outline: one column per track, a group's tracks inside the group's column, and on each column the track's type, routing, MIDI channel, monitor and arm state and devices, with a link to the track's page.
- The outline, with the embedded group and track pages, stays on the page as it is without the plugin.
- A garden without the key builds the same site as before.

## Capabilities

### New Capabilities

- `ableton-set-plugin`: which pages get a track view, what the page's entry holds, and what a reader sees.

### Modified Capabilities

- `first-party-plugins`: the requirement "Plugins shipped inside lsg" names `ableton-set` among the shipped plugins, and the misspelled-name scenario of "Stop on an unknown or malformed entry" names it in the error.
- `cli-static-publishing`: the offline help scenario of "First-party plugins from the garden's configuration" lists `ableton-set`.

## Impact

- New SDK module `ableton_set` and a `FirstParty::AbletonSet` variant in `crates/logseq-gardener-sdk/src/first_party.rs`; new files in `crates/logseq-gardener-sdk/plugins/ableton-set/`.
- `scripts/parity/budgets.toml` gains a `[plugins.ableton-set]` table, and the fixture garden gains a Set with a group, a track inside it and a track outside any group.
- Tests: `crates/logseq-gardener-sdk/tests/site_ableton_set.rs`, `tests/ableton-set.test.cjs`, and a Chromium check in `scripts/plugins/browser.py`.
- `lsg publish` help and `lsg/Ref/First-Party Plugins` describe the plugin.
- No new dependency. Pages without an entry load no plugin code.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
