## Why

Gitpa's own episode player, deleted in [codekiln/gitpa#159](https://github.com/codekiln/gitpa/issues/159) when gitpa adopted `podcast-episode`, showed a link to the full recording, announced Playing, Paused and Ended, and marked the chapter playing now for screen readers. The released player (v0.0.46) has none of these, so a reader cannot open or download the recording from the player, and a screen reader user hears only errors. [#588](https://github.com/codekiln/logseq-gardener/issues/588) asks for the three back.

## What Changes

- The player shows a **Full recording** link to the recording it holds. The link opens in a new tab, as `lsg`'s other external links do, so the player keeps playing.
- The player's live status says **Playing**, **Paused** or **Ended** as the recording changes state, and still reports errors.
- The current chapter in the player's trail carries `aria-current="true"`.
- The player's `<audio>` element has the accessible name `Full episode recording`. When the module is cleaned up and the element returns to its page, the name goes with the player.

Each item uses the episode entry the plugin already has. Pages, page metadata and the build are unchanged, and a site without the plugin is unchanged.

## Capabilities

### New Capabilities

### Modified Capabilities

- `podcast-episode-plugin`: adds the requirement "Show the recording link and playback status".

## Impact

`crates/logseq-gardener-sdk/plugins/podcast-episode/podcast-episode.mjs` and `podcast-episode.css`; unit tests in `tests/podcast-episode.test.cjs`; the browser check in `scripts/plugins/browser.py`, run by `mise run plugins:browser-test`; and the `podcast-episode` section of `lsg/Ref/First-Party Plugins`. The module and stylesheet stay within the plugin's 8 KB gzip budget, and the module still loads only on pages with an episode entry.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
