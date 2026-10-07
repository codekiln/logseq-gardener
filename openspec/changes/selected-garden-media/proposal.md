## Why

Listeners need to play recordings and see artwork on a published episode page. The [November session](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/GitP___A___Session___24___11___19-Tue.md) already contains HTTPS GIF and MP3 links, but the Rust publisher renders these as plain labels; the [Gitpa presentation plan](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/GitP___A___Log___26___10___05%20Mon%20-%20simplify%20gitpa%20garden%20proxies___Plan.md) calls for an audio player and artwork.

## What Changes

- Render local and HTTPS MP3, WAV, and OGG image syntax as audio controls with a descriptive name and download link.
- Render HTTPS raster artwork as images, using the same plain escaped labels as local artwork.
- Keep media within selected, visible pages and preserve the local asset resolver's protections.
- Document a real session publishing command and validate excluded media against the generated site.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-static-publishing`: Extend supported rendering to audio controls and HTTPS artwork. Synchronization follows the local-static-site archival so its base requirement exists.

## Impact

The SDK HTML renderer and CLI publishing fixtures change. Generation remains local and performs no remote media requests; the browser loads HTTPS media when viewing the site. This work depends on [PR #33 — Publishing integration](https://github.com/codekiln/logseq-gardener/pull/33) reaching main.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md): a visitor can listen on the episode page.
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md): reuse the existing renderer and asset resolver.
