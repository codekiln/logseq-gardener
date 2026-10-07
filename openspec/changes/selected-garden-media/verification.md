# Selected podcast media

The November GitP episode displays its hosted artwork and plays its hosted recording through native browser controls. The CLI fixture also verifies local MP3, WAV, and OGG controls, download links, source preservation, and excluded/private media across every generated file.

## Real garden

The source is the existing public `codekiln/logseq-encode-garden` checkout. The [November session](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/GitP___A___Session___24___11___19-Tue.md) supplies direct HTTPS media. Gitpa's [publication preparation](https://github.com/codekiln/gitpa/blob/codex/13-render-garden-proxies/scripts/prepare_site.py) demonstrates the same listener presentation from proxied session bodies; the Gitpa main Ceremony pages additionally exercise local recording links.

```sh
mise exec -- cargo run --bin lsg -- publish \
  --graph "$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)" \
  --output /private/tmp/gitp-media-demo-20261007 \
  --filename-format triple-lowbar \
  --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu \
  --format json
```

Generated twenty pages with thirteen diagnostics for references outside the selected namespaces. The previous publisher produced twelve additional remote-media diagnostics on the same selection. The new site contains no September 24 page or media URL. All ordinary source files under pages, journals, and assets matched their pre-generation SHA-256 hashes; the source snapshot covered 7,263 files. Source gardens were preserved.

Served the output on a loopback HTTP server and followed the index link to the November episode in the Codex in-app browser. Its artwork loaded at its intrinsic 320 by 570 dimensions. Clicking the audio play button produced readyState 4, no media error, and a finite duration of about seven minutes; the player switched to a pause button and was paused after inspection. The recording link and preset links remained visible. This inspection verified playback; it did not verify a hosted download. The browser screenshot is saved locally at `/private/tmp/gitp-media-demo-20261007.png`.

## Checks

`mise run ci` passed, including formatting, Clippy, workspace tests, documentation, and the repository's existing validation tasks. The renderer and CLI fixture also verify that local recording links request downloads and hosted recording links say “Open recording” without a download attribute. The new CLI fixture checks every generated file for excluded/private text, remote URLs, and local media sentinels. Renderer tests cover URL and label escaping, query/fragment suffixes, malformed authorities, controls, unsafe schemes, ordinary links, and unsupported media fallbacks. Existing nested-label coverage ensures labels cannot request hidden assets.

## Limits

Browser codec support depends on the recording. Hosted media requires network access when viewed, and generation does not fetch remote media. Finer private-outline selection, homepage query evaluation, and broader reference/alias resolution remain separate work. This implementation depends on publishing integration reaching main; its OpenSpec cleanup also follows the local-static-site specification sync.
