## Context

The shared HTML document currently embeds fixed light colors. Gitpa adds a Nord light stylesheet and replaces its navigation during internal page changes. Theme selection must therefore live outside replaceable content, with shared behavior and site-owned palette overrides.

Logseq 0.10.6 separates an explicit appearance choice from its system preference and listens for system changes. The relevant local source was checked against [theme state](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/state.cljs#L1249-L1263) and the [system listener](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/ui.cljs#L418-L426).

## Goals / Non-Goals

**Goals:** Follow system appearance by default, preserve visitor overrides, offer accessible controls, and keep shared rendering usable during host navigation.

**Non-Goals:** A custom palette editor and migration of the Encode Garden publishing pipeline are deferred.

## Decisions

### Apply appearance before content paints

Include a small shared script in the document head. It reads a validated Light or Dark override from browser storage and otherwise follows the system preference. It sets the resolved appearance on the document element before body content arrives. CSS also follows the system preference without JavaScript. A script loaded only at the bottom would leave a longer interval with the wrong appearance.

### Keep visitor choice in the browser

Store Light or Dark under `logseq-gardener-theme` in localStorage. System removes the override. The setting is shared by Gardener pages on the same origin; graph content and publication metadata are unchanged. Storage failures retain the current choice in memory and use System on a later load that cannot read a preference. Missing media-query support uses Light.

### Use one labelled native selector

Render an Appearance label and a native select containing System, Light and Dark. The control starts hidden and becomes available when the shared script initializes it. Native focus and keyboard behavior avoid a separate custom menu. A binary switch would make returning to System unclear.

### Separate document state from replaced controls

Keep appearance state on the document element and expose idempotent `window.logseqGardenerInitTheme(root)` for newly inserted controls. The script initializes normal page loads itself. A host such as Gitpa calls the initializer after replacing its navigation; the active choice and media listener remain in the shared controller. Theme changes update controls without rebuilding page content or media players.

### Expose shared palette variables

Use shared CSS variables for text, background, surface, links, borders, warnings and focus indicators. Define readable light and dark defaults and use the resolved color scheme for native controls. Gitpa supplies Nord overrides and its fixed-player styles through the same variables. Existing custom stylesheets remain supported; a stylesheet that hard-codes colors must provide its own dark palette.

## Risks / Trade-offs

- Blocked browser storage prevents persistence across reloads → keep the current choice usable in memory and catch storage failures.
- A host replaces navigation without initialization → document the initializer and cover repeated initialization and host replacement in generic tests and Gitpa integration.
- Custom hard-coded site colors can hide shared dark defaults → document palette variables and verify Gitpa's overrides and player contrast independently.

## Migration Plan

Merge the shared implementation after generic tests and independent review, then pin it in Gitpa and add palette/player integration. Verify appearance choices, navigation, contrast and live publication before completing the change. A renderer-pin rollback restores the previous generated site; visitor preferences do not modify graph files.

## Resolved Questions

### 1 - What is the default when no visitor choice exists?

Follow the system appearance, including changes while the page remains open.

### 2 - How does a visitor return to automatic appearance?

Choose System; the stored override is removed and the current system preference applies immediately.

### 3 - What happens without JavaScript or persistent storage?

CSS follows the system preference without JavaScript and the selector remains hidden. With JavaScript but unavailable storage, the selector works for the current document and its internal navigation.

## Open Questions

None.
