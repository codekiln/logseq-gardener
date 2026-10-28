## Purpose

Show Logseq's video embeds and YouTube timestamps in a published garden: click-to-load players of Logseq's size, and timestamp links that seek the video above them, with no third-party request before the reader asks.

## Requirements

### Requirement: Classify video URLs in the SDK

The SDK SHALL turn the argument of `{{video}}`, `{{youtube}}`, `{{vimeo}}` and `{{bilibili}}` into a provider, an embed URL, a watch URL and an optional start time, following Logseq's `get-matched-video` patterns in its order: YouTube (`youtube.com`, `youtu.be`, `y2u.be` with an 11-character id) to `https://www.youtube.com/embed/<id>?enablejsapi=1`; `youtube-nocookie.com` to `https://www.youtube-nocookie.com/embed/<id>`; Loom to `https://www.loom.com/embed/<id>`; Vimeo to `https://player.vimeo.com/video/<id>`; Bilibili to `https://player.bilibili.com/player.html?bvid=<id>&high_quality=1&autoplay=0`, with `&page=` from `?p=`; any other URL to itself. `{{youtube}}` SHALL also accept an 11-character id. The start time SHALL come from `t=` in a `{{video}}` URL's query or fragment, as seconds with or without `s`, or in `XhYmZs` form; `{{youtube}}` SHALL have none. A non-URL argument SHALL be invalid and a missing one empty.

#### Scenario: YouTube with a start time
- **WHEN** the SDK reads `{{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s}}`
- **THEN** the provider is YouTube, the id is `TQsP_PlCY1I` and the start time is 205

#### Scenario: Short link with `?t=`
- **WHEN** the SDK reads `{{video https://youtu.be/yxi1O0T4l6k?t=14}}`
- **THEN** the start time is 14

#### Scenario: Other host
- **WHEN** the SDK reads `{{video https://example.com/film.mp4}}`
- **THEN** the provider is Other and the embed URL is `https://example.com/film.mp4`

### Requirement: Show a click-to-load player area

Each valid video macro SHALL render, inline in its block's content, `div.lsg-video` with the full block width, a height of `min(315px, (viewport width − 96px) × 315/560)` and a `16px 0` margin, holding a play link with an accessible name naming the provider whose `href` is the watch URL. With the media script, activating the link SHALL replace the area with the provider's iframe, adding `autoplay=1` and the start time, with `allowfullscreen`, Logseq's `allow` list `accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope` and a `title` naming the provider. An invalid argument SHALL render `span.warning` titled `Invalid URL` and an empty one `span.warning` titled `Empty URL`, each with the macro text.

#### Scenario: Desktop size
- **WHEN** a reader opens `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs` at 1280×800
- **THEN** the player area is 718px wide and 315px tall, and no literal `{{video` appears

#### Scenario: Phone size
- **WHEN** a reader opens the same page at 390×844
- **THEN** the player area is 165px tall and the page does not scroll sideways

#### Scenario: Click to play
- **WHEN** a reader clicks the player area on the AI Engineer Summit workshops page's first video
- **THEN** a YouTube iframe for `TQsP_PlCY1I` replaces it with `start=205` and `autoplay=1`

#### Scenario: Without JavaScript
- **WHEN** scripts are disabled and a reader activates a player area
- **THEN** the browser opens the video's watch URL

#### Scenario: Empty argument
- **WHEN** a page contains `{{video}}`
- **THEN** it renders `span.warning` titled `Empty URL` with text `{{video}}`

### Requirement: Link timestamps to their video

The SDK SHALL parse `{{youtube-timestamp N}}` as whole seconds, `M:SS` or `H:MM:SS` with minutes and seconds 0–59, and SHALL bind it to the nearest YouTube video before it in its page's outline, depth first. A bound timestamp SHALL render `a.svg-small.youtube-timestamp` with a 12px clock icon and the time as `MM:SS`, or `HH:MM:SS` when the hour is not zero, and an `href` to the video's watch URL at that time, in Logseq's timestamp colour without underline. A timestamp with no video before it SHALL render the icon and time as text. An argument the parser rejects SHALL render nothing and report a diagnostic. In block reference, embed and linked reference copies the timestamp SHALL link to its source page's video and SHALL be bound to no player on the current page.

#### Scenario: Chapter headings
- **WHEN** a reader opens the Omer Hamerman page
- **THEN** its 13 timestamps read `00:00` … `15:26`, each is reachable with Tab, and each links to `6cqVzgVQJfE` at its time

#### Scenario: Hour display
- **WHEN** a page contains `{{youtube-timestamp 3600}}`
- **THEN** it reads `01:00:00`

#### Scenario: Copy in Linked References
- **WHEN** a timestamp from the Omer Hamerman page appears in another page's Linked References
- **THEN** it links to `6cqVzgVQJfE` at its time and clicking it opens that link

### Requirement: Seek the bound video

On a plain click of a bound timestamp, the media script SHALL dispatch a bubbling, cancelable `lsg:media-seek` event carrying the watch URL as `source`, the seconds and the anchor, and SHALL do nothing more if a host cancels it. Otherwise it SHALL prevent navigation and either load the bound placeholder at that time and play, or seek the loaded player to that time and play, without loading YouTube's `iframe_api`. Modified clicks SHALL keep ordinary link behaviour.

#### Scenario: Seek before and after loading
- **WHEN** a reader clicks `04:40` on the Omer Hamerman page before the player has loaded, and again after it has
- **THEN** both times the video plays from 280 seconds and the page stays put

#### Scenario: Same video twice
- **WHEN** a reader clicks a timestamp under the second `TQsP_PlCY1I` video on the AI Engineer Summit workshops page
- **THEN** the second player seeks and the first does not change

#### Scenario: Host takes over
- **WHEN** a listener cancels `lsg:media-seek` and a reader clicks `04:40`
- **THEN** the listener sees `seconds: 280` and no player loads or seeks

### Requirement: Play video files

Image syntax for `.mp4`, `.webm` and `.mov` assets or HTTPS URLs SHALL render `div.asset-ref-wrap` with `data-ext` holding `<video src controls preload="none">`, centred.

#### Scenario: Asset video
- **WHEN** the fixture garden contains `![Film](../assets/film.mp4)`
- **THEN** the page shows `<video controls preload="none">` whose `src` is the copied asset

### Requirement: Show timestamps as text outside the page body

Search records, block reference text and breadcrumbs SHALL show a timestamp as its displayed time.

#### Scenario: Block reference text
- **WHEN** a block references the heading `{{youtube-timestamp 620}} Agent skill`
- **THEN** the reference reads `10:20 Agent skill`

### Requirement: Keep media within budgets

Only pages with a video or a bound timestamp SHALL load `media.js` and `media.css`; other pages SHALL be unchanged. No page SHALL request a third-party host before the reader activates a player or a timestamp. `media.js` SHALL be at most 3 KB, `media.css` at most 1 KB gzip, each placeholder at most 0.6 KB of markup, and `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton` SHALL grow by at most 20 KB gzip over 73,931 bytes. `parity:budgets` SHALL check the sizes in CI and `parity:perf` the third-party requests.

#### Scenario: No YouTube before a click
- **WHEN** a reader loads the Omer Hamerman and AI Engineer Summit workshops pages and waits 40 seconds
- **THEN** no request goes to a YouTube or Google host

#### Scenario: Page without media
- **WHEN** a site is generated for `tmux`
- **THEN** its page links neither `media.js` nor `media.css`
