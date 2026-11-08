# Verification

This file records the evidence for the tasks of #487 that name it. The tasks of #488, #489 and #490, and the change-wide Verify step (6.1 to 6.3), add their own sections when they land.

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
