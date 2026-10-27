# Pagefind comparison

This experiment indexes an `lsg` parity build with [Pagefind](https://github.com/CloudCannon/pagefind) and measures its index size, the bytes each keystroke downloads and the keystroke latency, with the same browser profiles and queries as `mise run parity:perf --search-site`. It answers one question from the [`match-logseq-search` design](../../openspec/changes/match-logseq-search/design.md): is `lsg`'s own split search index within Pagefind's numbers? The results are in [Search performance](../../docs/parity/search-performance.md).

## Run it

1. Build the whole encode garden with search: `mise run parity:build`.
2. Index that build with Pagefind and measure it:

   ```sh
   uv run --script experiments/pagefind/measure.py
   ```

The script runs Pagefind 1.5.2 through `mise exec npm:pagefind@1.5.2`, so nothing is installed into the project. It writes the Pagefind files and `results.json` to `target/pagefind/` and leaves the parity build unchanged. Pass `--site <path>` to index another build, `--output <path>` to write elsewhere and `--profiles desktop` to skip the phone profile.

## What it measures

- **Index size:** the bytes of the Pagefind bundle by kind (index chunks, fragments, metadata, WebAssembly, scripts). Pagefind stores its files already gzip-compressed, so these are the bytes a browser downloads.
- **First search:** the time and bytes to load Pagefind and run one search, the closest match to `lsg`'s first open.
- **Each keystroke:** for each prefix of each query in [`scripts/parity/search-queries.txt`](../../scripts/parity/search-queries.txt), the time to search and load the data of the first ten results (the five pages and five blocks `lsg`'s dialog draws), and the bytes downloaded. A keystroke that downloads anything is cold; the rest are warm. Keystrokes run one after another, 300 ms apart, so a slow keystroke never overlaps the next.
- **Heap growth:** the JavaScript heap after all queries, after garbage collection, against the heap before loading Pagefind.

[`pagefind.yml`](pagefind.yml) indexes the page files (`p-*.html`) without the header and left sidebar each page repeats.

## Limits of the comparison

Pagefind returns pages ranked by its own relevance and excerpts, not Logseq's groups, aliases, breadcrumbs or block anchors, so its results are not a replacement for `lsg`'s dialog. The comparison covers cost only: download size and time.
