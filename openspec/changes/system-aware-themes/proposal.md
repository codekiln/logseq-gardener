## Why

Generated sites currently use fixed light colors. Readers need the site to follow their system appearance and let them choose Light, Dark or System without losing that choice when they navigate.

## What Changes

- Add shared light and dark palettes that follow the system preference by default, including changes while the page is open.
- Add a labelled appearance selector with Light, Dark and System choices and a persistent browser preference.
- Apply the initial choice before page content appears. Sites that replace navigation can update the Appearance selector without resetting the reader’s choice.
- Provide palette variables for site styles, with readable disclosures, hierarchy, backlinks, focus indicators and native media controls.

## Capabilities

### New Capabilities

- `published-system-themes`: System-aware appearance, persistent visitor choice, accessible controls and host initialization on generated sites.

### Modified Capabilities

None.

## Impact

The shared HTML document, default CSS and a small browser script change in the SDK. Generic JavaScript and publication fixtures cover appearance state and generated markup. Gitpa will consume this behavior and supply its own palette and fixed-player styling in a separate integration. No graph files or publication selection change.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
