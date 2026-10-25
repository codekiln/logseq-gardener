# Verification

The six recorded corpus/profile rows in [search-performance.json](https://github.com/codekiln/logseq-gardener/blob/main/docs/parity/search-performance.json) pass unchanged 50/100 ms query budgets, new cold/warm/heap/corpus limits, zero eager search requests and zero prepared warm requests. Each profile contains 24 completed keys and zero missing completions. The actual selected corpus has 996 pages/3919 bullets; the larger generic corpus has 2048 pages/16384 bullets. Exact controller and bootstrap contents in all measured publications were compared against the implementation source.

Initial script accounting exposed the eager dialog cost and motivated the lazy bootstrap. Generic publication now measures 15964 initial script units against the existing 16000 limit. The actual Gitpa host's remaining player/navigation data/script baseline is reported separately; production deployment is outside this change.

The existing Chromium functional proof now also verifies frame coalescing, corrupt corpus and offline recovery, lazy-module failure/retry, options and metadata changes during held import, bounded buffered typing without underlying GA/TT shortcuts, and Escape restoring page typing/Tab while import remains pending. Existing privacy/alias/source-anchor and pure evaluator/loader fixtures remain in full CI. Native cleanup153 tracks separate sync/archive after merge.

Final verification commands: `mise run search:browser-test`, `mise run parity:budgets`, `mise run ci`. The optimized review binary is generated from this implementation. Root independently verified actual advancing audio through canonical page and original-source-block result navigation in the final Gitpa consumer.
