## Verification

`lsg` now shows a player area where Logseq shows a YouTube player, and every `{{youtube-timestamp}}` reads as Logseq's does and seeks the video above it. No page with a video requests YouTube or Google before the reader clicks. The player area is 10px wider than Logseq's on the Omer Hamerman page at 1280px, because `lsg`'s block is wider there; the height matches.

Measured on 2026-10-28 with the whole encode garden (`mise run parity:build`, run `20261028T044002Z`, 8,697 pages) against Logseq's `gh-pages` build `b491e00` and the values the [analysis](../../../docs/parity/media-embeds.md) measured on it, read in Chromium.

### Omer Hamerman page

`Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs`:

| Check | Logseq (`gh-pages`) | `lsg` |
| --- | --- | --- |
| Player area at 1280×800 | 718×315 iframe, margin `16px 0` | 728×315 placeholder, margin `16px 0` |
| Player area at 390×844 | 239×165 | 276×165, no sideways scroll |
| Timestamps | 13: `00:00` … `15:26` | the same 13, in the same order |
| Timestamp colour, light | `rgb(52, 81, 178)`, no underline | the same |
| Timestamp size in an `h3` | 19.2px text, 12px icon in a 20px box | the same |
| Timestamp focusable with Tab | no (no `href`) | yes; `href` is `https://www.youtube.com/watch?v=6cqVzgVQJfE&t=<N>s` |
| Requests to YouTube or Google before a click | about 1.2 MB | none in 40 seconds (`parity:perf --media`) |

Seeking, checked in the browser on the same page:

- A listener that cancels `lsg:media-seek` saw `seconds: 280` for `04:40`; no player loaded and the page stayed put.
- Clicking `04:40` with the placeholder still showing loaded `https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1&origin=…&autoplay=1&start=280`, and the video played from 280 s (290 s reported 10 seconds later).
- Clicking `10:20` with the player loaded kept the same iframe and moved playback to 623 s 3 seconds later, through `postMessage` commands; `iframe_api` was never requested.

### Other pages

- `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`: two placeholders, 728×315, the first with `data-start="205"` from `&t=205s`. The page has no timestamps, so the repeated-video binding is covered by `media::tests::binds_timestamps_to_the_nearest_youtube_video_before_them` and `tests/site_media.rs` instead. Before a click the page requests only its own remote images (`swyx.io`, `pbs.twimg.com`, `do3z7e6uuakno.cloudfront.net`), as it did before this change.
- `Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture`: `youtu.be/…?t=14` starts at 14 s, which Logseq ignores (Resolved Question 2).
- `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton`: 846 bound timestamps; the page is 88,104 bytes gzip, 14,173 more than the 73,931-byte baseline, within the 20 KB growth budget.
- `tmux` links neither `media.js` nor `media.css`.
- No page in the build shows a literal `{{video` or `{{youtube` outside code.

### Sizes

From `mise run parity:budgets` on the fixture garden: `media.js` 2,403 bytes (budget 3,000), `media.css` 749 bytes gzip (budget 1,000), the largest placeholder 292 bytes (budget 600).

### Not done here

- The Maggie Appleton growth is measured on the whole encode garden, which CI does not build; `parity:budgets` checks the other `[media]` values on the fixture garden, and the growth budget is recorded in `budgets.toml` for the next whole-garden run.
