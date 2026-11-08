# Design

## Context

The [analysis](../../../docs/parity/gitp-a-shared-rendering.md) compared gitpa (`410f4d9`) and logseq-encode-garden (`a7d948e`) built with `lsg` 0.0.29 and assigned each reader behaviour an owner. This change covers the behaviour it assigned to first-party plugins. See proposal.md for the motivation.

What `lsg` has today:

- **Manifest plugins.** `lsg publish --plugin PATH` reads a JSON manifest. `plugins::read_manifests` checks it before the output directory exists, the publisher copies the module and stylesheets to `plugins/<id>/`, and `lsg-site.json` lists the plugin ([Plugin Host API](../../../docs/pages/lsg___Ref___Plugin%20Host%20API.md)).
- **The browser host.** With one plugin that has a module, every page except the graph view carries a `data-lsg-page` metadata element and imports `lsg-host-<digest>.mjs` after first paint. The host owns same-document navigation, keeps persistent regions outside `<main>`, and imports an `on-demand` module on the first page whose metadata has an entry for it.
- **Build plugins.** A manifest's `build` section runs a program once per build, and a Rust caller can add an in-process `BuildPlugin` to `SiteOptions::build_plugins` ([Build Plugin Protocol](../../../docs/pages/lsg___Ref___Build%20Plugin%20Protocol.md)). Both return `rows` and `metadata` contributions, which `contributions::resolve` checks. Metadata needs a module, and an in-process plugin has none, so in-process plugins contribute rows only.
- **Garden settings.** `garden_settings.rs` and `garden_config.rs` read top-level keys of `logseq/config.edn` with an EDN reader, including `lsg`'s own `:publishing/page-sequence?`.
- **Page sequences.** `site/sequence.rs` resolves `prev::` and `next::` to published neighbours and draws the Page sequence region at the bottom of a page.
- **Typed property values.** The advanced queries change added a reader that turns `[[A]], [[B]]` into a set of page names.

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) asks that each page load on its own, that search stay incremental, that phones stay light, and that a harness prove it. Its shared-engine comment asks that graph answers come from the SDK, so the CLI, the language server and the publisher share them.

## Goals / Non-Goals

**Goals:**

- Both sites turn on `podcast-episode` and `sequence-header` with the same line in `logseq/config.edn` and get the same behaviour on the fixture pages below.
- Gitpa can delete `site/player.js`, `site/episode-player.js`, `site/plugin-adapter.js`, `site/navigation.js`, `site/header-sequence.js`, `site/header-sequence.css` and the heading rewrite in `scripts/audio_chapters.py`.
- A garden without the key builds the same files as before.
- Each plugin meets its own budget rows, measured on the whole encode garden against Logseq's build.

**Non-Goals:**

- Per-plugin options in `config.edn`, such as a different chapters heading or player position.
- Pairing an in-process build plugin with a browser module for Rust callers of `SiteOptions::build_plugins`.

## Decisions

### The key

```clojure
:lsg/plugins [:podcast-episode :sequence-header]
```

The key is a top-level entry of `logseq/config.edn`, read with the other garden settings. Its value is a vector of keywords; each keyword is a first-party plugin id. An absent key and an empty vector turn on nothing. A value that is not a vector, an entry that is not a keyword, an unknown name and a repeated name each stop the build before the output directory exists. The error names `logseq/config.edn`, the entry, the `lsg` version and the plugins this `lsg` ships:

```text
logseq/config.edn: :lsg/plugins names :podcast-epsiode, which lsg 0.0.40 does not ship. It ships :podcast-episode and :sequence-header.
```

The key is the only switch. `lsg publish` has no flag that turns a first-party plugin on or off, and the SDK reads the key while it reads the garden, so a Rust caller gets the same plugins as the CLI.

### A closed list of plugins

The SDK names each first-party plugin as one variant of an enum, `FirstParty::{PodcastEpisode, SequenceHeader}`. Reading the key turns each keyword into a variant or an error, so no later code can hold an unknown plugin name. Each variant gives its manifest, its embedded files and its build step.

### Files inside the binary

Each plugin's source is a folder in the SDK crate:

```text
crates/logseq-gardener-sdk/plugins/podcast-episode/
  lsg-plugin.json
  podcast-episode.mjs
  podcast-episode.css
crates/logseq-gardener-sdk/plugins/sequence-header/
  lsg-plugin.json
  sequence-header.mjs
  sequence-header.css
```

The files are compiled into the SDK with `include_bytes!`, so a crate release carries them. The manifest has `id`, `apiVersion: 1`, `module`, `styles` and `activation: "on-demand"`, and no `pageProperties` and no `build` section. Its `version` is the `lsg` version. A unit test runs every embedded manifest through the checks `read_manifest` runs on a file, so a broken manifest fails `mise run test`.

When a garden turns a plugin on, the publisher writes its module and stylesheets to `plugins/<id>/<file>` and lists it in `lsg-site.json`, through the same code that writes a manifest plugin's files. Nothing is read from the garden or the network for the plugin.

### A build step paired with a browser module

A checked manifest gets a source and a build step that can each take one of two forms:

- source: `File(path)` for `--plugin`, or `FirstParty(variant)`;
- build step: `Command(BuildCommand)` for a manifest's `build` section, or `InProcess(Arc<dyn BuildPlugin>)` for a first-party plugin.

A first-party build step is an ordinary `BuildPlugin`. It runs once per build on its own thread while the publisher builds the indexes, receives the same view as other build plugins, and returns contributions that `contributions::resolve` checks. Because the manifest it belongs to has a module, its `metadata` contributions are accepted. `SiteOptions::build_plugins` keeps its current behaviour.

All plugins share one id namespace. A `--plugin` manifest or a Rust plugin with the id of a turned-on first-party plugin stops the build, naming both. Plugins are ordered by id, as today.

### How a plugin chooses its pages

A plugin applies to a page by its authored page properties. The SDK gains an entity-type operation: the set of pages that a page's `logseq-entity::` value names, read with the typed property reader and matched through page lookup, so case and aliases match as in Logseq.

- `podcast-episode` applies to a page whose types include `Logseq/Entity/Podcast/Episode`.
- `sequence-header` applies to a page with at least one valid neighbour under the authored page sequence rules.

The operation reads the authored value even when the garden hides `logseq-entity` with `:block-hidden-properties`; the hidden value never reaches a generated file. Only published pages are in the view, so a withheld or excluded page gets nothing. No code names `GitP/A` or any other garden namespace.

### What each plugin's entry holds

The browser receives only what the build step contributes, as metadata under the plugin's id:

| Plugin | Entry | Taken from |
| --- | --- | --- |
| `podcast-episode` | `recording` (URL), `title`, `chapters`: `[{anchor, seconds, title, parent}]` | the first audio embed among the descendants of the first heading block; the first heading's text; H3 to H6 headings and bullets under `## Chapters` that start with a timestamp |
| `sequence-header` | `prev` and `next`, each `{url, title}` | the authored page sequence |

A page the plugin does not apply to gets no entry, so its `on-demand` module is not imported there. A page of the episode type without a recording gets no entry and one build diagnostic.

### Where each part lives

The SDK holds every relationship, so `lsg` commands and the language server can call the same operations:

- reading `:lsg/plugins` (`garden_settings.rs`) and the `FirstParty` registry (`first_party.rs`);
- page entity types (beside `page_properties.rs`);
- an episode's recording and chapters (`podcast.rs`);
- page-sequence neighbours (`page_sequence.rs`, moved out of `site/sequence.rs`, which keeps drawing the bottom region from it);
- each plugin's build step, which calls these operations and returns contributions.

The publisher holds file copying, the metadata element and `lsg-site.json`, all shared with manifest plugins; it gains no new rendering. Each browser module draws only what its entry says: the player, the seek controls, the current-chapter trail and the header links.

### How each site turns the plugins on

Both sites add one line to their garden's `logseq/config.edn` and move their `lsg` pin to the first release that ships both plugins:

| Site | File | Also |
| --- | --- | --- |
| logseq-encode-garden ([#341](https://github.com/codekiln/logseq-encode-garden/issues/341)) | `logseq/config.edn` | Episode pages follow one shape first ([#339](https://github.com/codekiln/logseq-encode-garden/issues/339)), and `GitP/A` takes the show type `Logseq/Entity/Podcast`. |
| gitpa ([#159](https://github.com/codekiln/gitpa/issues/159), [#160](https://github.com/codekiln/gitpa/issues/160)) | `gitp-garden/logseq/config.edn` | `scripts/build_site.py` stops rewriting headings and adding its own scripts; gitpa deletes the files listed under Goals. `prepare_site.py` keeps choosing public pages before `lsg` runs. |

### Fixture pages

Both sites are compared on these pages, served locally, on the desktop and phone profiles:

| Page | Checks |
| --- | --- |
| `GitP/A/Session/26/11/01-Sun` | heading `18:17 Enter Microfreak/Preset/260 8bitfi` and bullet `05:15 - the bass comes in` seek; only a Previous link |
| `GitP/A/Session/26/10/29-Thu` | the hand-annotated chapter's bullets seek |
| `GitP/A/Session/26/10/24-Sat` | dialogue lines seek the player; Previous and Next links; the recording keeps playing to `GitP/A` and the asset page |
| `GitP/A/Session/26/10/20-Tue` | about 120 timestamps: the entry stays under 64 KB and the page within the first-paint budgets |
| `GitP/A/Session/26/09/24-Thu` | the older shape: the recording is found from the base type alone |
| `GitP/A`, `GitP/A/Session` | no plugin file requested when opened directly; the player stays when reached from an episode |
| `GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3` | an asset page reached while playing |
| gitpa's `Ghost in the Patch` and the encode site's journals | homepages: no plugin file requested when opened directly |

The repository's own fixture garden gets pages under `Tools/Plugins/Show/` with their own names: an episode with a recording, chapter headings, timestamp-led bullets, dialogue lines and both neighbours; a second episode; a page with the type and no recording; and a page whose `logseq-entity` is hidden. CI uses these; the GitP/A pages are checked in the Verify step.

## Risks / Trade-offs

- With any browser plugin on, the host takes over navigation on every page of the garden, 9,153 pages on the encode site → the host already meets the `[plugins]` navigation and listener budgets on the whole encode garden, `plugins:browser-test` covers its fallbacks, and removing the key restores the plain site.
- An `lsg` older than the release ignores `:lsg/plugins`, so a site that keeps an old pin builds without the plugins and without an error → each consumer issue moves the pin in the same commit that adds the key.
- Episode metadata sits in the page head and counts toward first paint → the `page_growth_gzip_kb` row caps it, measured on the largest episode.
- Every bullet under `## Chapters` that starts with a timestamp becomes a chapter, including spoken-word lines such as `18:17 — Let's try this 8bitfi lead, 260.` → this is what [gitpa #139](https://github.com/codekiln/gitpa/issues/139) asks; the trail shows the enclosing chapters, so the reader still sees the heading.
- A plugin fix reaches a site only through an `lsg` release and a pin move → both sites get the same fix at the same version.
- A garden that hides `logseq-entity` still gets the player on its episodes, which tells a reader the page is an episode → the recording on the page already tells the reader that.

## Migration Plan

1. [#487](https://github.com/codekiln/logseq-gardener/issues/487): the key, the registry, embedded files, the paired build step, the budget rows and `podcast-episode`'s player and dialogue seeking.
2. [#488](https://github.com/codekiln/logseq-gardener/issues/488): chapter headings.
3. [#489](https://github.com/codekiln/logseq-gardener/issues/489): timestamp-led bullets.
4. [#490](https://github.com/codekiln/logseq-gardener/issues/490): `sequence-header`.
5. A release, then the consumers as the table under How each site turns the plugins on shows.

To roll back, a site removes the `:lsg/plugins` line; its next build matches the build without plugins.

## Performance budgets

Nothing runs before first paint. A page without a plugin entry pays the metadata element and, after first paint, the host. A page with an entry also pays the plugin's module and stylesheet. New rows in `scripts/parity/budgets.toml`:

```toml
[plugins]
host_gzip_kb = 4
added_bytes_per_page = 0
navigation_ms_phone = 50
listener_growth = 0
page_overhead_gzip_kb = 0.5     # a page without a plugin entry, both plugins on, against no key
garden_growth_kb = 0            # plugin-added size of a page when the garden gains 2,000 pages

[plugins.podcast-episode]
script_styles_gzip_kb = 8       # module and stylesheet; gitpa's player code is about 7 KB gzip
page_growth_gzip_kb = 4         # the largest fixture episode against the build without the key
phone_heap_growth_mb = 1        # after playing across 20 navigations
build_ms_per_page = 0.05        # added build time per published page, encode garden

[plugins.sequence-header]
script_styles_gzip_kb = 1.5
page_growth_gzip_kb = 0.5
phone_heap_growth_mb = 0.2
build_ms_per_page = 0.05
```

The existing budgets apply to every page with the plugins on: 100 KB of data and 16 KB of script before the title, 300 ms (desktop) and 1,500 ms (phone) to the title, 10 MB of JavaScript heap, and a search p95 of 50 ms (desktop) and 100 ms (phone) per keystroke. Logseq's build of the same encode garden loads about 22,000 KB of data and 8,000 KB of script before the title and peaks near 2,000 MB of heap on the same pages ([Performance](../../../docs/pages/lsg___Ref___Logseq%20Compatibility___Performance.md)).

### Measurement

- **In CI, on the fixture garden.** `parity:budgets` builds the fixture garden without the key, with both plugins on, and with both on and 2,000 more pages. It checks each `[plugins.<id>]` size row, `page_overhead_gzip_kb` on `Tools/Queries`, `garden_growth_kb`, that no page loads a plugin script before the title, that pages without an entry link no plugin file, and that the search files match the build without the key.
- **In CI, on the encode garden.** `parity:build-bench` builds its pinned encode garden commit with each plugin turned on alone in a copy of `logseq/config.edn` and fails when the added time per published page exceeds `build_ms_per_page`.
- **Recorded, on the whole encode garden against Logseq.** `parity:perf --first-party` builds the current encode garden with both plugins on and measures each fixture page in that build and in Logseq's `gh-pages` build of the same commit: time to the title, data and script before the title, peak heap on the phone profile, search keystroke latency, and the heap after playing `26/10/24-Sat` across 20 navigations. `--record` writes the rows to the Performance page.
- **A test.** `mise run test` fails when a shipped plugin has no `[plugins.<id>]` table or a table lacks one of the four keys.

## Resolved Questions

### 1 - How is the key spelled?

Decided by the agent without codekiln: `:lsg/plugins` with a vector of keywords, `:lsg/plugins [:podcast-episode :sequence-header]`. The `lsg/` namespace marks a key only `lsg` reads, and Logseq ignores top-level keys it does not know. Keywords match how `config.edn` names things elsewhere, such as `:block-hidden-properties #{:public}`, and the vector form is the one the analysis and the consumer issues already use. Order has no effect, because plugins run in id order.

### 2 - What does a malformed value do?

Decided by the agent without codekiln: a value that is not a vector, a string or other non-keyword entry, and a repeated name each stop the build before output with a message naming `logseq/config.edn` and the entry. An empty vector is the same as no key. A silently skipped entry would publish a site without the behaviour its author asked for, and the author would find out only by reading the site.

### 3 - What does an unknown name do?

Decided by the agent without codekiln: the build stops before the output directory exists, and the message names the entry, the `lsg` version and the plugins it ships. This is the analysis's starting point. Naming the version helps an author whose config names a plugin from a newer `lsg` than the pinned one.

### 4 - Can the command line turn a first-party plugin on or off?

Decided by the agent without codekiln: no. `config.edn` is the only switch, the SDK reads it while it reads the garden, and `--plugin PATH` keeps adding manifest plugins. One declarative place means both sites, the CLI and any Rust caller build the same site from the same garden, and `lsg publish` gains no flag.

### 5 - How are a plugin's files embedded and written?

Decided by the agent without codekiln: each plugin's manifest, module and stylesheet live in `crates/logseq-gardener-sdk/plugins/<id>/`, are compiled in with `include_bytes!`, and are written to `plugins/<id>/<file>` through the code that writes manifest plugins' files. The folder sits inside the SDK crate so a crate release carries the files. A unit test runs the embedded manifests through the same checks as manifest files. The output paths match manifest plugins, so the host, `lsg-site.json` and the budget checks treat both the same.

### 6 - What version does a first-party plugin report?

Decided by the agent without codekiln: the `lsg` version. The plugin ships and is tested with that release, so a second version number would carry no information.

### 7 - How does a first-party plugin pair a build step with a browser module?

Decided by the agent without codekiln: a checked manifest holds a build step that is either a command or an in-process `BuildPlugin`, and the registry gives each first-party plugin an in-process one beside its module. Its metadata contributions are accepted because its manifest has a module, and they pass the same checks as any contribution. A process would need a program on the build machine, and the plugin's logic would leave the SDK. `SiteOptions::build_plugins` stays as it is; opening it to modules is listed under Non-Goals.

### 8 - What happens when a manifest plugin uses a first-party id?

Decided by the agent without codekiln: when both are in one build, the build stops and names the id, `logseq/config.edn` and the manifest. All plugins share one id namespace, as manifests and Rust plugins already do. A garden that leaves the first-party plugin off can still use a manifest with that id.

### 9 - How does a plugin choose its pages?

Decided by the agent without codekiln: `podcast-episode` applies to pages whose `logseq-entity::` names `Logseq/Entity/Podcast/Episode`, and `sequence-header` to pages with a valid `prev::` or `next::` neighbour. The match is an exact page match through page lookup, so case and aliases follow Logseq; a subtype such as `Logseq/Entity/Podcast/Episode/GitP/A` alone does not match. Every GitP/A episode already names the base type, and an exact match keeps the rule simple to state and test.

### 10 - Does hiding `logseq-entity` turn the player off?

Decided by the agent without codekiln: no. The SDK reads the authored property to choose pages, and only values from published content reach the output. Logseq's `:block-hidden-properties` hides a property from readers and leaves it in queries, so a garden that tidies its property rows should keep its player.

### 11 - Which pages load plugin code?

Decided by the agent without codekiln: both plugins use `on-demand` activation and add an entry only to pages they apply to, so a page without an entry imports neither module. The host itself loads after first paint on every page once a browser plugin is on, because it owns navigation and keeps the player alive across pages. That is the main cost for the encode site, and `page_overhead_gzip_kb` and the `[plugins]` rows measure it.

### 12 - Where does the recording come from?

Decided by the agent without codekiln: the first audio embed among the descendants of the page's first heading block, as the analysis states. A page of the type without one gets no entry and one build diagnostic, so `GitP/A`, which carries the episode type until the content issue changes it, builds without a player and tells the author why.

### 13 - Which timestamps make chapters?

Decided by the agent without codekiln: H3 to H6 headings that start with `M:SS`, `MM:SS` or `H:MM:SS` ([#488](https://github.com/codekiln/logseq-gardener/issues/488)), and bullets at any depth under `## Chapters` that start with one ([#489](https://github.com/codekiln/logseq-gardener/issues/489)). A chapter's parent is the nearest chapter among its ancestors. Timestamps later in a block or in code stay text. These are the rules the two issues give.

### 14 - Why does `sequence-header` draw its links in the browser?

Decided by the agent without codekiln: contributions have rows and metadata and no header slot, so the build step contributes the neighbours' URLs and titles and the 1.5 KB module draws them. A reader without the module still has the bottom Page sequence region that `lsg` core draws. A header contribution kind would widen the build protocol for one plugin.

### 15 - Does `sequence-header` change the bottom Page sequence region?

Decided by the agent without codekiln: no. The region and the property rows stay as `lsg` core draws them, and the header links use the same SDK resolution, so both always name the same pages. Gitpa shows a `prev::` row and the header today; the encode garden can choose its property rows in its own `config.edn`.

### 16 - What lives in the SDK and what in the publisher?

Decided by the agent without codekiln: the SDK holds the key, the registry, page entity types, an episode's recording and chapters, page-sequence neighbours and the build steps. The publisher writes files, metadata and `lsg-site.json` through the code manifest plugins already use, and draws no new HTML. Page-sequence resolution moves from `site/sequence.rs` into the SDK so the bottom region and the header plugin share one answer. This is the split the shared-engine comment on #106 asks for.

### 17 - Which budget rows must each first-party plugin meet?

Decided by the agent without codekiln: a `[plugins.<id>]` table with `script_styles_gzip_kb`, `page_growth_gzip_kb`, `phone_heap_growth_mb` and `build_ms_per_page`, with the values under Performance budgets, plus two rows in `[plugins]` for every page once any browser plugin is on. The four keys are the same for every plugin, so one check covers each new plugin, and a test fails when a plugin lacks its table. The values come from the analysis: gitpa's player code is about 7 KB gzip, and the in-process build budget is the existing `[build_plugins]` row. Gitpa's header script and stylesheet are 1.7 KB gzip together; the plugin's 1.5 KB is smaller because it reads metadata where gitpa's script reads the rendered property rows.

### 18 - How are the budgets measured, and which runs are in CI?

Decided by the agent without codekiln: `parity:budgets` on the fixture garden and `parity:build-bench` on the pinned encode garden run in CI; `parity:perf --first-party` measures the whole encode garden against Logseq's build of the same commit and records rows in the Performance page. Browser timings and phone heap need a local Chromium run, as for every earlier feature; sizes and build time can run on GitHub's runners.

### 19 - Where do the fixture pages live?

Decided by the agent without codekiln: the GitP/A pages and both homepages from #486 are checked in the Verify step on builds of the two real sites, and CI uses pages under `Tools/Plugins/Show/` in the repository's fixture garden, with their own names. The fixture garden's own `config.edn` keeps no `:lsg/plugins` key; `parity:budgets` adds it to a copy, so every other check keeps its no-plugin baseline.

### 20 - How does each site turn the plugins on?

Decided by the agent without codekiln: each adds `:lsg/plugins [:podcast-episode :sequence-header]` to its garden's `logseq/config.edn` (`logseq/config.edn` in logseq-encode-garden, `gitp-garden/logseq/config.edn` in gitpa) and moves its `lsg` pin in the same commit. Gitpa then deletes its player, navigation, header and heading-rewrite code. The consumer issues already exist in each repository.

### 21 - Which issue builds the shared parts?

Decided by the agent without codekiln: #487, because `podcast-episode`'s player is the first plugin that needs the key, the registry and the paired build step. The tasks group the work by issue so each apply step has one pull request.
