# Verification

This file records the evidence for the tasks of #487 to #490 and the change-wide Verify step (6.1 to 6.3).

## 1.10 Logseq ignores the key

`mise run parity:logseq-export` built Logseq 0.10.15's own export (frontend from `/Applications/Logseq.app`, export source from the pinned tag) of two copies of `tests/fixtures/parity-garden`, one without the key and one with `:lsg/plugins [:podcast-episode]`, with the lsg options `--filename-format triple-lowbar --journals --include Tools`, on 2026-11-08.

| Check | Without the key | With the key |
| --- | --- | --- |
| Export log | "Export public pages and publish assets … successfully" | the same line, no other output |
| Pages in Logseq's export | 27 compared, 0 missing | 27 compared, 0 missing |
| Page-by-page comparison with lsg | 15 pages differ | the same 15 pages, with the same words (the report's `pages` lists are identical) |

Neither log names a configuration error. The two `index.html` files differ only in block UUIDs, which Logseq generates on each export, and in 33 bytes: Logseq embeds the garden's `config.edn` text, which now holds the key. The 15 differing pages differ in both builds for reasons outside this change (for example, Logseq shows a ClojureScript error on a query page that lsg runs).

## 1.9 Build time on the pinned encode garden

`mise run parity:build-bench --garden target/encode-garden --runs 5` at encode garden commit `5095bdf` (13,524 published pages), fastest of five builds on an Apple Silicon Mac:

| Variant | Fastest build | Added time per page | Budget |
| --- | --- | --- | --- |
| lsg without plugins | 16.78 s | | |
| first-party podcast-episode | 16.39 s | −0.0290 ms | 0.05 ms |
| in-process no-op with blocks | 16.22 s | +0.0181 ms | 0.05 ms |

The difference is within the run-to-run noise; the step reads each page's leading properties and only an episode's outline.

## 2.4 Browser checks

`mise run plugins:browser-test` passed with the new podcast-episode scenarios: no plugin request on a page without an episode, the same `<audio>` element playing across a link and Back (0.30 s, then 0.52 s), the module requested once, a dialogue timestamp seeking the player to 11.5 s, a second episode taking over only from its Play button, and a page whose module request fails keeping its inline recording, which its dialogue line seeks.

## 3.2 Chapter entries and page growth

`mise run parity:budgets` passed. Each first-party copy of the fixture garden gains `Tools/Plugins/Show/Long Episode` with 120 timestamped chapter headings, as many as the largest GitP/A episode has timestamps:

| Row | Measured | Budget |
| --- | --- | --- |
| `podcast-episode` script and styles | 4,694 gzip | 8,000 |
| Long Episode entry | 120 chapters in 14,137 bytes of JSON | under 64,000 |
| Largest page growth (Long Episode) | 1,816 gzip | 4,000 |
| `Tools/Queries` without an entry | 146 gzip | 500 |

Search files match the build without the key.

## 3.3 Chapter headings in Chromium

`mise run plugins:browser-test` passed with the new scenario: the `00:20` heading timestamp on the fixture episode seeks the playing `<audio>` element to 20.2 s, the player shows `Opening › Twenty seconds in`, and when the module fails the heading stays text.

The branch's `lsg` built the `GitP` namespace of logseq-encode-garden (`66b81e9543`) and of gitpa's `gitp-garden` (`410f4d9`), each from a copy whose `logseq/config.edn` adds `:lsg/plugins [:podcast-episode]`, on 2026-11-08. Chromium counted the chapter seek links after the module loaded, on the desktop (1280×800) and phone (375×812) profiles alike. Recordings were served as a local tone with byte ranges.

| Page | Seek links, encode garden | Seek links, gitpa's copy | gitpa's site today |
| --- | --- | --- | --- |
| `GitP/A/Session/26/11/01-Sun` | 21 | 16 | 16 |
| `GitP/A/Session/26/10/29-Thu` | 19 | 19 | 19 |
| `GitP/A/Session/26/10/24-Sat` | 7 | 7 | 7 |
| `GitP/A/Session/26/10/20-Tue` | 11 | 11 | 11 |

The encode garden's `01-Sun` has five more timestamped headings than gitpa's copy of the page. Selecting `18:17` on `01-Sun` played the recording from 1,097 s and showed `Enter Microfreak/Preset/260 8bitfi`; selecting `14:51` on `29-Thu` played from 891 s and showed `pierced resonance`. No page raised an error.

Built without the key, the encode garden's `GitP` namespace is byte-identical to `lsg` 0.0.38 (main at `b1773f9`). With the key, the files that differ from 0.0.38 with the key, apart from the build revision in each page's metadata, are the plugin's module and stylesheet, `lsg-site.json` and the ten episode pages with chapters. Their gzip size grows by 136 to 397 bytes over 0.0.38 and by 288 to 552 bytes over the build without the key.

## 4.1 and 4.2 Timestamp-led bullets

`cargo test -p logseq-gardener-sdk podcast` covers a bullet nested under a chapter heading and under a plain bullet, a bullet deeper than H6, a timestamp later in a bullet, one in inline code, in a code block or after emphasis, a task marker or a priority, and timestamp-led bullets outside `## Chapters`. `tests/site_plugins.rs` checks the fixture episode's six chapters, three of them bullets, and the anchor on each block.

`mise run plugins:browser-test` passed with the new scenario: the `00:25` bullet under the `00:20` heading seeks the playing `<audio>` element to 25.2 s and the player shows `Opening › Twenty seconds in › the bass comes in`. A timestamp later in a bullet and one in code stay text, and when the module fails the bullet stays text.

The branch's `lsg` built the `GitP` namespace of logseq-encode-garden (`66b81e9543`) and of gitpa's `gitp-garden` (`410f4d9`), each from a copy whose `logseq/config.edn` adds `:lsg/plugins [:podcast-episode]`, and the same copies with `lsg` from main (`14974d4`), on 2026-11-08. Chromium counted the seek links after the module loaded, on the desktop (1280×800) and phone (375×812) profiles alike, with the recordings loaded from Backblaze.

| Page | Encode garden, main | Encode garden, branch | gitpa's copy, main | gitpa's copy, branch |
| --- | --- | --- | --- | --- |
| `GitP/A/Session/26/11/01-Sun` | 21 | 41 (20 bullets) | 16 | 36 (20 bullets) |
| `GitP/A/Session/26/10/29-Thu` | 19 | 19 | 19 | 19 |
| `GitP/A/Session/26/10/24-Sat` | 7 | 7 | 7 | 7 |
| `GitP/A/Session/26/10/20-Tue` | 11 | 11 | 11 | 11 |
| `GitP/A/Session/26/09/24-Thu`, `GitP/A`, `GitP/A/Session`, the `24-Sat` asset page | 0 | 0 | 0 | 0 |

Selecting `05:15` in `05:15 - the bass comes in` on `01-Sun` played the recording from 315 s on both copies and both profiles. The encode garden's trail read `Enter Microfreak/Preset/259 Infini Pad › Infini Pad as bass › the bass comes in - Infini Pad works great as…`; gitpa's copy, which nests the page differently, read `Infini Pad as bass › the bass comes in - …`. No page raised an error.

`29-Thu` has no bullet under `## Chapters` that starts with a timestamp in either garden: its hand-annotated chapter (`00:00 enter Voidom`) marks moments with headings, which seek since #488, so its count is unchanged. The `46:39 · …` bullet under `## Recording and Prep` stays text.

Three episodes gained chapters. Their growth over main with the key on:

| Page | Chapters | Entry | Page gzip growth |
| --- | --- | --- | --- |
| `GitP/A/Session/26/10/18-Sun` | 12 → 137 | 15,486 bytes | 3,648 bytes |
| `GitP/A/Session/26/11/01-Sun` | 21 → 41 | 4,229 bytes | 449 bytes |
| `GitP/A/Session/26/11/06-Fri` | 17 → 30 | 3,059 bytes | 279 bytes |

`18-Sun` gains 125 chapters from its spoken-word lines, as the design's risk on spoken-word lines expects. Its chapter titles held whole transcript lines and grew the page by 11 KB gzip, so the entry cuts each title to 48 characters at a space; the page keeps the full text. Every other file of the encode garden's `GitP` namespace is the same as main's apart from the build revision.

`mise run parity:budgets` passed: the module and stylesheet are 4,906 bytes gzip (budget 8,000), the 120-chapter Long Episode entry is 14,137 bytes, and its page grows by 1,856 bytes gzip (budget 4,000).

## 5.3 sequence-header in Chromium and in the budgets

`mise run plugins:browser-test` passed with the new sequence-header scenarios: a page outside the sequence requests no file under `plugins/sequence-header/` and shows no links; a page with both neighbours shows a Previous link before the header's spacer and a Next link after it, with `rel`, the page titles and the title `Seq/Three & <b>bold</b>` shown as text; Tab reaches both links with a visible outline; following a link to a page without an entry removes the links and Back brings them back; the last page shows only its Previous link; the module is requested once; on a 375-pixel phone each link is a 36-pixel arrow and the header does not overflow.

`mise run parity:budgets` passed:

| Row | Measured | Budget |
| --- | --- | --- |
| `sequence-header` script and styles | 1,465 gzip | 1,500 |
| Largest page growth (`Tools/Plugins/Show/Episode One`, both plugins' entries) | 282 gzip | 500 |
| Fixture pages with an entry | `Episode One` and `Episode Two`, the two pages whose neighbours are published | |

## sequence-header on both gardens

The branch's `lsg` built the `GitP` namespace of logseq-encode-garden (`66b81e9543`) and of gitpa's `gitp-garden` (`410f4d9`), each from a copy whose `logseq/config.edn` adds `:lsg/plugins [:podcast-episode :sequence-header]`, with readable URLs, on 2026-11-08. Gitpa's site today was built with its own `scripts/build_site.py` and its pinned `lsg` 0.0.29, which draws the header links with `site/header-sequence.js`. Chromium read the header links on the desktop (1280×800) and phone (375×812) profiles, served locally.

| Page | Encode garden | Gitpa's copy | Gitpa's site today |
| --- | --- | --- | --- |
| `GitP/A/Session/26/11/01-Sun` | Previous `26/10/29-Thu` | the same | the same |
| `GitP/A/Session/26/10/29-Thu` | Previous `26/10/26-Mon` | the same | the same |
| `GitP/A/Session/26/10/24-Sat` | Previous `26/10/22-Thu`, Next `26/10/26-Mon` | the same | the same |
| `GitP/A/Session/26/10/20-Tue` | Previous `26/10/18-Sun`, Next `26/10/22-Thu` | the same | the same |
| `GitP/A/Session/26/09/24-Thu` | Previous `26/09/25-Fri`, Next `24/11/19-Tue` | the same | the same |
| `GitP/A`, `GitP/A/Session`, the asset page `26/10/24-Sat/Asset/Audio/Full/mp3` | no links, no plugin request | the same | no links |
| Homepage (journals; `Ghost in the Patch`) | no links, no plugin request | the same | no links |

Each link has the same width as gitpa's on the desktop (for example 236 and 241 pixels on `24-Sat`) and is a 36-pixel arrow on the phone, where the title is visually hidden, as on gitpa. No header overflowed and no page raised an error. On both `lsg` builds, following a link from `24-Sat` to `GitP/A` removed the links and Back showed `24-Sat`'s two links again.

## 6.1 Every fixture page on both gardens

Both plugins work on every fixture page of both gardens, on the desktop and phone profiles, and no page raised an error. The branch's `lsg` built the `GitP` namespace of logseq-encode-garden (`66b81e9543`, with journals) and of gitpa's `gitp-garden` (`410f4d9`, with `Ghost in the Patch` as the homepage), each from a copy whose `logseq/config.edn` adds `:lsg/plugins [:podcast-episode :sequence-header]`, with readable URLs, on 2026-11-08. Chromium opened each page directly on the desktop (1280×800) and phone (375×812) profiles; the two profiles gave the same counts. Recordings were served as a local tone.

| Page | Seek links, encode / gitpa | Header links, both gardens | Episode entry, encode / gitpa |
| --- | --- | --- | --- |
| `GitP/A/Session/26/11/01-Sun` | 41 / 36 | Previous `26/10/29-Thu` | 4,211 / 3,709 |
| `GitP/A/Session/26/10/29-Thu` | 19 / 19 | Previous `26/10/26-Mon` | 1,957 / 1,959 |
| `GitP/A/Session/26/10/24-Sat` | 7 / 7 | Previous `26/10/22-Thu`, Next `26/10/26-Mon` | 887 / 887 |
| `GitP/A/Session/26/10/20-Tue` | 11 / 11 | Previous `26/10/18-Sun`, Next `26/10/22-Thu` | 1,338 / 1,338 |
| `GitP/A/Session/26/10/18-Sun` | 137 / 137 | Previous `26/10/11-Sun`, Next `26/10/20-Tue` | 15,290 / 15,291 |
| `GitP/A/Session/26/09/24-Thu` | 0 / 0, with the player | Previous `26/09/25-Fri`, Next `24/11/19-Tue` | 164 / 164 |
| `GitP/A`, `GitP/A/Session`, the asset page `26/10/24-Sat/Asset/Audio/Full/mp3`, the homepage | 0, no plugin file requested | none | none |

Entries are the characters of the page's `podcast-episode` entry as JSON; the largest, `18-Sun`, is under a quarter of the 64,000 limit. Every episode page requested both plugins' files; no other page requested either. No header overflowed on the phone.

On each garden and profile, the same `<audio>` element kept playing through this visit: start `24-Sat`, select its dialogue line at 855 s (the player moved to 855 s; the line sits in a collapsed transcript block, so the check clicked it from script), follow links to `GitP/A`, the asset page and `GitP/A/Session`, then go Back. On `01-Sun`, selecting the heading `18:17` moved the player to 1,097 s and selecting the bullet `05:15` moved it to 315–316 s. The encode garden's trail read `Enter Microfreak/Preset/260 8bitfi › Let's try this 8bitfi lead, 260.` and then `Enter Microfreak/Preset/259 Infini Pad › Infini Pad as bass › the bass comes in - …`. Gitpa's copy has chapters at 1,096 and 1,097 s, so its first trail named whichever chapter was current when the check read it.

## 2.5 and 6.2 The whole encode garden against Logseq

Every row is within its budget. `mise run parity:perf --first-party --record` built a copy of the whole encode garden (`66b81e9543`, 9,202 published pages) whose `logseq/config.edn` turns on both plugins, with `lsg` 0.0.41 (main at `f64db09`; this change adds no Rust), and Logseq 0.10.15's own export of the same copy, on 2026-11-08 on an Apple Silicon Mac. The rows are in the First-party plugins section of [Performance](../../../../docs/pages/lsg___Ref___Logseq%20Compatibility___Performance.md).

| Row | Measured, worst fixture page | Budget |
| --- | --- | --- |
| Title, desktop | 32 ms (`29-Thu`) | 300 ms |
| Title, phone | 417 ms (homepage) | 1,500 ms |
| Data before the title | 32.9 KB (`20-Tue`); 31.2 KB on `18-Sun`, the 137-chapter episode | 100 KB |
| Script before the title | 12.2 KB (`24-Sat`, `20-Tue`) | 16 KB |
| Peak JS heap, phone | 2.4 MB (`18-Sun`) | 10 MB |
| `podcast-episode` phone heap growth after playing across 20 navigations | 0.14 MB | 1 MB |
| `sequence-header` phone heap growth over the same visit | 0.03 MB | 0.2 MB |
| `podcast-episode` module and stylesheet | 4,906 gzip | 8,000 |
| `sequence-header` module and stylesheet | 1,465 gzip | 1,500 |
| Pages that load plugin code before the title | 0 | 0 |

Logseq's export of the same pages took 27 to 32 s to the title on the desktop and 196 to 211 s on the phone, loaded about 24,000 KB of data before the title and peaked at about 2,100 MB of heap.

The phone heap is the median of three visits that start `24-Sat`, start the recording and follow 20 links in place: 1.55 MB with both plugins on and the recording playing, 1.41 MB with `podcast-episode` blocked and 1.39 MB with both blocked. The three visits of each kind were within 3 KB of one another. The recording was still playing after the 20th link.

Two changes to the measurement came with this run. The run now measures `sequence-header`'s heap growth as well as `podcast-episode`'s, and includes `18-Sun`. Script before the title now counts only inline JavaScript: the first run counted the page's `<script type="application/json" data-lsg-page>` metadata as script, which put `01-Sun` at 16.2 KB and would have put `18-Sun`, whose entry is 15 KB, far over 16 KB. That element runs no code, and its size already counts in data before the title. The earlier Plugins section of the Performance page was measured with the old count, so its script figures include the recorder plugin's metadata.

Per-keystroke search latency was not measured in this run: the plugins add no search records, and `mise run parity:budgets` checks in CI that the search files match the build without the key.

## 6.3 CI

`mise run ci` passed on the branch on 2026-11-08.
