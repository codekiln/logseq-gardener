## Context

The `podcast-episode` module builds the player once, on the first episode page, from the page's own `<audio>` element. The player holds a title link, a chapter trail and a `role="status"` region that only reports errors. `show(episode)` runs when the player first holds a recording and when another episode takes over, and `showChapter()` redraws the trail when the current chapter changes.

Gitpa's own player, before [codekiln/gitpa#159](https://github.com/codekiln/gitpa/issues/159), also had a `Full recording` link, a status that read `Playing`, `Paused` or `Ended`, `aria-current` on the seek link of the heading playing now, and `aria-label="Full episode recording"` on its `<audio>`.

## Goals / Non-Goals

**Goals:**

- A reader can open or download the recording from the player.
- A screen reader user hears when playback starts, pauses and ends, and can find the chapter playing now.

**Non-Goals:**

- Highlight the chapter playing now on the episode page itself, as gitpa's `is-current-chapter` style did. Issue #588 asks only for the trail.

## Decisions

**The link.** `show(episode)` sets the link's `href` to `episode.source`, the recording URL the entry names, so the link follows the episode the player holds. It has `target="_blank"` and `rel="noopener noreferrer"`, as `lsg` renders external links. A link in the same tab would load the MP3 and stop the player.

**The status.** One listener on `playing`, `pause` and `ended` writes `playbackStatus(audio)`, a pure function that returns `Ended`, `Paused` or `Playing` from the element's `ended` and `paused` flags. Reading the flags, not the event name, gives the right word when the browser fires `pause` just before `ended`. `playing` is used instead of `play`, so the status says Playing when sound starts rather than when it is requested. Error messages keep their existing text and are written by the same `say` function.

**The trail.** `showChapter()` sets `aria-current="true"` on the last link it draws, which is the current chapter; the enclosing chapters before it get none. The trail is already redrawn only when the chapter changes.

**The name.** `build` sets `aria-label="Full episode recording"` on the `<audio>` element, and cleanup puts back the name the page gave the element, the embed's alt text such as `Episode recording`, before the element returns to its page.

## Risks / Trade-offs

[Switching episodes fires `pause` from `audio.pause()` and `audio.load()` before `playing`] → The status briefly reads Paused, then Playing. A polite region announces after the reader's action settles, and the final text is correct.

[A rejected `play()` after a load error writes "Press play in the recording controls to listen." over "The recording could not load."] → This is the existing behaviour and is out of scope here; both messages are errors the reader hears.

## Resolved Questions

### 1 - Does the change need an OpenSpec change, or is it a plain fix?

Decided by the agent without codekiln: an OpenSpec change. The player's behaviour is specified in `podcast-episode-plugin`, and the three items add requirements a reader can observe.

### 2 - Does the `Full recording` link open in the same tab, a new tab, or download?

Decided by the agent without codekiln: a new tab, with `target="_blank"` and `rel="noopener noreferrer"`. A same-tab link would leave the site and stop the player, and `lsg` already opens external links in a new tab. A `download` attribute is ignored for recordings on another host, such as Backblaze, and the browser's own audio page offers a download.

### 3 - Where does `aria-current` go: the trail, or the seek link on the page as in gitpa?

Decided by the agent without codekiln: the current chapter's link in the trail, as #588 asks. The trail is always visible in the player, while the page's heading may belong to another page than the one on screen.

### 4 - What does the status say before the reader starts the recording?

Decided by the agent without codekiln: nothing, unless the recording is already playing when the player is built, in which case it reads Playing. Announcing Paused on every episode page would add speech the reader did not ask for.

### 5 - Is the status word derived from the event or from the element's state?

Decided by the agent without codekiln: from the element's `ended` and `paused` flags, through the pure function `playbackStatus`. The browser fires `pause` right before `ended`; reading the flags gives Ended for both events, and one function covers all three states, so no other word can appear.

### 6 - Can the change be proposed, implemented and synced in one pull request?

Decided by the agent without codekiln: proposed and implemented in one pull request; synced and archived in a follow-up pull request tracked by a cleanup sub-issue, as `lsg/How To/Finish an OpenSpec Change After Merge` describes.

## Open Questions

None.
