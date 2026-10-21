# Verification

The October 21, 2026 Gitpa candidate was generated from its explicitly public prepared graph with the authored homepage query evaluated by nbb-logseq. Chrome inspection at http://127.0.0.1:8765/ confirmed the light Nord palette, sidebar, Ghost in the Patch title, introductory paragraph, RSS link, fly logo, and nine Recent Ceremonies links in the same order as https://codekiln.github.io/gitpa/.

The October 20 episode contains twelve audio controls and eleven waveform images. Its full-recording play button changed to Pause and enabled the playback scrubber after loading; playback was then paused. Microfreak/UG/06 Dig Osc rendered transcluded explanations and images rather than literal embed macros. Legacy episode hash URLs are mapped by Gitpa to the generated routes.

`mise run ci` passed formatting, Clippy, workspace tests, title validation, release build, strict OpenSpec validation, parser fixtures, skill versions, generated configuration, dependency notices, documentation examples, CLI conformance, and release configuration. Focused transclusion tests cover selected aliases, UUID descendants, original-source assets, excluded/private content, duplicate targets and IDs, cycles, expansion work, and valid paragraph/heading containers. Homepage tests cover titles, aliases, unavailable/ambiguous selectors, retained routes and media, and omitted-selector behavior.

An independent technical review found namespace selection could expose unmarked descendants in the Gitpa bridge and flow embeds could nest illegally inside headings or paragraphs. Both were corrected and regression tests added before deployment. Gitpa's Python suite passed 37 tests. Its build evaluates the real Datalog query and rejects unsupported views or unavailable destinations.
