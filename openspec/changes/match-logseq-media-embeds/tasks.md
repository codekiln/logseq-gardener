## 1. SDK

- [ ] 1.1 Add the `media` module: `VideoEmbed` classification for `{{video}}`, `{{youtube}}`, `{{vimeo}}` and `{{bilibili}}` following `get-matched-video`, with `t=` start times from query or fragment in seconds or `XhYmZs` form; unit-test every provider row of the analysis's table, `?t=14`, `&t=205s`, `{{youtube}}` with an id and with `&t=`, invalid and empty arguments
- [ ] 1.2 Add `Timestamp` parsing and `display()`; unit-test `0`, `41`, `620`, `3600`, `4:40`, `1:02:03` and rejected arguments (`61:00`, `abc`)
- [ ] 1.3 Add the per-page media index: video numbers in outline order and each timestamp's bound video; test the Omer Hamerman and AI Engineer Summit outlines, including the repeated id
- [ ] 1.4 Use `Timestamp::display()` for timestamps in search records, block reference text and breadcrumbs; test the `10:20 Agent skill` reference

## 2. Publisher

- [ ] 2.1 Render video macros in `site/render.rs` through new `site/media.rs`: the placeholder markup, the `Invalid URL` and `Empty URL` chips, and diagnostics; remove `video`, `youtube` and `youtube-timestamp` from the literal list in `macro_fallback`
- [ ] 2.2 Render bound, unbound and copied timestamps, with the copied form in reference and embed contexts
- [ ] 2.3 Render `.mp4`, `.webm` and `.mov` image syntax as `div.asset-ref-wrap` with `<video controls preload="none">`
- [ ] 2.4 Add `site/media.css`: placeholder size and look, the clock mask icon, the timestamp colour in light and dark, the video file layout
- [ ] 2.5 Add `site/media.js`: click-to-load, `lsg:media-seek` dispatch, loading at a start time, `postMessage` seek and play, modified-click passthrough, idempotent install
- [ ] 2.6 Link `media.js` and `media.css` only from pages with a video or bound timestamp; test that `tmux` is unchanged
- [ ] 2.7 Update `published-inline-markup` tests for the new macro handling

## 3. Harness and fixtures

- [ ] 3.1 Add a fixture-garden page with every provider, `{{youtube}}`, `?t=`, invalid and empty arguments, a `.mp4` URL on another host and `![Film](../assets/film.mp4)`
- [ ] 3.2 Add the `[media]` table to `scripts/parity/budgets.toml` and make `parity:budgets` check `media.js`, `media.css`, the placeholder markup and the Maggie Appleton page growth
- [ ] 3.3 Make `parity:perf` record third-party requests before a click on the two video fixture pages; make `parity:screenshots` capture the Omer Hamerman player area at 1280×800 and 390×844

## 4. Verification

- [ ] 4.1 Build the encode garden with `parity:build`; compare the fixture pages with the `gh-pages` build: player area size and position, timestamp text, icon size and colour, the AI Engineer Summit start times; record values and screenshots in `verification.md`
- [ ] 4.2 Check seeking in a browser: `04:40` before and after loading, the repeated video, a cancelling `lsg:media-seek` listener, keyboard activation, and the no-JavaScript link
- [ ] 4.3 Run `parity:perf --record` and commit the media rows in `docs/parity/performance.md`
- [ ] 4.4 Run `mise run ci`
