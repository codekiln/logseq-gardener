## Why

Logseq turns `{{video URL}}` into a YouTube, Vimeo, Bilibili or Loom player and `{{youtube-timestamp N}}` into a clock-and-time link that seeks the YouTube player above it. `lsg` shows both macros as literal code, and the encode garden uses them 118 and 2,727 times, so every YouTube note page shows source text where Logseq shows a player ([Video embeds and YouTube timestamps analysis](../../../docs/parity/media-embeds.md), [Issue #122 — Match Logseq's video and YouTube embeds and timestamps](https://github.com/codekiln/logseq-gardener/issues/122)).

Logseq loads every player as soon as the page renders, about 1.2 MB from YouTube and Google per page with a video, before the reader asks for it. `lsg` can show the same player area as a click-to-load placeholder, so pages with videos keep their current first paint and make no third-party request until the reader clicks.

## What Changes

- `{{video URL}}`, `{{youtube URL}}`, `{{vimeo URL}}` and `{{bilibili URL}}` render a player area in the block's content: full block width, 315px tall on desktop and `min(viewport − 96, 560) × 315/560` on phones, with a `16px 0` margin, as Logseq draws it.
- The player area is a placeholder until the reader clicks it: a dark box with a play button and an accessible name, which is also a link to the video so it works without JavaScript. A click replaces it with the provider's iframe, playing.
- The SDK classifies each URL as Logseq's `get-matched-video` does (YouTube, Loom, Vimeo, Bilibili, `youtube-nocookie.com`, any other URL) into an embed URL and a start time. The start time comes from `t=` in either the query or the fragment, in seconds or `1h2m3s` form; Logseq reads only `&t=<digits>`.
- A non-URL argument renders the `Invalid URL` warning chip, and a missing one the `Empty URL` chip, as Logseq does.
- `{{youtube-timestamp N}}` renders `a.svg-small.youtube-timestamp` with the clock icon and Logseq's `MM:SS` or `HH:MM:SS` time. Unlike Logseq, the link has an `href` to the video at that time, so it is focusable and works without JavaScript. An argument Logseq rejects renders nothing and reports a diagnostic.
- The SDK binds each timestamp to the nearest YouTube video before it in its page's outline. A click first offers the existing `lsg:media-seek` event, then loads the bound player at that time or seeks it if loaded, and plays. In reference and embed copies the timestamp links to its source video on YouTube. A timestamp with no video before it shows a plain time.
- Image syntax for `.mp4`, `.webm` and `.mov` renders `div.asset-ref-wrap` with `<video controls preload="none">`, extending the audio rule.
- Search snippets, block reference text and breadcrumbs show a timestamp as its time text.
- `media.js` and the media styles load only on pages with a video or a timestamp. A new `[media]` budget table caps the script, each placeholder's markup and the growth of the largest timestamp page.

## Capabilities

### New Capabilities

- `published-media-embeds`: video macros, providers, start times, the placeholder and player, timestamp links and their binding, seeking, video files, plain-text forms, loading and budgets.

### Modified Capabilities

- `published-inline-markup`: `video`, `youtube` and `youtube-timestamp` leave the list of macros that keep their literal rendering, and `vimeo` and `bilibili` stop rendering as unknown-macro warnings.

## Impact

- SDK: a new `media` module that parses the video and timestamp macros, classifies URLs into providers, embed URLs and start times, gives each video a per-page index, binds timestamps to videos in outline order, and supplies the plain-text form of a timestamp. Search text, block reference text and breadcrumbs use that form. The CLI and the language server can list a page's videos and timestamps from the same operation.
- Publisher: the macro arms in `site/render.rs`; video-file markup in the `media` path of `site/render.rs`; new `site/media.rs`, `site/media.js` and `site/media.css`; loading them only on pages that need them, as `site/dialogue.rs` does for `dialogue.js`.
- Parity harness: `scripts/parity/budgets.toml` gains a `[media]` table; `parity:budgets` checks the script, styles, placeholder and largest-page sizes; `parity:perf` records third-party requests before a click; `parity:screenshots` captures the Omer Hamerman page's player area at 1280×800 and 390×844. Three fixture pages join `scripts/parity/fixtures.txt` and the other cases join the fixture garden.
- Gitpa receives YouTube seeks through `lsg:media-seek`, the event its sticky player already handles for dialogue timestamps.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
