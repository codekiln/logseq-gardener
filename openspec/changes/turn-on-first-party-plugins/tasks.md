# Tasks

## 1. Shared parts: the key, the registry and the budget rows (#487)

- [x] 1.1 Read `:lsg/plugins` in `garden_settings.rs` into `FirstParty` variants, with errors for a non-vector value, a non-keyword entry, an unknown name and a repeated name that name `logseq/config.edn`, the entry, the `lsg` version and the shipped names; unit-test each, an empty vector, an absent key and a nested key, and run `cargo test -p logseq-gardener-sdk garden_settings`
- [x] 1.2 Add `first_party.rs` with the `FirstParty` enum, the embedded files under `crates/logseq-gardener-sdk/plugins/<id>/`, and the manifest source and build-step forms (`File` or `FirstParty`, `Command` or `InProcess`); add a unit test that runs every embedded manifest through the `read_manifest` checks and check that `cargo package -p logseq-gardener-sdk --list` includes the plugin files
- [x] 1.3 Write turned-on plugins' files to `plugins/<id>/` and list them in `lsg-site.json` with the `lsg` version, through the manifest plugins' code; in `tests/site_plugins.rs`, test the files, the version, a manifest with a first-party id (stops before output), the recorder fixture together with a first-party plugin in two argument orders (identical output) and a garden without the key (identical to the build before the change)
- [x] 1.4 Run a first-party build step in-process beside the other build plugins and accept its `metadata` contributions; test in `tests/site_build_plugins.rs` that it runs once, starts no process, and that `SiteOptions::build_plugins` still rejects metadata
- [x] 1.5 Add the page entity-type operation beside `page_properties.rs`, with the typed property reader and page lookup; unit-test case, an alias, two types in one value, a subtype alone (no match) and a garden that hides `logseq-entity`
- [x] 1.6 Make `lsg publish` exit one with empty stdout and no output directory on a key error or an id conflict, and describe the key, both plugins and the `lsg/Ref/First-Party Plugins` page in publishing help; test both in the CLI tests and run `mise run clilint:check`
- [x] 1.7 Add the `[plugins]` rows and the `[plugins.podcast-episode]` and `[plugins.sequence-header]` tables to `scripts/parity/budgets.toml`, and a test that fails when a shipped plugin lacks a table or a key; run `mise run test`
- [x] 1.8 Add `Tools/Plugins/Show/` fixture pages to `tests/fixtures/parity-garden` (two episodes, a typed page without a recording, a page with `logseq-entity` hidden) and make `parity:budgets` build a copy with the key and with 2,000 more pages, checking the size rows, `page_overhead_gzip_kb`, `garden_growth_kb`, no plugin script before the title, no plugin file on pages without an entry and identical search files; run `mise run parity:budgets`
- [x] 1.9 Add a first-party variant to `parity:build-bench` that turns each plugin on alone in a copy of the pinned garden's `config.edn` and checks `build_ms_per_page`; run `mise run parity:build-bench --garden target/encode-garden --runs 5`
- [x] 1.10 Export the fixture garden with the key through `mise run parity:logseq-export` and check that Logseq 0.10.15 shows no configuration error and exports the same pages as without the key; record the result in `verification.md`
- [x] 1.11 Add `docs/pages/lsg___Ref___First-Party Plugins.md` (the key, each plugin, the pages it applies to, its entry and its budget rows) and link it from `lsg/How To/Add a Site Plugin`, `lsg/Ref/Plugin Host API` and `lsg/Ref/Build Plugin Protocol`; run `mise run docs:check`

## 2. podcast-episode: recording and player (#487)

- [x] 2.1 Add `podcast.rs` to find an episode's recording and title; unit-test a recording under the first heading, the older `26/09/24-Thu` shape, an audio embed outside the first heading block and a page without a recording
- [x] 2.2 Add the `podcast-episode` build step contributing `recording` and `title`, with one diagnostic for a typed page without a recording; test the fixture episode's entry, the typed page without a recording, a withheld episode and the hidden-type page
- [x] 2.3 Write the module's player: one `<audio>` in a persistent region, the title linked to the episode, playback kept across navigation and Back, and another episode taking over only when the reader starts it; add Node tests in `tests/` for the module's pure functions and run `node --test tests/*.test.cjs`
- [x] 2.4 Handle `lsg:media-seek` on episode pages by cancelling the event and seeking the player; add `plugins:browser-test` scenarios for playing across navigation, Back, a second episode, a dialogue line, no plugin request on a page without an entry and the fallback when the module fails; run `mise run plugins:browser-test`
- [ ] 2.5 Add `parity:perf --first-party` with the fixture pages from the design, measured in `lsg` and in Logseq's `gh-pages` build of the same garden commit, including the heap after playing across 20 navigations; record the rows with `--record`

## 3. podcast-episode: chapter headings (#488)

- [ ] 3.1 Read chapters from H3 to H6 headings that start with `M:SS`, `MM:SS` or `H:MM:SS` in the SDK, with anchor, seconds, title and parent; unit-test each format, a timestamp in code, a timestamp later in the heading and two headings at the same time
- [ ] 3.2 Contribute `chapters` with the 64 KB entry limit; test the fixture episode and a fixture with 120 timestamps, and check with `mise run parity:budgets` that the episode stays within `page_growth_gzip_kb`
- [ ] 3.3 Turn each chapter heading's timestamp into a seek control and show the current chapter in the player; add a browser scenario that seeks from a heading and run `mise run plugins:browser-test`

## 4. podcast-episode: timestamp-led bullets (#489)

- [ ] 4.1 Read bullets at any depth under `## Chapters` that start with a timestamp as chapters whose parent is the nearest chapter among their ancestors; unit-test a nested bullet, a mid-text timestamp, a timestamp in code and a bullet outside `## Chapters`
- [ ] 4.2 Turn each bullet's timestamp into a seek control and show the chapter trail; add a browser scenario that seeks from a bullet and checks the trail, and run `mise run plugins:browser-test`

## 5. sequence-header (#490)

- [ ] 5.1 Move page-sequence resolution from `site/sequence.rs` into the SDK's `page_sequence.rs` and draw the bottom region from it; run `cargo test -p logseq-gardener-sdk sequence` with the existing tests unchanged
- [ ] 5.2 Add the `sequence-header` build step contributing `prev` and `next` with URL and title; test both neighbours, one neighbour, a withheld, ambiguous and self target, and a page without neighbours
- [ ] 5.3 Write the module that draws the header links with `rel`, escaped titles, keyboard focus and wrapping, and updates them after each navigation; add browser scenarios for both links, one link, no links and Back, and run `mise run plugins:browser-test` and `mise run parity:budgets`

## 6. Verification

- [ ] 6.1 Build logseq-encode-garden and gitpa with the branch's `lsg` and the key added in local copies of their `config.edn`, then check every fixture page from the design in Chromium on the desktop and phone profiles; record the results in `verification.md`
- [ ] 6.2 Run `mise run parity:perf --first-party --record` on the whole encode garden and check every row against the budgets; record the figures in `verification.md`
- [ ] 6.3 Run `mise run ci`

## Workflow follow-up

- After the last apply step merges, finish the change as `lsg/How To/Finish an OpenSpec Change After Merge` describes.
- After a release, the consumer issues turn the plugins on: [logseq-encode-garden #341](https://github.com/codekiln/logseq-encode-garden/issues/341), [gitpa #159](https://github.com/codekiln/gitpa/issues/159) and [gitpa #160](https://github.com/codekiln/gitpa/issues/160).
