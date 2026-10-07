## Context

The renderer already distinguishes selected page links, raster images, and other local files. All remote image syntax currently becomes a label. Current public GitP session pages provide direct HTTPS artwork and recording links; older Gitpa Ceremony pages reference local recordings.

## Goals / Non-Goals

**Goals:** Play selected recordings, display selected artwork, preserve accessible download links, and keep source gardens and publication selection intact.

**Non-Goals:** Broader embed expansion, homepage query evaluation, and graph alias lookup remain separate functional work.

## Decisions

### Render media through the existing link handler

Use a shared plain-label helper for images and audio. MP3, WAV, and OGG image syntax produces an audio element with controls and `preload="none"`, an accessible name, and a visible download link. The local asset resolver continues to validate and copy files. Ordinary links retain their existing behavior. Browser codec support varies, so the download link remains available even when playback fails.

### Permit HTTPS media with supported path extensions

Require an HTTPS authority, reject controls, and classify the URL path before query and fragment suffixes. Escape URL attributes and source labels. HTTPS media remains remote: the renderer emits a URL without making a network request. Source-authored HTTP links continue to work as links; remote media uses HTTPS. Arbitrary remote downloads would add network dependence and move media copying into generation.

### Keep publication selection before rendering

Reuse selected-document planning and private-page withholding. Media URLs in excluded or withheld pages never reach the renderer. Nested labels remain plain and cannot request additional media.

## Risks / Trade-offs

Remote recordings require network access when the visitor listens. Remote artwork availability and browser codec support depend on the source host and recording; the site retains descriptive text and an audio download link. An OGG container can hold different codecs, so the renderer supplies no guessed MIME codec declaration.

## Migration Plan

Land after publishing integration. Regenerate into a fresh destination to use the new rendering; existing generated sites remain unchanged. Sync this delta after the local-static-site base requirements enter main specs.

## Resolved Questions

### 1 - Should generation fetch remote media?

Generation emits selected HTTPS references. The browser fetches the media while viewing the generated site.

## Open Questions

None.
