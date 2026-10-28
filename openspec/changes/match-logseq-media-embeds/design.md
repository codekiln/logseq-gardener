## Context

The [video embeds and YouTube timestamps analysis](../../../docs/parity/media-embeds.md) measured Logseq's `gh-pages` build of the encode garden (`8132c95`) and an `lsg` build from `48fcc54`. Logseq renders `{{video}}` as an iframe inline in the block, sized `min(window − 96, 560) × 315/560` by attribute and stretched to the block's width by `iframe { width: 100% }`. Mounting the first YouTube player inserts `iframe_api`; a click on `{{youtube-timestamp N}}` calls `seekTo` on the last YouTube player before it in the document.

`lsg` writes `video`, `youtube`, `youtube-timestamp` and `tweet` as literal `<code>`, and `vimeo` and `bilibili` as unknown-macro warning chips (`macro_fallback` in `site/render.rs`). [#90](https://github.com/codekiln/logseq-gardener/issues/90) added dialogue timestamps for audio: `dialogue.js` (2,430 bytes) loads only on pages that need it and offers a cancelable `lsg:media-seek` event with `{source, seconds, anchor}` before seeking the page's audio. Gitpa's sticky player cancels that event and plays the moment itself.

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) asks for Logseq's look and behaviour without its cost: graph work at build time, first paint independent of garden size, browser features loaded only when used, and bounded phone memory.

## Goals / Non-Goals

**Goals:**

- Every `{{video}}`, `{{youtube}}`, `{{vimeo}}` and `{{bilibili}}` in the garden shows a player area of Logseq's size and position, playing the video Logseq plays from the start time the author wrote.
- Every `{{youtube-timestamp}}` reads as Logseq's does and seeks the video Logseq's would on that page, with the keyboard and without JavaScript too.
- Pages with media keep their first paint: no third-party request and no YouTube script before the reader clicks.

**Non-Goals:**

- `{{tweet}}` and `{{twitter}}`, which need the same click-to-load treatment for Twitter's widget and have no parity issue yet.

## Decisions

### The SDK owns parsing, classification and binding

A new SDK module, `media`, turns macro arguments into values:

- `VideoEmbed { provider, video_id, embed_url, watch_url, start }` for `{{video}}`, `{{youtube}}`, `{{vimeo}}` and `{{bilibili}}`, with `provider` one of `YouTube`, `YouTubeNoCookie`, `Loom`, `Vimeo`, `Bilibili` or `Other`, following `get-matched-video`'s patterns and order. `{{youtube}}` accepts an 11-character id as well as a URL, and has no start time, as in Logseq.
- `Timestamp { seconds }` from `{{youtube-timestamp}}`, accepting whole seconds, `M:SS` and `H:MM:SS` with minutes and seconds 0–59, as `parse-timestamp` does.
- A per-page index: each video gets its outline-order number on its page, and each timestamp the number of the nearest YouTube or `youtube-nocookie.com` video before it in outline order (depth first, parents before children), or none.
- `Timestamp::display()` gives `MM:SS`, or `HH:MM:SS` when the hour is not zero, used by the link, search text, block reference text and breadcrumbs.

The publisher calls these and renders. The SDK holds no HTML.

### Placeholder, then the provider's iframe

Each video renders:

```html
<div class="lsg-video" data-lsg-video="0" data-provider="youtube"
     data-embed="https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1">
  <a class="lsg-video-play" href="https://www.youtube.com/watch?v=6cqVzgVQJfE"
     aria-label="Play YouTube video">…play icon…</a>
</div>
```

The box is `width: 100%; height: min(315px, calc((100vw - 96px) * 315 / 560)); margin: 16px 0`, dark, with a centred play button. Without JavaScript the link opens the video on its site. With `media.js`, a click replaces the box with the iframe, adding `autoplay=1`, `start=<s>` when the embed has one, and for YouTube `enablejsapi=1` and `origin`. The iframe carries Logseq's `allow` list, `allowfullscreen` and `title` (`YouTube video`, `Vimeo video`, and so on). `Other` URLs keep Logseq's plain iframe pointing at the URL as written, behind the same placeholder.

### Start time from `t=` anywhere in the URL

The SDK reads `t=` from the query or the fragment, as seconds (`205`, `205s`) or `XhYmZs`. YouTube honours all of these when the link is opened; the author wrote them to start there. This changes the start of 4 garden pages that use `youtu.be/…?t=`.

### The timestamp is a real link

```html
<a class="svg-small youtube-timestamp" data-lsg-video="0"
   href="https://www.youtube.com/watch?v=6cqVzgVQJfE&amp;t=280s">04:40</a>
```

The clock icon is drawn once in `media.css` with a CSS mask on `::before`, 12×12 as Logseq's scaled 20×20 icon, in `currentColor`. The link uses Logseq's measured colour `rgb(52, 81, 178)` in light mode and the theme's link token in dark, with no underline. A timestamp bound to no video renders `span.youtube-timestamp` with the icon and time, which does nothing, as Logseq's click does nothing. In reference and embed copies the timestamp keeps the `href` of its source page's video and has no `data-lsg-video`, so it opens that video at that time.

### Seeking without YouTube's script

`media.js` handles plain clicks on `[data-lsg-video]` links and leaves modified clicks to the browser. It dispatches `lsg:media-seek` with `{source: watch URL, seconds, anchor}`; if a host cancels it, the script stops. Otherwise, if the bound video is still a placeholder, the script loads it with `start=<seconds>&autoplay=1`; if loaded, it sends `seekTo` and `playVideo` commands with `postMessage`, which `enablejsapi=1` allows. `iframe_api` is never loaded.

### Media code loads only where needed

`media.js` and `media.css` are written at the site root. Pages with at least one video or bound timestamp link to them with `defer`, as `dialogue.js` is linked. Pages without media are byte-for-byte unchanged.

### Video files

`![label](x.mp4)`, `.webm` and `.mov` render `div.asset-ref-wrap[data-ext]` holding `<video src controls preload="none">`, centred, extending the existing audio rule. `preload="none"` keeps first paint unchanged, as for audio.

## Risks / Trade-offs

- A placeholder without a poster looks less like Logseq's player before the click. → It keeps the size and position, so the page reads the same, and it is what keeps YouTube off the page until the reader asks.
- Honouring `?t=` differs from Logseq on 4 pages. → The author's link asked for that time; a reader opening the same link on YouTube gets it.
- `postMessage` commands depend on YouTube's iframe accepting `enablejsapi=1`. → The fallback reloads the iframe with the new `start`, and the timestamp's `href` still works.
- 846 timestamp links grow the largest timestamp page. → The icon is CSS, so each link adds only its `href` and attributes; `[media]` caps the growth.

## Performance budgets

New `[media]` table in `scripts/parity/budgets.toml`:

- `script_kb = 3`: `media.js`, uncompressed.
- `styles_gzip_kb = 1`: `media.css`.
- `placeholder_kb = 0.6`: the markup each video adds.
- `timestamp_page_baseline_gzip_kb = 73.931` and `timestamp_page_growth_gzip_kb = 20`: `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton`, keeping it under the 100 KB first-paint budget.
- `third_party_requests_before_click = 0`, checked by `parity:perf` on the Omer Hamerman and AI Engineer Summit pages.

The `[desktop]` and `[phone]` first-paint budgets apply unchanged to every fixture page. Peak phone heap is unchanged before a click; after a click it is YouTube's, which `lsg` does not control and Logseq pays for every video.

## Resolved Questions

### 1 - Does the player area keep Logseq's fixed 315px height or a 16:9 box?

Decided by the agent without codekiln: Logseq's: full block width and `min(viewport − 96, 560) × 315/560` tall, which is 315px at desktop widths. Parity is the goal, and this is what readers of the Logseq build see (718×315 on the Omer Hamerman page).

### 2 - Does `lsg` honour `?t=` and `1m30s` start times, which Logseq ignores?

Decided by the agent without codekiln: yes. The start time is the author's, written in a form YouTube honours; Logseq ignores it only because its regex reads `&t=<digits>`. The look and the player are unchanged.

### 3 - Does a URL on an unknown host become an iframe or a link?

Decided by the agent without codekiln: an iframe, as Logseq does, behind the same click-to-load placeholder. The garden's only case is an `.mp4` on another host, which plays in the iframe; the placeholder means no other host loads until the reader clicks.

### 4 - Does the timestamp have an `href`?

Decided by the agent without codekiln: yes, to the video's watch URL with `&t=<N>s`. It keeps Logseq's look and adds keyboard access and a working link without JavaScript, which Logseq's `href`-less anchor lacks.

### 5 - Does the placeholder show YouTube's poster image?

Decided by the agent without codekiln: no, a dark box with a play button. A poster is one request to `i.ytimg.com` per video that tells YouTube about the visit, which breaks the goal of no third-party request before a click.

### 6 - How does a timestamp seek a loaded player?

Decided by the agent without codekiln: by `postMessage` commands to the iframe, with `enablejsapi=1`, and by loading the player with `start=<N>&autoplay=1` when it is still a placeholder. This needs no YouTube script; reloading the iframe's `src` would restart buffering on every click.

### 7 - What does a timestamp seek when the same video appears twice?

Decided by the agent without codekiln: the nearest video before it in the outline, by its index on the page. Logseq gives both iframes the same `id` and seeks the last registered one, which is a defect; the outline order is what Logseq's document-order rule intends.

### 8 - What does a timestamp copied into Linked References or an embed do?

Decided by the agent without codekiln: it links to its source page's video at that time and is not bound to any player on the current page. Logseq seeks whatever video sits above it on screen, which is usually the wrong one.

### 9 - Do video files get `preload="none"`?

Decided by the agent without codekiln: yes, as the existing audio rule does. Logseq's `<video>` uses the browser default, which can download the file before the reader plays it.

### 10 - Is the media script separate from `dialogue.js`?

Decided by the agent without codekiln: yes, `media.js`, sharing only the `lsg:media-seek` event. The two load on different pages (3 dialogue pages, 113 video pages), so merging them would make each page load code it does not use.

### 11 - What lives in the SDK and what in the publisher?

Decided by the agent without codekiln: the SDK parses the macros, classifies providers, computes embed and watch URLs and start times, indexes videos on a page, binds timestamps, and gives the timestamp's text form. The publisher writes the placeholder, iframe and video markup, the link, the styles and the script. The CLI and language server can then use the same media operations, as the shared-engine comment on #106 asks.
