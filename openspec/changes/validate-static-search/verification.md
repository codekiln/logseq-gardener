# Verification

The six recorded corpus/profile rows in [search-performance.json](https://github.com/codekiln/logseq-gardener/blob/main/docs/parity/search-performance.json) pass unchanged 50/100 ms query budgets, new cold/warm/heap/corpus limits, zero eager search requests and zero prepared warm requests. Each profile contains 24 completed keys and zero missing completions. The actual selected corpus has 996 pages/3919 bullets; the larger generic corpus has 2048 pages/16384 bullets. Exact controller and bootstrap contents in all measured publications were compared against the implementation source.

Initial script accounting exposed the eager dialog cost and motivated the lazy bootstrap. Generic publication now measures 15273 initial script units against the existing 16000 limit. The actual Gitpa host's remaining player/navigation data/script baseline is reported separately; production deployment is outside this change.

The existing Chromium functional proof now also verifies frame coalescing, corrupt corpus and offline recovery, lazy-module failure/retry, options and metadata changes during held import, bounded buffered typing without underlying GA/TT shortcuts, and Escape restoring page typing/Tab while import remains pending. Existing privacy/alias/source-anchor and pure evaluator/loader fixtures remain in full CI. Native cleanup153 tracks separate sync/archive after merge.

Final verification commands: `mise run search:browser-test`, `mise run parity:budgets`, `mise run ci`. The optimized review binary is generated from this implementation. Root independently verified actual advancing audio through canonical page and original-source-block result navigation in the final Gitpa consumer.

Rebased onto current main `cbad7796f2bddd1842a1f5d5b8aac7a870e62e41`, preserving published outlines, zoom measurement and outline budgets. The sync 661-byte host-options shim defers the 1969-byte starter after paint, retaining the current 16000 initial-script ceiling without changing the 50/100 ms query limits. The six current-profile rows replace the pre-outline matrix.

The independent final host proof includes native bullet zoom, parent breadcrumb unzoom, search navigation followed by Back restoring zoom and Forward, original block focus, and advancing playback through the same audio element.
