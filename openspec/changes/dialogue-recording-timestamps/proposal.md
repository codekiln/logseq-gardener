## Why

Transcript readers should be able to select a speaker's timestamp and hear that moment in the associated recording. Existing dialogue text already supplies the speaker and time; the published page needs safe links and coordinated playback.

## What Changes

- Recognize speaker-prefixed dialogue timestamps at original outline entries while preserving their text and formatting.
- Bind controls to a single distinct supported recording rendered from that original document.
- Render ordinary recording-fragment links with inline-audio playback and a cancelable event for host players.
- Load the playback helper only on pages containing dialogue controls.

## Capabilities

### New Capabilities

- `published-dialogue-timestamps`: Original-source dialogue recognition, recording association, accessible fragment links and coordinated inline playback.

### Modified Capabilities

None.

## Impact

The shared renderer gains a narrow dialogue recognizer and a small browser helper. Generic Rust and JavaScript fixtures cover recognition boundaries, media identity, copied context, playback state and host interception. Gitpa supplies its persistent-player adapter separately.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
- [Make transcript timestamps seek the associated recording](https://github.com/codekiln/logseq-gardener/issues/90)
