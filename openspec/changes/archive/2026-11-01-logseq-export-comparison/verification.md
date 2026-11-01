# Verification: logseq-export-comparison

`mise run parity:logseq-export` builds Logseq 0.10.15's export of the docs graph and an `lsg` site of it and compares all 88 pages. Before [PR #389](https://github.com/codekiln/logseq-gardener/pull/389) fixed [Issue #385](https://github.com/codekiln/logseq-gardener/issues/385), it reported the three `alias` rows that `lsg` left out and exited 1. With that fix, every page matches and the task exits 0.

Checked on 2026-11-01 on macOS with Logseq.app 0.10.15.

## The docs graph before the properties fix

This branch on top of `main` at `f08fbc2`:

```
- Garden: docs at f08fbc2
- Logseq: 0.10.15 (03bcefb), frontend from /Applications/Logseq.app/Contents/Resources/app
- lsg: 0.0.12 (f08fbc2), 0 diagnostics
- lsg options: --filename-format triple-lowbar --include lsg --include Diataxis --homepage lsg --search --readable-urls
- Pages compared: 88
- Pages that differ: 3
- Pages missing from Logseq's export: 0
- Minutes: 3.7
```

`Diataxis/Explanation`, `lsg` and `lsg/Ref/Site Manifest` differed only in their properties box: Logseq showed `alias: Diataxis/Concept`, `alias: Logseq Gardener` and `alias: lsg-site.json`, and `lsg` showed no `alias` row. The blocks of all 88 pages showed the same words in both builds.

## The docs graph with the properties fix

This branch rebased on `main` at `8393c0c`, which includes PR #389:

```
- Garden: docs at 4590ee4
- Logseq: 0.10.15 (03bcefb), frontend from /Applications/Logseq.app/Contents/Resources/app
- lsg: 0.0.12 (4590ee4), 0 diagnostics
- lsg options: --filename-format triple-lowbar --include lsg --include Diataxis --homepage lsg --search --readable-urls
- Pages compared: 88
- Pages that differ: 0
- Pages missing from Logseq's export: 0
- Minutes: 3.9
```

Both runs also reported:

- `contents`, the one Logseq page with blocks outside the `lsg` build; the docs options select `lsg` and `Diataxis`.
- `assets/[file].png`, `assets/image_1742292361076_0.png`, `assets/image_1742369972123_0.png` and `assets/x.mp4`, file names inside code examples that Logseq's export failed to copy. They do not affect the exit status.

`git status` reported the same files in `docs/` before and after the builds in both runs.

## The page reading on fixture pages

The task ran the browser tests in `scripts/parity/test_logseq_export.py` before it built: 30 tests passed. Removing a handling from the page reading made its browser test fail:

- without removing `.open-block-ref-link`, the badge test and the fixture comparison failed, because Logseq's text read `block3`;
- with the scrolling limited to one screen, the lazy-block test and the fixture comparison failed;
- with the prototype's text-node join in place of `innerText`, the split-span test and the fixture comparison failed, because the `lsg` text read `2 px`.

## The fallbacks

- With an empty cache and no local checkout, `logseq_source()` fetched the pinned paths from GitHub, and their digest matched `source.json`.
- With an empty cache and no Logseq.app, `frontend_folder()` downloaded `Logseq-linux-x64-0.10.15.zip`, matched its SHA-256, unpacked 129 MB of frontend files and matched the frontend digest; a second call used the cache. Logseq's export of the docs graph with that frontend wrote a `static/` folder identical to the one written with Logseq.app's frontend, and the same 112 pages in its graph data.

## Another garden

On a three-page garden, `mise run parity:logseq-export --garden <garden>` without options stopped and asked for the `lsg publish` options. With `-- --include Notes --filename-format triple-lowbar` it compared `Notes` and `Notes/Child`, and listed `Other` as a Logseq page outside the `lsg` build.
