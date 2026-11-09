# Render the GitP/A namespace the same on both sites

The Ghost in the Patch episodes (`GitP/A/Session/YY/MM/DD-Ddd` pages, typed `logseq-entity:: [[Logseq/Entity/Podcast/Episode]]`) read differently on the two sites that publish them. On [codekiln.github.io/gitpa](https://codekiln.github.io/gitpa), built from [codekiln/gitpa](https://github.com/codekiln/gitpa), a reader gets a player that keeps playing across pages, chapter headings that seek the recording, and previous and next links in the header. On the [logseq-encode-garden site](https://codekiln.github.io/logseq-encode-garden), where the episodes are written, the same page has an inline audio element and plain text timestamps.

Gitpa gets its extra behaviour from about 52 KB of its own browser scripts and from Python that rewrites `lsg`'s HTML after the build. The plan moves the behaviour that more than one garden needs into first-party optional plugins that ship inside each `lsg` release. A garden turns a plugin on with one entry in its `logseq/config.edn`. Both sites then run the same plugin code at the same `lsg` version, gitpa deletes its copies, and `lsg` with no plugins keeps matching Logseq's own published site.

Tracked in [Render the GitP/A namespace the same on both sites](https://github.com/codekiln/logseq-gardener/issues?q=is%3Aissue+%22Render+the+GitP%2FA+namespace+the+same+on+both+sites%22).

## What was compared

Both sites were built on 2026-11-07 with `lsg` 0.0.29 built from logseq-gardener `a8a7a08`. Both repositories already pin release v0.0.29 in their Pages workflows, so the pinned build and the main build are the same version.

- Gitpa `410f4d9`: `scripts/build_site.py --renderer target/release/lsg --search`, as its Pages workflow runs it.
- logseq-encode-garden `a7d948e`: `mise-tasks/site/build --lsg target/release/lsg`, as its Pages workflow runs it. 9,153 pages and 519 journals.

Each output was served on 127.0.0.1 and the fixture pages below were opened in a browser and read from the generated HTML.

## Reader behaviour on each site

| Behaviour | Gitpa site | Encode site | Owner |
| --- | --- | --- | --- |
| Persistent player | `site/player.js`, `site/episode-player.js` and `site/navigation.js` move the page's `<audio>` into a footer that stays while the reader moves between pages. | An inline `<audio>` in the page; it stops when the reader leaves. | Plugin `podcast-episode` |
| Chapter timestamp seeking | `scripts/audio_chapters.py` rewrites H3–H6 headings that start with `MM:SS` into `recording#t=N` links before `lsg` runs; `player.js` seeks on click. 16 seek links on 2026-11-01. | Timestamps are plain heading text. 0 seek links on 2026-11-01. | Plugin `podcast-episode` |
| Timestamp-led bullets ([gitpa #139](https://github.com/codekiln/gitpa/issues/139)) | Not built. `05:15 - the bass comes in` stays text. | Same. | Plugin `podcast-episode` |
| Dialogue transcript seeking ([gitpa #85](https://github.com/codekiln/gitpa/issues/85)) | `lsg` renders `speaker · MM:SS — text` lines as `data-lsg-dialogue-time` links and fires `lsg:media-seek`; `player.js` handles the event and seeks the footer player. | `lsg` seeks the inline `<audio>` when no handler cancels the event. | `lsg` core already; the plugin's player handles the event |
| Previous and next header | `site/header-sequence.js` reads the `prev::` and `next::` rows and puts links in the header. | `prev::` and `next::` show only as property rows. | Plugin `sequence-header` |
| Homepage episode list | The authored advanced query on gitpa's `Ghost in the Patch` page is evaluated by `nbb-logseq` and its rows are written into `index.html` after `lsg` runs. | No list. `lsg` does not run advanced queries, `:result-transform` or `:view`. | `lsg` core (Logseq runs these queries on its published site), then garden content |
| Asset page visibility | `scripts/prepare_site.py` publishes an asset page only when a public episode links it. The 2026-10-20 `Synth-Forward/mp3` page is withheld. | Every page is public (`:publishing/all-pages-public? true`). | Gitpa only, before rendering |
| `public::` on a `## Notes` heading | `prepare_site.py` drops a notes section unless its first line is `public:: true`. | Every notes section is published. | Gitpa only, before rendering |
| Hidden administrative properties | `prepare_site.py` keeps six keys on episodes and gitpa's `config.edn` hides `public`, `podcast-guid` and others; 2026-11-01 shows `prev` only. | The garden's `config.edn` hides nothing; 2026-11-01 shows `prev`, `logseq-entity`, `public`, `created-by` and `date-created`. | Each garden's `config.edn` (standard Logseq) |
| Readable URLs and old links | `lsg --readable-urls --redirect-old-urls`; `navigation.js` maps old `#/page/…` links during its own navigation. | `lsg --readable-urls --redirect-old-urls`, including `#/page/…`. | `lsg` core already; gitpa's copy goes with `navigation.js` |
| Search | `lsg --search`. | `lsg --search`. | `lsg` core already |
| Theme | `lsg` theme control plus gitpa's Nord stylesheet (`--stylesheet site/theme.css`). | `lsg` theme control. | `lsg` core already; the stylesheet is gitpa's |
| Downloads | `prepare_site.py` turns preset and MIDI asset embeds into labelled links; the player leaves `download` links alone. | The episode page carries labelled links as the garden's episode definition asks. | Garden content |
| RSS feed | `scripts/build_rss.py` from `podcast-guid::`, `podcast-published-at::` and `public::`. | None. | Gitpa only |
| Copy feed URL button and artwork path | Added by `build_site.py`; the fly artwork is copied to its old path for subscribers. | None. | Gitpa only |

## Who owns each behaviour

- **`lsg` core** owns standard Logseq behaviour, so every garden gets it with no configuration. Only one core change is needed: run advanced queries with `:result-transform` and `:view` at build time, as Logseq's published site runs them in the browser. Dialogue seeking, old-link redirects, search and themes are already core.
- **First-party optional plugins** own behaviour beyond Logseq that more than one garden can use. They key off page properties and `logseq-entity::` types. Gitpa's code keys off `GitP/A/Session/` paths in `prepare_site.py` and `/gitpa/` URLs in `navigation.js`; the plugins use neither.
  - `podcast-episode` applies to pages whose `logseq-entity::` includes `[[Logseq/Entity/Podcast/Episode]]`. It finds the recording, the first audio embed under the page's first heading, at build time, and contributes the recording URL and the chapter list as page metadata. Its browser module keeps one player across pages, seeks on chapter headings and timestamp-led bullets, and handles `lsg:media-seek`.
  - `sequence-header` applies to any page with `prev::` or `next::` and puts the two links in the header.
- **Gitpa only** keeps what only the feed site needs: the RSS feed, `podcast-guid::`, `podcast-published-at::`, the Copy feed URL button, the artwork path kept for subscribers, and its Nord stylesheet. Its privacy selection (the `public::` episodes, public notes sections and the asset pages they link) stays in `prepare_site.py`, which runs before `lsg`. No browser plugin decides what is public.
- **Garden content** changes in logseq-encode-garden so both sites see one episode shape:
  - `GitP/A` carries `logseq-entity:: [[Logseq/Entity/Podcast/Episode]]`; it is the show and should carry `[[Logseq/Entity/Podcast]]`.
  - Sessions before 2026-10-18 lack `[[Logseq/Entity/Podcast/Episode/GitP/A]]` and `public::`.
  - The `prev::`/`next::` chain ends early: 2026-10-26, 2026-10-29 and 2026-11-01 have no `next::`, and 2026-10-09, 2026-10-12 and 2026-10-14 are outside the chain.
  - `GitP/A` gets the same episode-list query as gitpa's homepage, so the encode site shows the list once `lsg` runs advanced queries.

Decided by the agent without codekiln: hidden properties stay a per-garden `config.edn` choice. Logseq hides properties for the whole garden, and hiding `created-by` or `public` across the encode garden would remove information from thousands of other pages. The two sites will keep showing different property rows on episode pages.

Decided by the agent without codekiln: the homepage episode list is an `lsg` core change. Logseq's published site runs the authored advanced query, so a garden that publishes with Logseq already gets the list; a plugin would duplicate the query language for one garden.

## Where plugins live and how a garden turns one on

Decided by the agent without codekiln: first-party plugins live in the logseq-gardener repository, ship inside the `lsg` binary, and use the same manifest and host contract as any other plugin ([Plugin Host API](https://github.com/codekiln/logseq-gardener/blob/main/logseq-gardener-docs/pages/lsg___Ref___Plugin%20Host%20API.md), [Build Plugin Protocol](https://github.com/codekiln/logseq-gardener/blob/main/logseq-gardener-docs/pages/lsg___Ref___Build%20Plugin%20Protocol.md)). Each release tests them against its own host, so a site cannot pair a plugin with a host version it was never tested with. Both sites get a plugin fix by moving their pin.

A garden turns plugins on in `logseq/config.edn`, where `lsg` already reads its own keys such as `:publishing/page-sequence?`:

```clojure
:lsg/plugins [:podcast-episode :sequence-header]
```

Logseq ignores the key. An unknown plugin name stops the build before it writes output. `--plugin PATH` keeps working for a garden's own plugins. The OpenSpec proposal settles the key's spelling and lets a first-party plugin pair an in-process build step with a browser module, which in-process plugins cannot do today.

## Cost on the encode garden

The budgets come from [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) and [issue 106, Match Logseq's published site for logseq-encode-garden](https://github.com/codekiln/logseq-gardener/issues/106): 100 KB of data and 16 KB of script before the title, a 10 MB peak heap on the phone profile, and a host module of 4 KB gzip.

- **Every page once any browser plugin is on.** A `data-lsg-page` element of a few hundred bytes and the host module, loaded after first paint. The host also takes over navigation between pages for the whole garden. That is the main behaviour change for the encode site's 9,153 pages, and the plugin issues measure it with `parity:perf`.
- **`podcast-episode`.** `on-demand`, so only an episode page, or a page reached after one, imports it. Gitpa's equivalent player code is about 7 KB gzip (`player.js` 4.4 KB, `episode-player.js` 0.7 KB, `plugin-adapter.js` 1.9 KB); the host replaces gitpa's 5.5 KB `navigation.js`. Budget: 8 KB gzip of script and styles, none of it before first paint, one `<audio>` element, and at most 1 MB of added heap on the phone profile. The build step reads blocks of episode pages only; it stays within the `[build_plugins]` in-process row of 0.05 ms per published page.
- **`sequence-header`.** `on-demand`; its links come from build metadata. Budget: 1.5 KB gzip.

## Migration order

1. `lsg` core: run advanced queries with `:result-transform` and `:view` at build time.
2. Propose: an OpenSpec change for first-party plugins and the `:lsg/plugins` key.
3. `podcast-episode`: persistent player and `lsg:media-seek`.
4. `podcast-episode`: chapter headings seek the recording.
5. `podcast-episode`: timestamp-led bullets seek the recording.
6. `sequence-header`.
7. A release, then the consumers: gitpa moves its pin, turns the plugins on and deletes its player, navigation, header and query code; the encode garden turns the plugins on and brings its episode pages to one shape.

## Fixture pages

Both sites are compared on these pages, served locally, on the desktop and phone profiles:

- `GitP/A/Session/26/11/01-Sun`: timestamp-led bullets, newest episode, no `next::`.
- `GitP/A/Session/26/10/29-Thu`: the hand-annotated chapter the episode definition points to.
- `GitP/A/Session/26/10/24-Sat`: dialogue timestamps, both `prev::` and `next::`, preset and MIDI downloads.
- `GitP/A/Session/26/10/20-Tue`: the largest episode, with about 120 timestamps and segmented recordings.
- `GitP/A/Session/26/09/24-Thu`: an episode in the older shape.
- `GitP/A` and `GitP/A/Session`.
- `GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3`: an asset page.
- Each homepage: gitpa's `Ghost in the Patch` and the encode site's journals.

On each episode page a reader can start the recording, follow a link to another page with the recording still playing, seek from a chapter heading, a timestamp-led bullet and a dialogue line, and follow the header's previous and next links.

## Out of scope

- Per-namespace hidden properties, which Logseq has no setting for.
- Moving gitpa's RSS build into a plugin. Only gitpa publishes a feed.
- Restoring transcripts ([gitpa #72](https://github.com/codekiln/gitpa/issues/72)) and the wider reading improvements in [gitpa #70](https://github.com/codekiln/gitpa/issues/70).
