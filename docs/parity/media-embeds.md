# Video embeds and YouTube timestamps

Logseq turns `{{video URL}}` into a YouTube, Vimeo, Bilibili or Loom player in an iframe, with a start time when a YouTube URL has `&t=`. `{{youtube-timestamp N}}` becomes a clock icon and a time such as `04:40`. Clicking it seeks the nearest YouTube player above it and starts playback. `lsg` shows every one of these macros as literal code, for example `{{video https://www.youtube.com/watch?v=6cqVzgVQJfE}}`, with a build diagnostic. The encode garden uses `{{video}}` 118 times in 113 files and `{{youtube-timestamp}}` 2,727 times in 54 files, so every YouTube note page in the `lsg` build currently shows source text where Logseq shows a player.

The gap is the player, the timestamp link and the seek. Two parts need decisions in the proposal. Logseq reads a start time only from `&t=<digits>`, so it ignores `?t=14` on `youtu.be` links, which 4 garden pages use. Logseq's timestamp is an anchor with no `href`: it cannot be reached with the keyboard and does nothing without JavaScript. `lsg` can keep Logseq's look and still give the timestamp a real link to the video at that time.

Logseq loads every YouTube player when the page renders, even 5,849px below the fold. One player costs about 1.2 MB of transfer from YouTube and Google and runs YouTube's player script. `lsg` should publish a click-to-load placeholder with the same size and position, and load the player only when a reader clicks it or a timestamp. Pages then keep their current first paint. Classifying video URLs, parsing timestamps and binding each timestamp to its video are SDK work, done at build time; the publisher renders the placeholder, the timestamp link and a small seek script that reuses the `lsg:media-seek` event from [#90](https://github.com/codekiln/logseq-gardener/issues/90).

Measured on 2026-10-26 against the `gh-pages` build of the encode garden (`gh-pages` commit `8132c95`, served locally, light theme, 1280×800 and 390×844, Chromium with network access to YouTube) and an `lsg` build of garden commit `083106d` from `lsg` `48fcc54` (`mise run parity:build`, run `20261027T002614Z`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from [Prototype](../logseq-reference/Prototype.md). The live build is older than the checkout in one place: its YouTube iframe has neither `picture-in-picture; web-share` in `allow` nor `referrerpolicy`. The Prototype board differs from the live build in several places, listed where they occur: it draws a 560px 16:9 placeholder, gives the timestamp the title `Seek video`, shows a toast and a `Seeked to` badge, and limits seeking to videos on the same page.

### `{{video URL}}`

[`macro-video-cp`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1328-L1377) matches the URL against [`get-matched-video`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/util/text.cljs#L12-L23) (YouTube, Loom, Vimeo, Bilibili, in that order):

| URL | Player |
| --- | --- |
| `youtube.com`, `youtu.be`, `y2u.be` with an 11-character id | The YouTube player below, with `start` from `&t=` |
| `youtube.com` with an id of another length | An iframe whose `src` is the URL as written |
| `youtube-nocookie.com` | `https://www.youtube-nocookie.com/embed/<id>`, no start time, no seek |
| `loom.com/share/…` or `/embed/…` | `https://www.loom.com/embed/<id>` |
| `vimeo.com`, `player.vimeo.com` | `https://player.vimeo.com/video/<id>` |
| `bilibili.com/video/<bvid>` | `https://player.bilibili.com/player.html?bvid=<id>&high_quality=1&autoplay=0`, plus `&page=` from `?p=` |
| Any other URL | An iframe whose `src` is the URL as written |
| Not a URL | `span.warning.mr-1`, title `Invalid URL`, text `{{video …}}` |
| No argument | `span.warning.mr-1`, title `Empty URL`, text `{{video}}` |

- **Start time.** Only `&t=(\d+)` is read ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1355-L1358)). `watch?v=TQsP_PlCY1I&t=205s` gives `start=205`. `youtu.be/yxi1O0T4l6k?t=14` and `?t=1m30s` give no start time (measured on `Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture`).
- **Non-YouTube iframe.** `allowfullscreen`, `allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope"`, `framespacing="0"`, `frameborder="no"`, `border="0"`, `scrolling="no"`, `width` and `height` as for YouTube; Bilibili's height uses 360/560. Measured on `Atlassian/Blog/25/05/Introducing Atlassian's Remote Model Context Protocol`: an `.mp4` URL on another host becomes an iframe pointing at the file, 749×315.
- **Paste.** Pasting a matching video URL in the editor wraps it in `{{video …}}` ([`paste.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/paste.cljs#L49-L57)). A bare video URL in a page renders as an ordinary link.

### The YouTube player

[`youtube-video`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L45-L70) renders:

```html
<iframe id="youtube-player-6cqVzgVQJfE" allowfullscreen=""
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope"
  frameborder="0" src="https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1"
  height="315" width="560"></iframe>
```

- **Wrapper.** The iframe sits inline in the block's content (`div.inline` or `div.is-paragraph` inside `.block-body`), with no wrapper of its own.
- **Size.** `width` is `min(window width − 96, 560)` and `height` is `width × 315 / 560`, both fixed when the block renders. Logseq's base styles then set `iframe { width: 100%; margin: 1rem 0 }` ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L129-L132)), so the iframe fills the block's width and keeps the attribute height. Measured: 718×315 on `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs` at 1280px (749×315 one level shallower), not 560×315 and not 16:9; 239×165 at 390px (attributes 294×165). Margin `16px 0`.
- **API.** Mounting the first player inserts `https://www.youtube.com/iframe_api` and registers a `YT.Player` for each iframe in `:youtube/players`, keyed by video id ([`youtube.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L12-L43)). The API adds `title="<video title>"` to the iframe.
- **Same video twice.** Both iframes get the same `id` (measured on `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`, `youtube-player-TQsP_PlCY1I` twice), and the second registration replaces the first in the player map.
- **Loading.** No `loading="lazy"` and no placeholder: the iframe loads as soon as Logseq renders its block.

### `{{youtube URL}}`, `{{vimeo URL}}`, `{{bilibili URL}}`

- `{{youtube}}` takes an 11-character id or a URL matching `youtube-regex` and renders the same player with no start time, even when the URL has `&t=` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1488-L1495)).
- `{{vimeo}}` and `{{bilibili}}` render the iframes from the table above ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1290-L1326)); Bilibili's height is at least 500px.
- The encode garden uses none of the three.

### `{{youtube-timestamp N}}`

- **Argument.** Whole seconds (`620`), `M:SS` or `H:MM:SS`, with minutes and seconds 0–59 ([`parse-timestamp`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L124-L140)). Any other argument renders nothing ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1497-L1500)).
- **Display.** Two-digit `MM:SS`, with `HH:` only when the hour is not zero: `0` → `00:00`, `41` → `00:41`, `620` → `10:20`, `3600` → `01:00:00` ([`seconds->display`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L72-L83)). Measured on the Omer Hamerman page: `00:00`, `00:41`, `01:46` … `15:26`.
- **Markup.** `<a class="svg-small youtube-timestamp">` holding the 20×20 `clock` icon (`svg.h-5.w-5`, `fill="currentColor"`) and the time ([`youtube.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L103-L111)). No `href`, no `title`, no role.
- **Styles.** `.svg-small svg { transform: scale(0.6); display: inline }` ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L557-L560)): the icon draws at 12×12. The link inherits the block's font size (16px; 19.2px in an `h3`), has no underline and a pointer cursor. Measured colour `rgb(52, 81, 178)`, different from page references (`#106ba3`); the rule that sets it was not traced and may come from the garden's `custom.css`.
- **Keyboard.** An anchor without `href` is not focusable; measured `focus()` leaves it unfocused, so keyboard readers cannot seek.
- **Seek.** A click stops the event and calls `seekTo(seconds, true)` on the player of the last YouTube iframe in the document that comes before the link ([`get-player`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/video/youtube.cljs#L85-L101)). The rule is document order, not the outline or the page: a timestamp in Linked References seeks whatever YouTube iframe precedes it on screen, and with a repeated video id it seeks the last registered player. If no YouTube player precedes the link, the click does nothing. Measured: clicking `04:40` before playing put the video at 282.9 s and playing.
- **Elsewhere.** In block reference text, breadcrumbs and search results the timestamp is plain `MM:SS` text ([Transclusion](../logseq-reference/Transclusion.md)).
- **Editing only.** `mod+shift+y` and the `Embed YouTube timestamp` command insert the current player time ([`commands.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/commands.cljs#L669-L680)). Publishing has no editor.

### Video files

- `![label](../assets/x.mp4)`, `.webm` and `.mov` render `div.asset-ref-wrap` with `data-ext` and `<video src controls>` ([`media-link`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1047-L1074), [`asset-reference`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L709-L741)). `.flv`, `.avi` and `.mkv` go to the image path instead. Base styles centre `video` with auto side margins.
- `{{youtube-timestamp}}` does not seek a `<video>` element.
- The encode garden has no video file links.

### Performance in Logseq

Measured with an empty cache, all third-party requests recorded in a HAR file, 40 seconds after load:

- `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs`, one player 1,341px down the page: 1,201 KB transferred from `www.youtube.com` (1,054 KB), `i.ytimg.com` (122 KB, the `maxresdefault.webp` poster) and Google hosts, plus YouTube's logging requests. Third-party script decoded to 3.5 MB, almost all YouTube's player.
- `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`, the same video twice at 5,849px and 6,268px: both players loaded without scrolling. The second added little because it reused the first player's script.
- These costs come after the graph itself: about 20 s to the title on the desktop profile ([Parity performance](performance.md)).

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/render.rs`, `dialogue.rs`, `dialogue.js`, the [published inline markup spec](../../openspec/specs/published-inline-markup/spec.md), the [local static publishing spec](../../openspec/specs/local-static-publishing/spec.md) and the [published dialogue timestamps spec](../../openspec/specs/published-dialogue-timestamps/spec.md), checked against the whole-garden build:

- **Macros are literal code.** `macro_fallback` writes `video`, `youtube`, `youtube-timestamp` and `tweet` as `<code>{{name args}}</code>` and reports `Unsupported macro (including embeds and queries); preserved as literal text`. The inline markup spec keeps this rendering until the issue that owns each macro changes it. `{{vimeo}}` and `{{bilibili}}` are not in that list, so they would render as `span.warning` chips. The build's page bodies hold 127 literal `{{video` and 2,786 literal `{{youtube-timestamp` (more in linked reference fragments).
- **Timestamp headings.** On the Omer Hamerman page each chapter heading reads `{{youtube-timestamp 41}} Introducing tuicr`, in code style.
- **Video files.** `.mp4` and `.webm` assets are copied into the build, but image syntax for them renders a plain link (`<a href="assets/film.mp4">Film</a>`). Only MP3, WAV and OGG get `<audio controls preload="none">`.
- **Dialogue seeking from #90.** For pages with exactly one original audio recording, `lsg` turns `speaker · MM:SS — text` lines into `a.dialogue-timestamp` links with `href="<recording>#t=<seconds>"` and `data-lsg-dialogue-time`. `dialogue.js` (2,430 bytes, loaded only by the 3 pages that need it) dispatches a cancelable `lsg:media-seek` event with `{source, seconds, anchor}`; Gitpa's sticky player cancels it and plays the moment itself, and otherwise the script seeks the page's `<audio data-lsg-recording>`. It does not know YouTube, and `{{youtube-timestamp}}` does not produce these links.
- **Sizes.** The Omer Hamerman page is 43,008 bytes (11,997 gzipped). The largest timestamp page, `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton` (846 timestamps), is 447,528 bytes (73,931 gzipped), already within the 100 KB first-paint budget.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **YouTube player from `{{video}}` and `{{youtube}}`.** A YouTube URL with an 11-character id renders a player for `https://www.youtube.com/embed/<id>` with `allowfullscreen`, the measured `allow` list and an accessible title. Check: the Omer Hamerman page shows a player area for `6cqVzgVQJfE` and no literal `{{video`.
- 2. **Size and position.** The player area takes the block's full width, 315px tall at desktop width and `min(window width − 96, 560) × 315/560` on phones, with a `16px 0` margin, as Logseq draws it. The proposal decides whether to match Logseq's fixed height or keep the area 16:9, which Logseq's own 560×315 attributes intend. Check: at 1280×800 the area on the Omer Hamerman page is 718px wide and 315px tall (or the decided ratio); at 390×844 it fits without sideways scroll.
- 3. **Start time.** `&t=205s` on a `{{video}}` YouTube URL gives `start=205`; `{{youtube}}` gives no start time. The proposal decides whether to also honour `?t=` and `1m30s` forms, which Logseq ignores. Check: the AI Engineer Summit page has players starting at 205 and 806; `Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture` starts at 0, or at 14 if the proposal chooses to differ.
- 4. **Other providers and fallbacks.** Loom, Vimeo, `youtube-nocookie.com` and Bilibili URLs give the iframes in the table above; other HTTPS URLs give a plain iframe, as Logseq does, or a link if the proposal decides that framing arbitrary hosts is unsafe; a non-URL or empty argument gives the `Invalid URL` or `Empty URL` warning chip. `{{vimeo}}` and `{{bilibili}}` work the same as their `{{video}}` forms. Check: a fixture-garden page under `tests/fixtures/parity-garden/` with one of each produces the listed `src` values; the Atlassian page shows its `.mp4` URL in the decided form.
- 5. **Timestamp link.** `{{youtube-timestamp N}}` renders `a.svg-small.youtube-timestamp` with the 12px clock icon and Logseq's time format; an argument `parse-timestamp` rejects renders nothing and reports a diagnostic. Unlike Logseq, the link has an `href` to the video at that time (for example `https://www.youtube.com/watch?v=<id>&t=<N>s`), so it is focusable and works without JavaScript. Check: on the Omer Hamerman page the 13 links read `00:00` … `15:26`, each is reachable with Tab, and `{{youtube-timestamp 3600}}` reads `01:00:00`.
- 6. **Binding a timestamp to its video.** The build binds each timestamp to the nearest YouTube video before it in its own page's outline, which matches Logseq on every garden page: each of the 54 files with timestamps has its video above them, except three that only mention the macro inside inline code. In reference and transclusion copies the timestamp keeps the binding to its source page's video, where Logseq seeks whatever video is above it on screen. A timestamp with no video before it stays a plain time, as Logseq's click does nothing. Check: on the AI Engineer Summit page, a timestamp under the second player seeks the second player even though both have the same id; a copy of an Omer Hamerman timestamp in Linked References links to `6cqVzgVQJfE`.
- 7. **Seek.** Clicking a timestamp loads the player if it is not loaded, seeks it to N seconds and plays, without leaving the page. It first dispatches the existing `lsg:media-seek` event with the YouTube source, so a host such as Gitpa's sticky player can take over; modified clicks keep ordinary link behaviour. Check: on the Omer Hamerman page, clicking `04:40` before and after the player has loaded leaves the video playing at 280 s; a listener that cancels `lsg:media-seek` sees `seconds: 280` and no seek happens.
- 8. **Video files.** Image syntax for local or HTTPS `.mp4`, `.webm` and `.mov` renders `div.asset-ref-wrap` with `<video controls>`, extending the existing audio rule. Check: the fixture garden's `![Film](../assets/film.mp4)` renders `<video controls>` with the copied asset as `src`.
- 9. **Plain-text contexts.** Search snippets, block reference text and breadcrumbs show a timestamp as `MM:SS` text. Check: a block reference to the `{{youtube-timestamp 620}} Agent skill` heading reads `10:20 Agent skill`.
- 10. **Performance.** No page loads anything from YouTube until a reader clicks a player or a timestamp. Check: loading the Omer Hamerman and AI Engineer Summit pages makes no request to a YouTube or Google host, and `parity:perf` shows their first-paint values within budget.

## Performance

Logseq's eager iframes cost about 1.2 MB and YouTube's player script per page with a video, whether or not the reader watches it. `lsg` should not copy that ([#106](https://github.com/codekiln/logseq-gardener/issues/106)):

- **Placeholder first.** The publisher writes a placeholder of the measured size with a play button and the video's accessible title, as a link to the video on YouTube. Without JavaScript the link opens YouTube. With JavaScript, a click replaces the placeholder with the iframe and `autoplay=1`, so the reader clicks once, as with Logseq's player.
- **Poster image.** A poster from `i.ytimg.com` makes the placeholder look like a player but costs one image request to YouTube per video and tells YouTube about the visit. The proposal decides between a poster with `loading="lazy"` and a plain dark box; measure both.
- **Seeking without the full API.** Logseq loads `iframe_api` to call `seekTo`. With `enablejsapi=1` the page can send the same command to the iframe with `postMessage`, so `lsg` needs no YouTube script of its own. Before the player loads, a timestamp click can load it with `start=N&autoplay=1` directly. The proposal compares both against reloading the iframe's `src`.
- **One small script, only where needed.** Like `dialogue.js`, the media script loads only on pages that have a video or a timestamp, and shares `lsg:media-seek` with it. Pages without media are unchanged.
- **Page size.** Each timestamp grows from a `<code>` literal to a link with an icon. On the Maggie Appleton page that is 846 links; define the clock icon once (CSS mask or `<use>`), not inline per link, so the page stays within its gzip budget.

Budget rows that apply:

- `[desktop]` and `[phone]`: `data_before_title_kb`, `script_before_title_kb`, `title_ms` and `peak_heap_mb` stay within budget for every fixture page, with no third-party requests before a click.
- A new `[media]` table in the proposal, for example `script_kb` for the media script, `placeholder_kb` for the markup each video adds, and `timestamp_growth_gzip_kb` for the largest timestamp page (73,931 bytes gzipped today).

## Where the work belongs

- **SDK:** parsing `{{video}}`, `{{youtube}}`, `{{vimeo}}`, `{{bilibili}}` and `{{youtube-timestamp}}` arguments into a provider, id, embed URL and start time, or seconds; binding each timestamp to the video before it in its source page's outline; giving each video a stable identity on its page so two players of one video stay distinct; and the plain-text form of a timestamp used by search, block reference text and breadcrumbs. The CLI and the language server can then answer "which videos and timestamps does this page have" from the same index.
- **Publisher:** the placeholder, iframe and `<video>` markup and styles, the timestamp link and icon, the media script and its use of `lsg:media-seek`, the diagnostics, and the `[media]` budget rows.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs` | The Prototype board's video page: one `{{video}}`, 13 timestamps in `h3` chapter headings | 718×315 player, `00:00` … `15:26` links that seek it | Literal `{{video …}}` and `{{youtube-timestamp …}}` |
| `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025` | The same video twice, with `&t=205s` and `&t=806s`, far below the fold | Two players with `start`, the same `id`, both loaded at once | Literal code |
| `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton` | 846 timestamps under one video: the size budget | not measured | 447,528 bytes, 73,931 gzipped |
| `Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture` | `youtu.be/…?t=14`, which Logseq ignores | Player with no start time | Literal code |
| `Atlassian/Blog/25/05/Introducing Atlassian's Remote Model Context Protocol` | `{{video}}` with an `.mp4` URL on another host | Plain iframe pointing at the file | Literal code |

The first three are added to [`scripts/parity/fixtures.txt`](../../scripts/parity/fixtures.txt). The other two, and the providers the garden does not use (Vimeo, Bilibili, Loom, `youtube-nocookie.com`, `{{youtube}}`, invalid and empty URLs, video files), belong in the fixture garden under `tests/fixtures/parity-garden/`. `parity:screenshots` shows the player area; the seek checks need a click script in the Apply step.

## Out of scope

- `{{tweet}}` and `{{twitter}}`. Logseq loads Twitter's widget script and draws a 550px `div.twitter-tweet` with an iframe from `platform.twitter.com` (measured on `Person/Andrej Karpathy/X/26/04/AI Psychosis`). The garden uses it 8 times. No parity issue owns it yet; it needs the same click-to-load decision and a separate issue.
- Gitpa's sticky player and persistent playback across navigation, which #90 and Gitpa own. This issue only makes YouTube seeks dispatch `lsg:media-seek`.
- Seeking audio and video files from `{{youtube-timestamp}}`, which Logseq does not do; dialogue timestamps already cover audio ([published dialogue timestamps](../../openspec/specs/published-dialogue-timestamps/spec.md)).
- The editor's timestamp insertion (`mod+shift+y`, `Embed YouTube timestamp`) and paste-to-macro conversion.
- Videos inside slide decks, which belong with [Issue #121 — Add View as slides](https://github.com/codekiln/logseq-gardener/issues/121).
- How block embeds and references draw their frame around a copied video, which belongs with [Issue #114 — Match Logseq's block references and block and page embeds](https://github.com/codekiln/logseq-gardener/issues/114); this issue decides only what a copied timestamp seeks.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
