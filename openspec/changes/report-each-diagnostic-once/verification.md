# Verification: report-each-diagnostic-once

Each problem is now reported once, under the file that holds it, and no problem is lost. Checked on 2026-11-01 with `lsg` built from main (before) and from this change (after).

## Case graph

A copy of `docs` with one added page, `pages/lsg___Broken.md`, linked from `lsg.md`:

```
- # Broken
	- see [[lsg/Nowhere Page]] and [help](../../src/help.rs)
```

Published with the flags of `mise run docs:site` (`--include lsg --include Diataxis --homepage lsg --search --readable-urls`). Before:

```
pages/lsg.md: pages/lsg___Broken.md: Unresolved or excluded page reference: lsg/Nowhere Page
pages/lsg.md: pages/lsg___Broken.md: Unsupported local asset ../../src/help.rs: asset path escapes the source garden
pages/lsg___Broken.md: Unresolved or excluded page reference: lsg/Nowhere Page
pages/lsg___Broken.md: Unsupported local asset ../../src/help.rs: asset path escapes the source garden
.: Page dates omitted: the garden is not in a git repository
```

After:

```
pages/lsg___Broken.md: Unresolved or excluded page reference: lsg/Nowhere Page
pages/lsg___Broken.md: Unsupported local asset ../../src/help.rs: asset path escapes the source garden
.: Page dates omitted: the garden is not in a git repository
```

Without `--homepage` both builds report the same lines as with it: the repeats come from `lsg` listing the block in its Linked References, because the block links to `lsg/Nowhere Page`.

## Published gardens

| Garden | Diagnostics before | Diagnostics after | Files that differ |
| --- | --- | --- | --- |
| Case graph above, with and without `--homepage` | 5 | 3 | none |
| `docs`, `mise run docs:site` flags | 0 | 0 | none |
| Encode garden, `--journals --filename-format triple-lowbar` with one `--include` per top-level name: 8925 pages, 513 journals | 36359 | 12497 | none |

In the encode garden, 23877 of the 36359 diagnostics before had two file names in front. With the listing page's file removed, the before report has 8596 distinct lines, and the after report has the same 8596 distinct lines. Every diagnostic a page reported for its own text before is still reported, as many times as before.
