# Publish a selected garden through lsg

The new `lsg publish` command generated a local HTML site from the public encode garden. The source checkout was based on [e12f956 — Refresh LangSmith CLI install and auth details](https://github.com/codekiln/logseq-encode-garden/commit/e12f9568693662aee21c0c61b9787a767a56d4e9), with existing user changes. These results describe that working snapshot.

## Real-garden command

After building `lsg` in the publishing CLI worktree, run:

```sh
./target/debug/lsg publish \
  --graph /path/to/logseq-encode-garden \
  --output /private/tmp/logseq-gardener-cli-demo-20261007 \
  --filename-format triple-lowbar \
  --include My/AI --exclude My/AI/Agent \
  --include 'AI/ES/25/ws/1/Building Agents with Model Context Protocol' \
  --format json
```

The command exited successfully with one JSON document on stdout. It reported 33 pages, ten assets, no withheld pages, 488 skipped journals, and 567 publication diagnostics. Diagnostics remained on stderr. The generated index contained only titles beneath the included roots and none beneath `My/AI/Agent`.

## Browser inspection

The generated index opened through a loopback HTTP preview. The Words and Phrases to Avoid page displayed headings and nested outlines; its link to Be like the holograms Data from ST:TNG conferred with opened the generated destination. Garden index navigation returned to the index.

The selected Model Context Protocol workshop page displayed its copied images. All ten image elements completed loading with nonzero natural widths, and a screenshot showed a copied image within the rendered outline. Browser inspection used HTTP; direct file opening remains a user action outside the automation browser's supported protocols.

## Source preservation and remaining scope

Hashes of all visible ordinary files under pages, journals, and assets matched before and after generation. Existing user changes were preserved.

The command delegates to the same SDK publisher tested in [PR #19 — Publish selected Logseq namespaces as local HTML](https://github.com/codekiln/logseq-gardener/pull/19). Parsed `public:: false` withholds a whole page; journals are skipped, and aliases, block references, embeds, and queries retain the SDK's current fallbacks. The successful trial demonstrates selected-page HTML generation through the normal CLI. More complete page lookup remains planned in [Issue #4 — Add read-only page lookup through the SDK](https://github.com/codekiln/logseq-gardener/issues/4).

## CLI verification

The actual CLI pipeline tests cover selected links and assets, excluded/private sentinels across generated files, unchanged sources, invalid selectors and arguments, fatal SDK errors, empty selection, native paths, and separate stdout/stderr reports. Recursive help tests discover the publishing command and exercise its equivalent help aliases and navigable sections with closed input.

The Unicode-control path regression checks parseable JSON, preservation of original path values after JSON decoding, successful publication at the native path, and escaped human output. The regression reproduced invalid JSON before the fix and passed afterward. Linux also exercises filenames containing invalid UTF-8; macOS rejects creating those filesystem names, so its native-path trial uses valid Unicode and control characters.

The complete `mise run ci` workflow passed after the JSON correction, including CLI and SDK tests, formatting, linting, documentation checks, the pinned hierarchical-help assessment, and existing release/distribution consistency checks. The `cli-static-publishing` OpenSpec change also passed strict validation.
