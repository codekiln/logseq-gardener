## 1. Player

- [x] 1.1 Add `playbackStatus(audio)` and write it to the status on `playing`, `pause` and `ended`.
- [x] 1.2 Add the `Full recording` link, updated in `show(episode)`, opening in a new tab.
- [x] 1.3 Set `aria-current="true"` on the current chapter's link in the trail.
- [x] 1.4 Set `aria-label="Full episode recording"` on the player's `<audio>` and put back the page's own name on cleanup.

## 2. Tests and documentation

- [x] 2.1 Add unit tests for `playbackStatus` to `tests/podcast-episode.test.cjs`.
- [x] 2.2 Check the link, the status, `aria-current` and the accessible name in `scripts/plugins/browser.py`.
- [x] 2.3 Describe the link, the status and the accessible names in the `podcast-episode` section of `lsg/Ref/First-Party Plugins`.

## 3. Handoff

- [x] 3.1 Check episode pages of a local encode-garden build with the plugin on, and record the evidence in the pull request.
- [x] 3.2 Run `mise run plugins:browser-test` and `mise run ci`, and open the implementation pull request with a cleanup sub-issue.
