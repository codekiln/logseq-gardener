## Why

Garden owners can publish podcast chapters and recordings, but changing the site's appearance currently requires editing Rust. A local stylesheet lets an owner give an episode site its own typography, colors, and spacing.

## What Changes

- Add optional local CSS to the SDK publishing options and `lsg publish --stylesheet PATH`.
- Copy the stylesheet into the generated site and load it after the built-in styles on every page.
- Validate the stylesheet before creating output and document how its relative resources resolve.
- Update publishing help to describe current alias, block-reference, and remote-media support.

## Capabilities

### New Capabilities

- `custom-site-stylesheet`: Local stylesheet validation, copying, and inclusion through the SDK and CLI.

### Modified Capabilities

None.

## Impact

The site publisher, CLI arguments and offline help, integration tests, and local-site documentation change. Existing SDK calls and default output remain compatible. No additional dependencies are required.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
