## 1. #325 Turn docs/ into a Logseq graph and build it with lsg

- [x] 1.1 Add `docs/logseq/config.edn` with the settings in the design and one `:hidden` line for each Markdown file under `docs/` that the page map lists as a source
- [x] 1.2 Commit an empty `docs/logseq/custom.css` and `docs/pages/contents.md`, and add `docs/logseq/bak/` and `docs/logseq/.recycle/` to `.gitignore`
- [x] 1.3 Write the `lsg` homepage, the `Diataxis` kind index pages and `lsg/How To/Edit the Documentation`
- [x] 1.4 Create a stub for every other page in the page map, holding its `tags::` and a one-line summary block
- [x] 1.5 Add `mise run docs:site`, which builds `lsg` from the checkout, publishes the graph with the design's options into a fresh directory, and fails on any diagnostic
- [x] 1.6 Make `mise run docs:check` run `docs:site`, and extend `scripts/docs.py` to check relative links in the remaining old files, missing `:hidden` entries and kind tags that do not match titles
- [x] 1.7 Check out full history in the `ci` job of `.github/workflows/ci.yml`
- [x] 1.8 Add a short section to `CONTRIBUTING.md` on opening `docs/` in Logseq and building the site
- [ ] 1.9 Open `docs/` in Logseq 0.10 desktop, re-index, and record in the pull request that the home page is `lsg`, every page loads and `git status` stays clean, or name who will make that check

## 2. #326 Deploy the docs site to GitHub Pages

- [ ] 2.1 Pass the readable page URL option from `readable-page-urls` in `docs:site`
- [ ] 2.2 Add `.github/workflows/docs-site.yml` with the build and deploy jobs the design describes
- [ ] 2.3 Enable GitHub Pages with the GitHub Actions source
- [ ] 2.4 Write `lsg/How To/Publish a Garden to GitHub Pages`
- [ ] 2.5 After merge, confirm the deployment succeeded and the homepage loads at https://codekiln.github.io/logseq-gardener/, and comment on #326 with the run link

## 3. #327 Move install, CLI and tutorial docs into the docs graph

- [ ] 3.1 Write `lsg/How To/Install with mise`, including what mise checks against each release's `.sha256` files, as read in mise's source, and embed its block of install commands on the `lsg` homepage
- [ ] 3.2 Convert `docs/installation.md` into `lsg/How To/Install a Release Archive`, `lsg/How To/Build from Source` and the dependency step of `lsg/How To/Use the Rust SDK`, with directions for the published releases
- [ ] 3.3 Convert `docs/cli.md` into `lsg/Ref/Command Line` and `lsg/How To/Look Up a Page File`
- [ ] 3.4 Convert `docs/weekend-trial.md` into `lsg/Tutorial/Publish Part of a Public Garden`, using a released `lsg` installed with mise
- [ ] 3.5 Convert `docs/page-titles.md` and `docs/garden-loading.md` into `lsg/Ref/Page Titles`, `lsg/Ref/Garden Loading` and the remaining steps of `lsg/How To/Use the Rust SDK`, describing the current loader
- [ ] 3.6 Delete the five documents, remove their `:hidden` lines and repoint every link to them

## 4. #328 Move publishing guides into the docs graph

- [ ] 4.1 Convert `docs/publishing.md` into `lsg/Static Publishing`, `lsg/Ref/Site Manifest`, `lsg/Ref/Plugin Host API`, `lsg/Ref/Build Plugin Protocol`, `lsg/How To/Add a Site Plugin` and its parts of the other pages the page map names
- [ ] 4.2 Convert `docs/local-site.md` into `lsg/How To/Publish Selected Namespaces`, `lsg/How To/Customize a Published Site`, `lsg/Ref/Publish Options`, `lsg/Ref/Selection and Visibility`, `lsg/Ref/Rendering and Assets`, `lsg/Ref/Published Pages`, `lsg/Ref/Site Navigation` and `lsg/Ref/Host Integration API`
- [ ] 4.3 Convert `docs/static-search.md` into `lsg/Ref/Static Search Data`, `lsg/Ref/Search Dialog` and its parts of the other pages the page map names
- [ ] 4.4 Point the comments in `crates/logseq-gardener-sdk/examples/publish_site.rs` and `crates/logseq-gardener-sdk/src/site/host.mjs` at the new pages
- [ ] 4.5 Delete the three documents, remove their `:hidden` lines and repoint every link to them, leaving `src/help.rs` and `tests/publish.rs` to #334

## 5. #329 Move contributor and release docs into the docs graph

- [x] 5.1 Convert `docs/architecture.md` and `docs/parser-adoption.md` into `lsg/Architecture`, `lsg/Parser Adoption` and `lsg/How To/Change the Parser Revision`
- [x] 5.2 Convert `docs/releases.md` into the release pages the page map names, describing the automated releases as they run now, and point `fnox.toml` and `.mise/tasks/release/store-app-key` at `lsg/How To/Rotate the Release App Key`
- [x] 5.3 Convert `docs/licensing.md` and `docs/dependencies.md` into `lsg/Licensing`, `lsg/How To/Update Dependencies`, `lsg/Ref/Dependencies` and their part of `lsg/Ref/Release Archives`, and keep `mise run notices:check` passing
- [x] 5.4 Convert `docs/openspec-cleanup.md` into `lsg/How To/Finish an OpenSpec Change After Merge`, point `.rulesync/rules/overview.md` at it, and run `mise exec -- rulesync generate` and `mise run rulesync:check`
- [x] 5.5 Convert `docs/publishing-cleanup-verification.md` into `lsg/Ref/Publishing Cleanup Verification`
- [x] 5.6 Convert `help-assessment.md` into `lsg/How To/Review Help Examples`, update every reference to it, and remove it from `include` in `dist-workspace.toml`
- [x] 5.7 Write `lsg/How To/Contribute a Change` and `lsg/Ref/Project Checks`, and shorten `CONTRIBUTING.md` to an entry point that keeps the section from #325
- [x] 5.8 Delete the converted documents, remove their `:hidden` lines and repoint every link to them

## 6. #330 Move Logseq parity reports into the docs graph

- [ ] 6.1 Convert each feature report, `comparison-harness.md` and `search-performance.md` into its `lsg/Ref/Logseq Compatibility/` page, keeping every measurement and its date
- [ ] 6.2 Convert `docs/parity/README.md` and `docs/parity/publication-speed.md` into `lsg/How To/Compare a Build with Logseq`, `lsg/Ref/Parity Tasks`, `lsg/How To/Measure Performance` and `lsg/Ref/Publication Speed`
- [ ] 6.3 Make `scripts/parity/perf.py --record` write `lsg/Ref/Logseq Compatibility/Performance` as Logseq outline Markdown, convert the current results, and keep its tests passing
- [ ] 6.4 Update the paths in `scripts/parity/`, `.mise/tasks/parity/` and the comments in `.github/workflows/ci.yml`
- [ ] 6.5 Keep the `parity-comparison-harness`, `published-graph-view` and `published-page-outline` deltas in this change matching the page file `perf.py` writes
- [ ] 6.6 Write the overview `lsg/Ref/Logseq Compatibility` from the reports
- [ ] 6.7 Delete the reports, remove their `:hidden` lines and repoint every link to them

## 7. #331 Move the Logseq reference boards into the docs graph

- [ ] 7.1 Convert `docs/logseq-reference/README.md` into `lsg/Ref/Logseq Published Site`, with the table that pairs each page with its HTML board
- [ ] 7.2 Convert each board into its `lsg/Ref/Logseq Published Site/` page, keeping every measurement, class name and string, and every table inside a block
- [ ] 7.3 Keep each page source on `lsg/Ref/Logseq Published Site/Clickable Prototype` in a code block
- [ ] 7.4 Delete the board documents, remove their `:hidden` lines and repoint every link to them

## 8. #332 Check the docs graph with Logseq's own parser

- [x] 8.1 Add a check to `mise run docs:check` that loads the docs graph with Logseq's file parser and the SDK and reports each difference in pages, titles, aliases, page references, block references or properties with its file and item
- [x] 8.2 Add a fixture with a deliberate mismatch and a test that the check fails on it
- [x] 8.3 Make the check obtain the pinned Logseq source when no local checkout has it, and confirm it runs in CI
- [x] 8.4 Describe the check on `lsg/How To/Edit the Documentation`

## 9. #333 Rewrite the README for released binaries and mise installs

- [ ] 9.1 Rewrite `README.md`: problem and audience first, then the global and project mise installs, first use, and links to site pages for everything else
- [ ] 9.2 Keep commands that change the machine out of executed `sh` blocks, and keep the executed first-use commands passing
- [ ] 9.3 Make `mise run docs:links` check the README's links to the site
- [ ] 9.4 Point the README assessment skills and the example link in `.rulesync/rules/communication-style.md` at the install page, and regenerate the RuleSync output
- [ ] 9.5 Run `assess-readme-style` and `assess-readme-purpose`, and resolve every finding or record the accepted exception in the pull request

## 10. #334 Link CLI help to pages on the docs site

- [ ] 10.1 Add the documentation base URL and a function that turns a page title into its URL with the SDK's readable route function
- [ ] 10.2 Replace every `docs/` path in `src/help.rs` with a page title and URL, and update `tests/publish.rs`
- [ ] 10.3 Add the ignored test `tests/docs_links.rs` and run it from `mise run docs:check` against the site it builds
- [ ] 10.4 Keep the `cli-static-publishing` delta in this change matching the help text
- [ ] 10.5 Refresh the help assessment and keep `mise run clilint:check` passing
- [ ] 10.6 Write `lsg/How To/Link CLI Help to a Docs Page`
