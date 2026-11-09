## Why

A page published with `lsg publish --readable-urls --redirect-old-urls` is reachable at its folder, through its old `p-<digest>.html` file and through Logseq's `#/page/<name>` link. Search engines and feed readers need one address per page, and the page itself names none. Gitpa wrote `<link rel="canonical">` into its episode pages after `lsg` ran, and the work for [codekiln/gitpa#159](https://github.com/codekiln/gitpa/issues/159) removes that code with its episode player. [Issue #589 Write rel=canonical on pages published with --readable-urls](https://github.com/codekiln/logseq-gardener/issues/589) asks `lsg` to write the link for every page, since it already knows each page's route.

## What Changes

- With `--readable-urls`, every published page, journal, generated namespace page and reference-only page names its own route in a `<link rel="canonical">` in its head. The `--homepage` page names the site root, in both of its copies.
- The plugin host of a readable build replaces the canonical link when it moves to another page in the same document.
- The digest file redirects of `--redirect-old-urls` already name the page's route as canonical; they stay as they are.
- Without `--readable-urls`, the output is unchanged. Logseq's own published site writes no canonical links.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readable-page-urls`: pages in a readable build name their route as canonical, and the plugin host keeps that link current across navigation.

## Impact

- SDK: `site/readable.rs` writes the link beside `<base>` and patches the readable plugin host; `site/mod.rs` passes each page's route.
- CLI: one clause in the `--readable-urls` help text.
- Docs: the Readable URLs section of `lsg/Ref/Publish Options`, the help review record in `lsg/How To/Review Help Examples`, and the readable URL row of `docs/parity/gitp-a-shared-rendering.md`.

## Citations

- [My/Principle/Simplify/Minimize Surface Area](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Minimize%20Surface%20Area.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don't%20Repeat%20Yourself%20DRY.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
