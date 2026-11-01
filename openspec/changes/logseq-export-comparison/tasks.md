## 1. Pinned inputs

- [x] 1.1 Add `scripts/parity/logseq-export/source.json` with the Logseq tag, commit, paths and digest, and the frontend's version, digest and release archive
- [x] 1.2 Add `scripts/parity/logseq-export/package.json` and `package-lock.json` for `@logseq/nbb-logseq` 1.2.173, `mldoc` 1.5.7 and `fs-extra` 9.1.0

## 2. Builds

- [x] 2.1 Fetch, verify and cache the Logseq source in `scripts/parity/logseq_export.py`
- [x] 2.2 Find and verify the frontend: `--static-dir`, Logseq.app, the cache, then the release archive
- [x] 2.3 Run Logseq's export into the run directory, keep its log, and read the files it failed to copy
- [x] 2.4 Build `lsg` and publish the garden with the docs options or the options passed to the task
- [x] 2.5 Check that the garden's `git status` is the same before and after the builds

## 3. Comparison

- [x] 3.1 Pair `lsg-site.json` pages with Logseq's pages from the export's inline graph data
- [x] 3.2 Read each page's block text and property rows in headless Chromium from one local server
- [x] 3.3 Compare words and property rows, write `logseq-export.json` and `logseq-export.md`, and exit 1 when a page differs
- [x] 3.4 Add the `parity:logseq-export` mise task

## 4. Tests

- [x] 4.1 Add unit tests in `scripts/parity/test_logseq_export.py` for words, property rows, the comparison, the export log, the page list, the report and the scrolling loop
- [x] 4.2 Add fixture pages under `scripts/parity/testdata/logseq-export/` and a browser test class that the task runs before it builds

## 5. Docs

- [x] 5.1 Describe the task on `lsg/Ref/Parity Tasks` and `lsg/How To/Compare a Build with Logseq`
- [x] 5.2 Update the sentences on `lsg/Ref/Logseq Compatibility/Comparison Harness` that say the harness never runs Logseq's export

## 6. Verification

- [x] 6.1 Run the task on the docs graph and record the summary in `verification.md`
- [x] 6.2 Run the release archive fallback end to end and record the result in `verification.md`
- [x] 6.3 Run `mise run ci`
