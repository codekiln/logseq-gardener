## 1. Build step

- [x] 1.1 Add `ROUTING_SETTINGS` and `leading_reference` to the SDK module `ableton_set`.
- [x] 1.2 Resolve a routing's leading reference in the `ableton-set` build step and write `{title, url, detail}` or text.

## 2. Browser module

- [x] 2.1 Read a routing object in `ableton-set.mjs` and draw the target as a link followed by its detail.
- [x] 2.2 Style the link and its focus outline in `ableton-set.css`.

## 3. Tests and documentation

- [x] 3.1 Cover a resolved target, a target without detail and a withheld target in `tests/site_ableton_set.rs`.
- [x] 3.2 Cover the module's reading of a routing object in `tests/ableton-set.test.cjs`.
- [x] 3.3 Check the drawn link in `scripts/plugins/browser.py`.
- [x] 3.4 Describe the routing value in the `ableton-set` section of `lsg/Ref/First-Party Plugins`.

## 4. Handoff

- [x] 4.1 Run `mise run plugins:browser-test` and `mise run ci`, and open the implementation pull request.
