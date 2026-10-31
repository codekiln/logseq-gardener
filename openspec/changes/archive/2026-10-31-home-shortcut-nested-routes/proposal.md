## Why

`g h` should open the home page, but on [Gitpa](https://codekiln.github.io/gitpa/) it reloads the page the reader is on ([Issue #293](https://github.com/codekiln/logseq-gardener/issues/293)). `lsg-shell.js` opens `index.html` relative to the current page, and Gitpa serves each page from a directory route such as `/gitpa/GitP/A/Session/26/10/29-Thu/`, so `index.html` names that page's own file. Any host that serves pages from nested routes has the same problem.

## What Changes

- `g h` opens the `index.html` in the directory `lsg-shell.js` loads from, which is the site root.
- The search dialog's Go to home command runs the same action, so it opens the site root too.

## Capabilities

### New Capabilities

### Modified Capabilities

- `published-app-shell`: `g h` opens the site root's `index.html` from any page address, including a nested route.

## Impact

`crates/logseq-gardener-sdk/src/site/shell.js` and its tests in `tests/shell.test.cjs`. Sites that `lsg` publishes keep every page at the site root, so `g h` opens the same file there as before. Hosts that move pages into nested routes, such as Gitpa, get a working `g h` without rewriting the script.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
