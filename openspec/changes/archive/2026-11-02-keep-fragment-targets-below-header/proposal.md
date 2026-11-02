## Why

On a published site, a link to a fragment such as `#source-block-6172` scrolls its target to the top of the window, where the sticky 48px header covers the target's first line ([Issue #316 — Keep fragment targets below the sticky header](https://github.com/codekiln/logseq-gardener/issues/316)). Source-block links, note results from search and any heading a host links to land this way. No stylesheet sets `scroll-padding-top`, so both the browser's own fragment scrolling and the `scrollIntoView()` call in `fragments.js` align the target with the top of the viewport.

## What Changes

- Every generated page sets `scroll-padding-top` on `html` to 56px: the 48px header plus an 8px gap. The browser's fragment scrolling and `scrollIntoView()` then stop with the target's first line below the header, at every viewport width.
- The rule ships in the inline shell styles, so no file or script is added.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-app-shell`: the requirement "Show Logseq's header on every page" adds that a fragment target scrolls into view below the header.

## Impact

- `crates/logseq-gardener-sdk/src/site/shell.rs`: one CSS rule in `STYLES`, about 30 bytes per page.
- Tests: `tests/site_shell.rs` asserts the rule.
- Docs: lsg/Ref/Host Integration API, in `docs/pages/lsg___Ref___Host Integration API.md`, tells host authors that fragment scrolling already clears the header.
- Hosts: Gitpa can remove the `scroll-padding-top` rule it added to `site/player.css` in [codekiln/gitpa#129 fix: play chapter timestamps in the pinned player](https://github.com/codekiln/gitpa/pull/129).

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
