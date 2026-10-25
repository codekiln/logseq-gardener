## Context

The renderer already resolves supported audio through its normal media and asset paths. Dialogue timestamps currently remain plain text. The existing interview convention begins an outline entry with a plain or emphasized speaker, a middle dot, a time, and a dialogue dash. Its syntax is separate from Logseq date nodes, code, and authored links.

## Goals / Non-Goals

**Goals:** Preserve dialogue formatting, associate timestamps with one original recording, support native links and keyboard activation, and let a host player own playback.

**Non-Goals:** Explicit selection among multiple recordings and automatic transcript offset adjustment are deferred.

## Decisions

### Recognize dialogue at the original outline boundary

Inspect the original entry's inline sequence before rendering. Accept a plain speaker or bold/italic plain speaker followed by `· M:SS —` or `· H:MM:SS —`; retain the speaker and speech. Validate the trailing time components and a safe integer total. Written responses, headings, links, code and copied content keep their existing text. This retains parser boundaries that a whole-page text replacement would erase.

### Bind through the existing media renderer

Collect distinct supported original audio sources while rendering. Finalize candidate timestamp ranges after the document is rendered, so recording position does not affect association. Only a unique source supplies links. Existing URL checks and asset resolution determine the destination; fragment replacement preserves its query. Transcluded and backlink audio do not enter the original recording set. A separate explicit source property can support multi-recording pages later.

### Coordinate playback through a cancelable event

Each control remains an ordinary recording link with an integer `#t=` fragment. A small document controller intercepts plain activation and dispatches bubbling, cancelable `lsg:media-seek` with `{source, seconds, anchor}`. A host that cancels the event owns seeking and playback. Otherwise the controller seeks the matching original inline audio, clamps to known duration, and plays during the gesture. A pending seek waits for metadata and keeps only the latest requested position. Native media controls remain the recovery path when play is rejected.

### Load the controller only where controls exist

Publish a standalone helper only when generated dialogue controls exist and reference it only from those pages. The helper installs once per document and uses delegated activation, so replaced content works without repeated listeners. Pages without recognized dialogue retain their existing script payload. A conditional inline loader reads rewritable `link[rel="lsg-dialogue"]` resource metadata and requests the helper after the page load event. It captures the resolved helper URL before a host can replace page metadata. Before loading, or on failure, ordinary recording-fragment links remain usable. This keeps combined search/dialogue source pages, journal pages and feed pages within the existing 16 KB initial-script budget.

## Risks / Trade-offs

- Multiple original recordings make inference ambiguous → retain literal timestamps and document the rule.
- Duration is unavailable during generation → validate syntax statically and clamp against loaded media duration at runtime.
- Host and inline players could both seek → allow the host to cancel the media-seek event before fallback, and prevent the handled source click.
- Copied context could bind to another recording → activate only original source dialogue and original audio.

## Migration Plan

Regenerate sites with the shared renderer. Gitpa can handle the event through its persistent player when it consumes the renderer. Its existing ordinary media-fragment links remain usable. Reverting the renderer removes automatic dialogue controls without changing source files.

## Resolved Questions

### 1 - What happens on pages with several recordings?

Automatic controls require one distinct supported original recording. Multiple originals retain literal dialogue times until an explicit source-selection extension is designed.

### 2 - Which timeline does a timestamp use?

The authored timestamp is an absolute position in the selected recording. Generation does not infer trims or measure remote duration.

### 3 - How can a persistent player take control?

Cancel `lsg:media-seek` and handle the supplied recording source and time. The shared helper then performs no inline playback.

## Open Questions

None.
