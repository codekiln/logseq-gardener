# Verification

This file records the evidence for the tasks of #487 and #488. The tasks of #489 and #490, and the change-wide Verify step (6.1 to 6.3), add their own sections when they land.

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
