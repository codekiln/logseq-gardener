## 1. Build step

- [x] 1.1 Add the SDK module `ableton_set`: the entity types, the setting names, the first heading with its block properties, and the pages a page embeds.
- [x] 1.2 Add `FirstParty::AbletonSet` with its manifest, summary and build step, which contributes each Set page's entry.
- [x] 1.3 Report a Set without published tracks, and a Set whose entry exceeds the metadata limit, as one diagnostic each.

## 2. Browser module

- [x] 2.1 Add `plugins/ableton-set/ableton-set.mjs`: read the entry and draw the track view above the outline.
- [x] 2.2 Add `plugins/ableton-set/ableton-set.css`: columns, group columns, sideways scrolling, theme colours and the focus outline.

## 3. Tests

- [x] 3.1 `crates/logseq-gardener-sdk/tests/site_ableton_set.rs`: a fixture Set with a group, a grouped track and an ungrouped track; embeds that add no column; a hidden setting; the unchanged outline.
- [x] 3.2 `tests/ableton-set.test.cjs`: the module's reading of an entry.
- [x] 3.3 `scripts/plugins/browser.py`: the drawn view in Chromium at desktop and phone width, in light and dark, with the keyboard and across navigation.
- [x] 3.4 Add the `[plugins.ableton-set]` budget table and a fixture Set to the parity garden for `parity:budgets`.

## 4. Documentation

- [x] 4.1 Describe the plugin in `lsg/Ref/First-Party Plugins`.
- [x] 4.2 Name the plugin in `lsg publish` help and in the `:lsg/plugins` error tests.

## 5. Handoff

- [x] 5.1 Build `GitP/A/Session/26/11/06-Fri/Set` from logseq-encode-garden with the plugin on and check the view at desktop and phone width in light and dark.
- [x] 5.2 Run `mise run plugins:browser-test` and `mise run ci`, and open the implementation pull request.
