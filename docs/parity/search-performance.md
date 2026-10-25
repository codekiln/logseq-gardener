# Static search validation

Shared search stays within the existing 50ms desktop and 100ms phone p95 keystroke budgets. It evaluates current input once per animation frame. The measurement observes completed results after rendering, including empty and unchanged matches; stale-result clearing is excluded. Each recorded corpus/profile sample types 24 characters across `music`, `needle` and `zz-no-match96`, with the existing 300ms key gap and nearest-rank percentile calculation.

The initial95 controller exceeded the existing 16KB generic initial-script budget:25,826 on a small publication. A fingerprinted startup module now imports the full dialog only on first open. The corpus and evaluator remain lazy. The startup preserves host options and publication changes during import. It buffers up to 512 UTF-16 units while loading, isolates pending keys from page shortcuts, and restores ordinary typing/Tab immediately when cancelled. Failed module imports retry with a request suffix to bypass browser-cached module failure; the fingerprinted module content remains immutable.

## Measurements

[Recorded matrix](search-performance.json) contains the selected corpus counts, sizes and raw desktop/phone rows. Profiles use the existing parity harness: desktop 1280×800 without throttling; phone 412×823,4× CPU slowdown, 150ms latency and 1.6384Mbps download. Each row is one fresh context followed by cached reopening; these are local Chromium measurements, not production telemetry.

| Corpus | Pages / bullets | Profile | p95 key (ms) | First ready (ms) | Warm ready (ms) | Transferred body (KB) | Heap growth (MB) |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| small | 16 / 128 | desktop | 2.4 | 79.7 | 15.5 | 8.6 | 0.0 |
| small | 16 / 128 | phone | 3.0 | 846.8 | 29.1 | 8.6 | 0.1 |
| large | 2048 / 16384 | desktop | 11.7 | 138.7 | 23.7 | 255.7 | 5.4 |
| large | 2048 / 16384 | phone | 23.1 | 2377.6 | 42.4 | 255.7 | 8.7 |
| actual | 996 / 3919 | desktop | 5.2 | 136.7 | 12.2 | 257.2 | 4.1 |
| actual | 996 / 3919 | phone | 11.3 | 2357.6 | 19.3 | 257.2 | 3.0 |

First-open readiness includes controller import, validated manifest/corpus loading, JSON parsing, index preparation and initial empty-query rendering. Transferred body lengths come from Resource Timing encoded body size; the local parity server sends eligible text as gzip. Corpus `gzip_bytes` is separately a reproducible offline level6 estimate. Decoded lengths include the UI/evaluator/manifest as well as the corpus. Heap growth is measured at readiness, not a guarantee of the maximum during parsing. Warm requests count search artifacts only; unrelated host audio requests are excluded.

The generic small/large publications keep all initially referenced scripts, including modules/deferred scripts, below16KB. Their page/title/data/heap measurements remain within the existing generic budgets. CI now accounts conservatively for these initial scripts on search-enabled fixtures so a lazy-corpus claim cannot conceal an eager dialog module.

Gitpa's host baseline is reported separately from the shared generic shell. Its persistent player/navigation and compatibility assets add site-owned work. The final measured host records28.5–39.3 KB of script and131–135 KB of data before its title, above the generic16 KB/100 KB limits; the search data remains absent before opening. Host integration is independently reviewed in [Gitpa83 — shared search integration](https://github.com/codekiln/gitpa/issues/83), including advancing audio through canonical page and original-block navigation. This validation does not claim production deployment.

## Budgets and architecture

The existing 50/100ms per-key limits and16KB/100KB generic initial script/data limits are unchanged. `scripts/parity/budgets.toml` additionally caps the bootstrap at 2 KB, decoded corpus at 8 MB, offline gzip estimate at 1 MB, first-open readiness at 2 seconds desktop/10 seconds phone, heap growth at 30 MB, and warm readiness at 100/200 ms. Eager data requests and prepared warm-open requests must both be zero. These loading limits allow headroom over the measured larger corpus while making growth visible; larger publications require new measurement.

The largest tested generic corpus has 2,048 pages and 16,384 original bullets. Its completed query and loading results support one lazy corpus and synchronous pure evaluation for the measured scope. Chunking or a Worker becomes appropriate if supported corpus/profile measurements exceed these limits; neither is needed to meet the current matrix. A successful small synthetic benchmark alone would not establish this decision.

## Reproduce

Build a release exporter with `mise run build:release`, then publish any selected fixture with `--search`. Measure it through the existing harness:

```sh
mise run parity:perf --search-site /tmp/selected-site --search-output /tmp/search-results.json --search-queries /tmp/search-queries.txt
```

For a host with a URL prefix, serve a parent containing that directory and add `--search-path gitpa/index.html`. The standalone mode uses the same profiles, resource probes, completion timestamps, percentiles, budgets and report writer as the normal parity command. It fails unavailable readiness, missing/nonfinite completion evidence, unchanged keystrokes or exceeded search limits. It does not require an unrelated Logseq export or feed/build benchmark.

The synthetic matrix uses 16 and 2,048 stored pages under `Notes`, eight original bullets per page, and unique `Sample N` aliases. Each bullet is `Source note I about static garden music and reading. Needle topic N-I.`. Publish with `--filename-format triple-lowbar --include Notes --search`. Use the three queries above, one per line. This makes the generated corpus reproducible without access to personal source notes.

`mise run search:browser-test` reuses the generated-publication functional proof: keyboard/IME/focus, mobile/themes, ordinary/modified links, nested/host routes, stale close/revision changes, corrupt/missing/offline artifacts, module failure/retry and pre-import host-option handoff. Offline before first open shows an actionable failure while the current page remains usable; returning online can retry. Prepared search can reopen offline. There is no offline reload or Service Worker guarantee. Existing Rust serialized privacy/source-anchor fixtures and pure engine/loader tests remain in `mise run ci`.
