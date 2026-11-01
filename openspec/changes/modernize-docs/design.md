## Context

The documentation lives in Markdown files under `docs/`, plus `CONTRIBUTING.md` and `help-assessment.md` at the repository root. Readers find these files on GitHub, and agents read them in the checkout. The README and `docs/installation.md` still say no binary release exists, although releases v0.0.1 through v0.0.6 are published. `lsg help` names three of these files by path, and a test asserts one of them.

`lsg` already publishes real gardens: Gitpa builds its podcast site with a released `lsg` ([Publish workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml), [publishing notes](https://github.com/codekiln/gitpa/blob/main/docs/gardener-publishing.md)). Several facts about `lsg` and Logseq shape this design:

- `lsg` and Logseq load every `.md` file beneath every visible folder of the graph root, apart from paths that `:hidden` in `logseq/config.edn` names ([markdown-garden-loading](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/markdown-garden-loading/spec.md)). `lsg` needs a `pages/` or `journals/` folder in the root.
- `lsg publish` selects pages only through `--include`, and reports a diagnostic for a page reference or tag with no selected page, for a relative link that does not name a supported file under `assets/`, for syntax it does not render, and for page dates read from a shallow git clone.
- Logseq 0.10 desktop reads `.md`, `.json`, `.edn`, `.css` and a few other formats. It makes pages only from Markdown and Org files, and it skips `.html`. When it opens a graph, it creates `logseq/config.edn`, `logseq/custom.css` and `pages/contents.md` if they are missing, and a journal page for today when journals are on.
- Readable page URLs come from the separate change `readable-page-urls` ([Issue #324 Publish pages at readable namespace URLs](https://github.com/codekiln/logseq-gardener/issues/324)), which adds an option to `lsg publish` and a route rule to the SDK.
- Child issues [#325](https://github.com/codekiln/logseq-gardener/issues/325) through [#334](https://github.com/codekiln/logseq-gardener/issues/334) of [Issue #322 Publish the documentation as a Logseq graph on GitHub Pages](https://github.com/codekiln/logseq-gardener/issues/322) implement this change, several of them in parallel pull requests.

## Goals / Non-Goals

**Goals:**

- A reader who opens `docs/` in Logseq 0.10 desktop reads the same pages that the site shows.
- Each page serves one reader need, carries a title that says which, and links the pages that serve the reader's other needs.
- Each merge to `main` deploys the site built by the `lsg` of that commit, and each pull request uploads a preview.
- CLI help links pages on the site, and CI fails when a linked page is missing.
- Parallel pull requests change separate files and separate lines of shared files.

**Non-Goals:**

- A site for each release, so that help links match the installed version's documentation. The site follows `main`; each release archive carries the docs graph of its own commit.

## Decisions

### Graph layout

`docs/` is the graph root. Release archives already include `docs` (`dist-workspace.toml`), so every release carries the graph.

```text
docs/
  logseq/config.edn        graph settings
  logseq/custom.css        empty; committed so Logseq leaves it alone
  pages/                   every page, named by title in the triple-lowbar format
  pages/contents.md        Logseq's Contents sidebar page; not published
  assets/                  images that pages show, when a page needs one
  parity/*.json            data written by the parity scripts
  dependency-sources.json  inventory written by scripts/notices.py
  logseq-reference/html/   the HTML boards behind the Logseq reference pages
  <unconverted>.md         old documents, hidden until converted
```

The non-Markdown files stay at their current paths. `lsg` reads only `.md`; Logseq keeps the `.json` files as file records without pages and skips the `.html` boards. Their scripts, the CI comments, `THIRD_PARTY_LICENSES.txt` and the release archives keep working without path changes.

### Configuration

```clojure
{:file/name-format :triple-lowbar
 :publishing/all-pages-public? true
 :publishing/page-properties? true
 :publishing/page-properties-collapsed? false
 :feature/enable-journals? false
 :default-home {:page "lsg"}
 :favorites ["lsg"
             "lsg/How To/Install with mise"
             "lsg/Tutorial/Publish Part of a Public Garden"
             "lsg/How To/Publish a Garden to GitHub Pages"
             "lsg/Ref/Command Line"
             "lsg/Ref/Logseq Compatibility"]
 :hidden ["/architecture.md"
          "/cli.md"
          "/parity/README.md"
          "/logseq-reference/Main.md"]}
```

The `:hidden` vector above is shortened; it holds one line for each Markdown file under `docs/` that the page map lists as a source. Each entry is the file's path from `docs/` with a leading `/`, and each sits on its own line, so a conversion removes its own lines and parallel pull requests merge without conflicts. Every entry includes `.md`, so no entry is a prefix of another file's path.

`:publishing/page-properties?` and its collapse setting make the site show page properties such as `tags::` and `see-also::`, as Logseq does. Journals are off, so Logseq creates no journal file and opens the `lsg` page. The favorites fill the left sidebar in Logseq and on the site.

### Files Logseq creates

The repository commits `docs/pages/contents.md` and an empty `docs/logseq/custom.css`, and `.gitignore` lists `docs/logseq/bak/` and `docs/logseq/.recycle/`. With journals off, opening the graph in Logseq leaves `git status` clean. `contents.md` links the `lsg` page and the kind index pages, so Logseq's Contents sidebar offers the same starting points as the site.

### Page titles, kinds and tags

Titles follow codekiln's Diataxis convention ([Logseq/Entity/Diataxis](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis.md)) under the topic root `lsg`:

| Kind | Title | Tag |
| --- | --- | --- |
| How To | `lsg/How To/<task>` | `[[Diataxis/How To]]` |
| Tutorial | `lsg/Tutorial/<goal>` | `[[Diataxis/Tutorial]]` |
| Reference | `lsg/Ref/<subject>` | `[[Diataxis/Reference]]` |
| Explanation | `lsg/<concept>`, or `lsg` for the topic itself | `[[Diataxis/Explanation]]` |

Explanation pages take the concept's name inside the `lsg` namespace, such as `lsg/Architecture`, so `--include lsg` selects every documentation page. They carry `[[Diataxis/Explanation]]`, which the convention allows beside `[[Diataxis/Concept]]`.

The graph holds the kind index pages `Diataxis`, `Diataxis/How To`, `Diataxis/Tutorial`, `Diataxis/Reference` and `Diataxis/Explanation`, which carries `alias:: [[Diataxis/Concept]]`. Each says what its kind is for and links [Diátaxis](https://diataxis.fr/) and codekiln's convention; its Pages tagged with section, in Logseq and on the site, lists the pages of its kind. `Diataxis` holds the table above with links to the four kind pages. Without these pages every tag would be an unresolved reference, which `lsg` reports.

Titles avoid `?`, `:`, `#`, `%`, `"` and `'`, so file names, artifact paths and readable URLs stay plain. Titles differ by more than letter case.

### Page format

Pages are Logseq outline Markdown. Page properties sit at the top of the file as `key:: value` lines, followed by a blank line. Every block starts with `- `, and children are indented with one tab per level. A paragraph becomes a block; a list becomes child blocks; a heading becomes a heading block with its content as children; a table or a fenced code block stays inside one block.

Each page follows the page shape that codekiln's entity page for its kind describes ([How To](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis___How%20To.md), [Tutorial](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis___Tutorial.md), [Reference](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis___Reference.md), [Explanation](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis___Explanation.md)) and leaves out empty sections. Converted parity reports and Logseq reference boards keep their own sections, which already describe one subject systematically.

A document that mixes steps with reference tables becomes a how-to page and a reference page that name each other in `see-also::`. A short document stays one page; the page map keeps each page to more than a couple of blocks.

Converted pages keep every fact their source states, unless the fact describes finished work, such as the "Next publishing steps", "Publishing follow-up", "Current work" and "Evidence and next work" sections. Those sections are replaced with current facts or dropped. Dated measurements keep their dates.

### Links

- A link to another page is a page reference, `[[lsg/Ref/Command Line]]`, to a page in the graph.
- A link to a repository file is a GitHub URL on `main`, such as `https://github.com/codekiln/logseq-gardener/blob/main/src/help.rs`, or `tree/main/` for a folder. `lsg` reads a relative link as an asset and reports any other target.
- An image that a page shows goes in `docs/assets/`, as in any Logseq graph.
- A repository file outside the graph that points at a page, such as a code comment, `fnox.toml`, a task message or a RuleSync rule, names the page title and its file, for example `lsg/How To/Rotate the Release App Key` in `docs/pages/lsg___How To___Rotate the Release App Key.md`. CLI help and the README link the site once #334 and #333 land; until then the README links page files.

### Logseq features in the docs

The pages use the Logseq features `lsg` publishes where they help a reader:

- Namespaces and the Hierarchy section: every documentation page sits under `lsg`, and the `lsg` page's Hierarchy section lists them all. `lsg/Ref/Logseq Compatibility` and `lsg/Ref/Logseq Published Site` are overview pages whose Hierarchy sections list their detailed pages.
- Tags: the kind tags, with the kind index pages as their lists.
- Aliases: `lsg` has `Logseq Gardener`, `Diataxis/Explanation` has `Diataxis/Concept`, and `lsg/Ref/Site Manifest` has `lsg-site.json`. Writers add an alias where readers search for another name.
- Page properties: `tags::`, `alias::` and `see-also::`.
- Block references and embeds: the homepage embeds the block of install commands from `lsg/How To/Install with mise`, which #327 adds when it writes that block, and the compatibility overview references each report's summary block. A referenced block carries an `id::` property.
- Favorites in the left sidebar.

Pages use no query, raw HTML, drawer, footnote or macro that `lsg` reports, since any diagnostic fails the build.

### Homepage

The homepage is the page `lsg`, published at `index.html` with `--homepage lsg` and opened by Logseq through `:default-home`. It carries `alias:: [[Logseq Gardener]]` and `tags:: [[Diataxis/Explanation]]`; in codekiln's convention, the explanation of a topic takes the topic's own name. It says what `lsg` is for and who uses it, embeds the install commands, and links the tutorial, the main how-to pages, the main reference pages, the compatibility overview and the contributor pages. Its Hierarchy section lists every page.

### Page map

Each group heading names the issue that writes the pages in its table. Each row names a source document, the page it becomes and the page's kind. A source document that yields several pages appears once per page, with the sections that page takes. "New" marks a page with no source document. #325 creates a stub of every page in this map, holding its `tags::` and a one-line summary block, so pages can link each other whichever issue merges first.

Each page draws only on documents of its own issue. Where two issues' documents state the same fact, the page that owns the subject keeps it and the other issue drops it. The Publishing reports section of `docs/cli.md` is the one case: its publish-specific facts also appear in `docs/local-site.md` and go to `lsg/Ref/Publish Options`, and its facts that hold for every command go to `lsg/Ref/Command Line`.

#### #325 Turn docs/ into a Logseq graph and build it with lsg

| Source | Page | Kind |
| --- | --- | --- |
| New | `lsg` (alias `Logseq Gardener`; the homepage) | Explanation |
| New | `Diataxis` | Index |
| New | `Diataxis/How To` | Index |
| New | `Diataxis/Tutorial` | Index |
| New | `Diataxis/Reference` | Index |
| New | `Diataxis/Explanation` (alias `Diataxis/Concept`) | Index |
| New | `lsg/How To/Edit the Documentation` | How To |
| New | `Contents` (`pages/contents.md`; Logseq only) | None |

`lsg/How To/Edit the Documentation` shows how to open `docs/` in Logseq 0.10 desktop, name and tag a page, convert a document, and build and check the site.

#### #326 Deploy the docs site to GitHub Pages

| Source | Page | Kind |
| --- | --- | --- |
| New | `lsg/How To/Publish a Garden to GitHub Pages` | How To |

The page walks a reader through publishing their own garden with a released `lsg` installed by mise, using this repository's workflow and Gitpa's as worked examples.

#### #327 Move install, CLI and tutorial docs into the docs graph

| Source | Page | Kind |
| --- | --- | --- |
| New, drawing on `docs/installation.md` | `lsg/How To/Install with mise` | How To |
| `docs/installation.md` (Binary releases) | `lsg/How To/Install a Release Archive` | How To |
| `docs/installation.md` (build, PATH and linker paragraphs) | `lsg/How To/Build from Source` | How To |
| `docs/installation.md` (Rust SDK), `docs/page-titles.md` (Try the SDK), `docs/garden-loading.md` (example and module names) | `lsg/How To/Use the Rust SDK` | How To |
| `docs/cli.md` (commands, Navigate documentation, JSON format 1, Streams and exit codes, the facts of Publishing reports that hold for every command, lookup matching and JSON) | `lsg/Ref/Command Line` | Reference |
| `docs/cli.md` (Find a page's source file) | `lsg/How To/Look Up a Page File` | How To |
| `docs/weekend-trial.md` | `lsg/Tutorial/Publish Part of a Public Garden` | Tutorial |
| `docs/page-titles.md` (Supported title rules) | `lsg/Ref/Page Titles` | Reference |
| `docs/garden-loading.md` (Supported input) | `lsg/Ref/Garden Loading` | Reference |

`lsg/How To/Install with mise` covers a global install with `mise use -g github:codekiln/logseq-gardener`, a project install with `mise use github:codekiln/logseq-gardener` that pins `lsg` in the project's `mise.toml`, pinning a version, running one version with `mise x`, and what mise checks against each release's `.sha256` files. The tutorial installs a released `lsg` with mise. `lsg/Ref/Garden Loading` describes the current loader: every visible folder, `:hidden`, and git-ignored files.

#### #328 Move publishing guides into the docs graph

| Source | Page | Kind |
| --- | --- | --- |
| `docs/publishing.md` (introduction, Current work, Sources, Trust) | `lsg/Static Publishing` | Explanation |
| `docs/publishing.md` (Try a garden), `docs/local-site.md` (introduction, Try the public garden, Try a podcast selection) | `lsg/How To/Publish Selected Namespaces` | How To |
| `docs/local-site.md` (Customize appearance, Publish journals, Choose a homepage, and the settings in Page properties, Reference sections, Header and left sidebar and Appearance), `docs/static-search.md` (turning search on) | `lsg/How To/Customize a Published Site` | How To |
| `docs/publishing.md` (Plugins and Build plugins: steps and example manifests) | `lsg/How To/Add a Site Plugin` | How To |
| `docs/local-site.md` (option paragraphs, the JSON report and exit codes) | `lsg/Ref/Publish Options` | Reference |
| `docs/publishing.md` (Namespace selection, Publication boundaries), `docs/local-site.md` (Visibility and references) | `lsg/Ref/Selection and Visibility` | Reference |
| `docs/local-site.md` (Rendering and assets) | `lsg/Ref/Rendering and Assets` | Reference |
| `docs/local-site.md` (Hierarchy, Collapsed outlines, Outlines collapse and block zoom, Page properties, Reference sections, Namespace destinations and page sequences, Dialogue timestamps) | `lsg/Ref/Published Pages` | Reference |
| `docs/local-site.md` (Header and left sidebar, Right sidebar, All pages, the journals feed, View as slides, Graph view, Appearance) | `lsg/Ref/Site Navigation` | Reference |
| `docs/local-site.md`, `docs/static-search.md` (browser functions, events and data attributes for hosts) | `lsg/Ref/Host Integration API` | Reference |
| `docs/publishing.md` (Site manifest) | `lsg/Ref/Site Manifest` (alias `lsg-site.json`) | Reference |
| `docs/publishing.md` (manifest fields, What a page carries, Host API, Navigation) | `lsg/Ref/Plugin Host API` | Reference |
| `docs/publishing.md` (Build plugins: fields, The view, Contributions, failures, in-process plugins) | `lsg/Ref/Build Plugin Protocol` | Reference |
| `docs/static-search.md` (introduction, Publication boundary, Artifacts, Matching and ranking, Pure consumer API, Caching and host routes, Checks) | `lsg/Ref/Static Search Data` | Reference |
| `docs/static-search.md` (Shared dialog and persistent hosts, state attributes) | `lsg/Ref/Search Dialog` | Reference |

Each browser function a host calls, such as `window.logseqGardenerInitOutline`, is described once, on `lsg/Ref/Host Integration API`; the feature pages link it.

#### #329 Move contributor and release docs into the docs graph

| Source | Page | Kind |
| --- | --- | --- |
| `docs/architecture.md` | `lsg/Architecture` | Explanation |
| `docs/parser-adoption.md` (all but Change the parser revision) | `lsg/Parser Adoption` | Explanation |
| `docs/parser-adoption.md` (Change the parser revision) | `lsg/How To/Change the Parser Revision` | How To |
| `docs/releases.md` (introduction, Release flow, Version policy) | `lsg/Releases` | Explanation |
| `docs/releases.md` (Release flow steps, Recovery) | `lsg/How To/Run a Release` | How To |
| `docs/releases.md` (One-time GitHub setup) | `lsg/How To/Set Up Release Automation` | How To |
| `docs/releases.md` (Rotate the App private key) | `lsg/How To/Rotate the Release App Key` | How To |
| `docs/releases.md` (Distribution materials), `docs/licensing.md` (Distribute builds) | `lsg/Ref/Release Archives` | Reference |
| `docs/licensing.md` (introduction, AGPL obligations) | `lsg/Licensing` | Explanation |
| `docs/licensing.md` (Update dependencies) | `lsg/How To/Update Dependencies` | How To |
| `docs/dependencies.md` | `lsg/Ref/Dependencies` | Reference |
| `docs/openspec-cleanup.md` | `lsg/How To/Finish an OpenSpec Change After Merge` | How To |
| `docs/publishing-cleanup-verification.md` | `lsg/Ref/Publishing Cleanup Verification` | Reference |
| `help-assessment.md` | `lsg/How To/Review Help Examples` | How To |
| `CONTRIBUTING.md` (planning, RuleSync, pull requests, releases) | `lsg/How To/Contribute a Change` | How To |
| `CONTRIBUTING.md` (checks), `docs/releases.md` (Local checks and maintenance) | `lsg/Ref/Project Checks` | Reference |

`CONTRIBUTING.md` stays at the root as a short entry point: install the tools, run `mise run ci`, and the pages above. The release pages describe the automated releases as they run now and keep the key rotation and recovery steps.

#### #330 Move Logseq parity reports into the docs graph

| Source | Page | Kind |
| --- | --- | --- |
| New, drawing on every report | `lsg/Ref/Logseq Compatibility` | Reference |
| `docs/parity/README.md` (introduction, What you need, running the tasks in order) | `lsg/How To/Compare a Build with Logseq` | How To |
| `docs/parity/README.md` (The tasks, The CI check, Change the fixtures or budgets) | `lsg/Ref/Parity Tasks` | Reference |
| `docs/parity/README.md` (quick and recorded `parity:perf` runs), `docs/parity/publication-speed.md` (See where a build spends its time, Run the benchmark), `docs/parity/search-performance.md` (Reproduce) | `lsg/How To/Measure Performance` | How To |
| `docs/parity/publication-speed.md` (Measured results, Rendering each reference copy once, The bottleneck before Issue #290, The CI check) | `lsg/Ref/Publication Speed` | Reference |
| `docs/parity/comparison-harness.md` | `lsg/Ref/Logseq Compatibility/Comparison Harness` | Reference |
| `docs/parity/all-pages.md` | `lsg/Ref/Logseq Compatibility/All Pages` | Reference |
| `docs/parity/app-shell.md` | `lsg/Ref/Logseq Compatibility/App Shell` | Reference |
| `docs/parity/code-and-tables.md` | `lsg/Ref/Logseq Compatibility/Code and Tables` | Reference |
| `docs/parity/flashcards.md` | `lsg/Ref/Logseq Compatibility/Flashcards` | Reference |
| `docs/parity/graph-view.md` | `lsg/Ref/Logseq Compatibility/Graph View` | Reference |
| `docs/parity/hierarchy.md` | `lsg/Ref/Logseq Compatibility/Hierarchy` | Reference |
| `docs/parity/inline-markup.md` | `lsg/Ref/Logseq Compatibility/Inline Markup` | Reference |
| `docs/parity/journals.md` | `lsg/Ref/Logseq Compatibility/Journals` | Reference |
| `docs/parity/media-embeds.md` | `lsg/Ref/Logseq Compatibility/Media Embeds` | Reference |
| `docs/parity/page-outline.md` | `lsg/Ref/Logseq Compatibility/Page Outline` | Reference |
| `docs/parity/queries.md` | `lsg/Ref/Logseq Compatibility/Queries` | Reference |
| `docs/parity/references.md` | `lsg/Ref/Logseq Compatibility/References` | Reference |
| `docs/parity/right-sidebar.md` | `lsg/Ref/Logseq Compatibility/Right Sidebar` | Reference |
| `docs/parity/search.md` | `lsg/Ref/Logseq Compatibility/Search` | Reference |
| `docs/parity/slides.md` | `lsg/Ref/Logseq Compatibility/Slides` | Reference |
| `docs/parity/transclusion.md` | `lsg/Ref/Logseq Compatibility/Transclusion` | Reference |
| `docs/parity/performance.md` (written by `perf.py --record`) | `lsg/Ref/Logseq Compatibility/Performance` | Reference |
| `docs/parity/search-performance.md` (Budgets, Comparison with Pagefind) | `lsg/Ref/Logseq Compatibility/Search Performance` | Reference |

`lsg/Ref/Logseq Compatibility` is the compatibility overview. It names the target, Logseq 0.10's published site, and gives each feature its own block: what `lsg` publishes like Logseq, what differs, and a link to the detailed page. Its Hierarchy section lists every report.

#### #331 Move the Logseq reference boards into the docs graph

| Source | Page | Kind |
| --- | --- | --- |
| `docs/logseq-reference/README.md` | `lsg/Ref/Logseq Published Site` | Reference |
| `docs/logseq-reference/Main.md` | `lsg/Ref/Logseq Published Site/Page View` | Reference |
| `docs/logseq-reference/Journals.md` | `lsg/Ref/Logseq Published Site/Journals Home` | Reference |
| `docs/logseq-reference/AllPages.md` | `lsg/Ref/Logseq Published Site/All Pages` | Reference |
| `docs/logseq-reference/Overlays.md` | `lsg/Ref/Logseq Published Site/Search and More Menu` | Reference |
| `docs/logseq-reference/Flashcards.md` | `lsg/Ref/Logseq Published Site/Flashcards Review` | Reference |
| `docs/logseq-reference/Graph.md` | `lsg/Ref/Logseq Published Site/Graph View` | Reference |
| `docs/logseq-reference/Frontmatter.md` | `lsg/Ref/Logseq Published Site/Frontmatter and Properties` | Reference |
| `docs/logseq-reference/Transclusion.md` | `lsg/Ref/Logseq Published Site/Transclusion` | Reference |
| `docs/logseq-reference/Backlinks.md` | `lsg/Ref/Logseq Published Site/Backlinks` | Reference |
| `docs/logseq-reference/Hierarchy.md` | `lsg/Ref/Logseq Published Site/Namespace Hierarchy` | Reference |
| `docs/logseq-reference/Outline.md` | `lsg/Ref/Logseq Published Site/Outline and Blocks` | Reference |
| `docs/logseq-reference/Inline.md` | `lsg/Ref/Logseq Published Site/Inline Syntax` | Reference |
| `docs/logseq-reference/Reference.md` | `lsg/Ref/Logseq Published Site/Reference Sheet` | Reference |
| `docs/logseq-reference/Prototype.md` | `lsg/Ref/Logseq Published Site/Clickable Prototype` | Reference |

`lsg/Ref/Logseq Published Site` keeps the table that pairs each page with its HTML board in `docs/logseq-reference/html/`. The boards' measurements, class names and strings stay as written, and tables stay Markdown tables inside blocks. The clickable prototype page keeps each of its 24 page sources in a code block, so their links stay literal and create no pages.

#### #334 Link CLI help to pages on the docs site

| Source | Page | Kind |
| --- | --- | --- |
| New | `lsg/How To/Link CLI Help to a Docs Page` | How To |

The page shows a contributor how to link a new help section to a page, so later CLI links follow the contract below.

### Shared files

- `docs/logseq/config.edn`: #325 writes it. Each conversion removes only its own `:hidden` lines.
- `README.md`: each conversion repoints the README's links to the files it deletes at the new page files. #333 rewrites the README.
- `CONTRIBUTING.md`: #325 adds a short section on opening `docs/` in Logseq and building the site. #329 shortens the rest and keeps that section.
- The homepage: #325 writes it with links to the main pages, stubs included. #327 adds the embed of the install commands; later issues edit it only to feature another page.
- Other links to deleted files: each conversion repoints them everywhere outside `openspec/changes/archive/`, including `THIRD_PARTY_NOTICES.md`, `experiments/*/README.md` and the active change `parser-license-evidence`. #333 updates the README assessment skills and the example link in `.rulesync/rules/communication-style.md`.

### Site build

`mise run docs:site [--output DIR]` is an executable task in `.mise/tasks/docs/site`. It builds `lsg` with `cargo build --locked --release --bin lsg`, then runs:

```sh
target/release/lsg publish --graph docs --output "$output" \
  --filename-format triple-lowbar --include lsg --include Diataxis \
  --homepage lsg --search --format json
```

From #326 on, the command also passes the readable page URL option from `readable-page-urls`. Without `--output`, the task publishes into a new `target/docs-site/<UTC time>/site`, as `parity:build` does. It prints the report, prints every diagnostic, and exits non-zero when `diagnostic_count` is not zero, keeping the directory for inspection. The Pages workflow therefore cannot deploy a site with diagnostics.

`mise run docs:check` runs `docs:site` into a temporary directory, then `scripts/docs.py`, which:

- runs every `sh` block in `README.md`;
- checks relative links in `README.md`, `CONTRIBUTING.md`, `THIRD_PARTY_NOTICES.md` and each Markdown file under `docs/` outside `docs/pages/`;
- fails when such a file has no `:hidden` entry;
- fails when a page's kind tag is missing or does not match its title.

`lsg`'s diagnostics cover links between pages, block references and unsupported syntax. #332 adds the Logseq parser check and #334 adds the help link test to `docs:check`. `mise run ci` already runs `docs:check`; the `ci` job's checkout gets `fetch-depth: 0`, since `lsg` reports page dates from a shallow clone as a diagnostic. `mise run docs:links` keeps checking README web links, and #333 adds the README's links to the site.

### Logseq parser check

#332 reuses the Logseq OG file parser that `parser:og-check` runs from `openspec/changes/current-og-relationships/experiments/file-relationships`, at the revision that experiment pins. The check loads `docs/` with its `logseq/config.edn` through that parser and through the SDK, then compares pages, titles, aliases, page references, block references and properties, and prints each difference with its file and item. A small fixture with a deliberate mismatch proves the check fails. The experiment's setup reads Logseq's source from a local checkout; #332 makes the check obtain the pinned source itself when no checkout has it, so it runs in CI and on a new machine. Logseq 0.10 desktop itself is checked by hand in #325, since CI has no desktop app.

### Deployment

`.github/workflows/docs-site.yml`, modeled on Gitpa's Publish workflow, runs on pull requests, pushes to `main` and manual runs:

- A `build` job with `contents: read` checks out full history, sets up mise with the same pinned `jdx/mise-action` as CI, runs `mise run docs:site --output site`, and uploads `site` with `actions/upload-pages-artifact`. On a pull request, that artifact is the preview.
- A `deploy` job runs only for pushes and manual runs on `main`. It needs `build`, has `pages: write` and `id-token: write`, uses the `github-pages` environment, and runs `actions/deploy-pages`. Its concurrency group `pages` has `cancel-in-progress: false`, so deployments run one at a time and a running deployment finishes.

GitHub Pages uses the GitHub Actions source, enabled once with `gh api -X POST repos/codekiln/logseq-gardener/pages -f build_type=workflow`. Pages serves the artifact as it is, so the site needs no `.nojekyll` file. `.github/workflows/release.yml` stays as dist generates it.

### Readable URLs

The site uses readable page URLs from `readable-page-urls`. #326 adds the option to `docs:site`, so #326 starts after #324 as well as #325. The README links of #333 and the help links of #334 then use readable URLs from their first version.

### CLI link contract

- The CLI defines the base URL `https://codekiln.github.io/logseq-gardener/` once.
- A function turns a page title into its URL: the base URL followed by the route that the SDK's readable route function from `readable-page-urls` gives the title. If that change exposes no public function, #334 exposes one in the SDK, so the CLI and the publisher share one rule. `lsg/Ref/Static Search Data` becomes `https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/`.
- Help text names the page title and prints its URL, such as "See lsg/Ref/Static Search Data (https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/) for the data contract." A reader offline can find the same title in the release archive's docs graph. Help prints URLs and never fetches them.
- `tests/docs_links.rs` reads the help of every command, in human and JSON output, collects every URL that starts with the base URL, removes the base, and fails for each one that equals no `url` in `lsg-site.json`; the base URL alone counts as `index.html`. It reads the site directory from `LSG_DOCS_SITE` and is marked `#[ignore]`, because it needs a built site. `docs:check` runs it with `cargo test --locked --test docs_links -- --ignored` against the site it just built, and the test fails when `LSG_DOCS_SITE` is unset.
- `src/help.rs` stops naming `docs/static-search.md`, `docs/publishing.md` and `docs/local-site.md`, and `tests/publish.rs` asserts the new link. The help assessment is refreshed as `lsg/How To/Review Help Examples` describes.

### Machine-written files

- `scripts/parity/perf.py --record` writes `docs/pages/lsg___Ref___Logseq Compatibility___Performance.md` as Logseq outline Markdown: its page properties, a heading block per section, and each table inside its block. The functions that replace one section find that section's heading block.
- `scripts/parity/publish_bench.py --record` keeps writing `docs/parity/publication-speed.json`, and `perf.py --record --search-site` keeps writing `docs/parity/search-performance.json`.
- `scripts/notices.py` keeps writing `docs/dependency-sources.json`.

### Release archives

`dist-workspace.toml` keeps `docs` in `include`, so each archive carries the graph, its data files and its HTML boards. #329 removes `help-assessment.md` from `include` when the file becomes a page. A reader who extracts an archive can open its `docs` folder in Logseq without network access.

### Spec changes

This change modifies `parity-comparison-harness`, `published-graph-view` and `published-page-outline`, whose requirements name `docs/parity/performance.md`, to name the Performance page. It modifies `cli-static-publishing` so publishing help names the plugin pages. #330 and #334 keep these deltas matching what they build, and the cleanup issue [#335](https://github.com/codekiln/logseq-gardener/issues/335) syncs them. Archived changes keep the paths they were written with.

## Risks / Trade-offs

- Help links point at the site for `main`, which can describe behavior newer than an installed release. → The release archive carries the graph of its own commit, and help names page titles a reader can find there.
- A publisher change that adds a diagnostic for syntax the docs use fails the docs build. → The pull request that changes the publisher sees the failure and fixes the publisher or the page.
- Logseq desktop behavior is checked by hand. → The parser check compares Logseq's own parser with the SDK on every CI run.
- Many pull requests edit `:hidden` and `README.md` at once. → Each conversion changes only its own lines; conflicts that remain are in adjacent lines and resolve without judgment.
- Eight issues write pages that link each other. → Stubs from #325 keep every link resolving in any merge order, and the page map names one issue for each page.

## Migration Plan

- #325 merges first: the graph, its stubs, `docs:site` and the checks, with every old document hidden.
- #326 through #332 merge in any order. #326 also waits for #324. Each conversion deletes its documents and their `:hidden` lines.
- #333 follows #326 through #329, and #334 follows #324, #326, #327 and #328.
- #335 verifies, syncs the deltas and archives the change.

Each child is one pull request that a revert undoes. Disabling GitHub Pages in the repository settings takes the site down without touching the graph.

## Resolved Questions

### 1 - Which folder is the graph root, and where do pages live?

Decided by the agents without codekiln: `docs/` is the root, so the release archives, which already include `docs`, carry the graph. Pages live in `docs/pages/`, because `lsg` needs a `pages/` or `journals/` folder in the root and Logseq writes new pages there.

### 2 - Where do non-Markdown files sit?

Decided by the agent without codekiln: at their current paths. Neither `lsg` nor Logseq makes pages from them, and moving them would change scripts, CI comments, `THIRD_PARTY_LICENSES.txt` and archive contents with no gain for readers.

### 3 - How do unconverted files stay out of the graph?

Decided by the agents without codekiln: one `:hidden` entry per file, each on its own line, removed by the pull request that converts the file. `docs:check` fails when an unconverted file has no entry.

### 4 - What is the homepage?

Decided by the agent without codekiln: the topic page `lsg`, with the alias `Logseq Gardener`. `--include lsg` already selects it, its Hierarchy section lists every page, and Logseq opens it through `:default-home`.

### 5 - How are explanation pages named?

Decided by the agent without codekiln: `lsg/<concept>`, such as `lsg/Architecture`, so every documentation page sits in the `lsg` namespace and one `--include` selects them.

### 6 - What do the kind tags point at?

Decided by the agent without codekiln: kind index pages under `Diataxis/` in the docs graph, selected with `--include Diataxis`. A tag without a selected page is an unresolved reference, which `lsg` reports.

### 7 - Which tag do explanation pages carry?

Decided by the agent without codekiln: `[[Diataxis/Explanation]]`, which codekiln's convention allows. A tag that names the kind page's own title keeps that page's tagged-pages list working in Logseq without relying on alias resolution. The page's `alias:: [[Diataxis/Concept]]` lets the convention's other name resolve.

### 8 - How do pages link repository files?

Decided by the agent without codekiln: with GitHub URLs on `main`. `lsg` reads a relative link as an asset and reports a diagnostic when the link names anything other than a supported file under `assets/`.

### 9 - What fails the docs build?

Decided by the agents without codekiln: any `lsg` diagnostic. `docs:site` itself fails, so neither CI nor the Pages workflow can pass a site with diagnostics. CI checks out full history, because `lsg` reports page dates from a shallow clone.

### 10 - How is a page written when two issues' documents describe its subject?

Decided by the agent without codekiln: the page map gives each page one issue and draws each page from that issue's documents. A fact two documents repeat stays on the page that owns the subject.

### 11 - When does the site get readable URLs?

Decided by the agent without codekiln: in #326, which therefore also waits for #324. README and help links then never point at hashed page files.

### 12 - How does the help link test reach the built site?

Decided by the agent without codekiln: `docs:check` passes the site it built through `LSG_DOCS_SITE` to an ignored integration test, which fails when the variable is missing.

### 13 - What happens to `help-assessment.md`?

Decided by the agents without codekiln: it becomes `lsg/How To/Review Help Examples`, `CONTRIBUTING.md` links that page, and `dist-workspace.toml` drops the file.

### 14 - Who records the spec path changes for the performance page?

Decided by the agent without codekiln: this change carries the deltas now, and #330 keeps them matching the file `perf.py` writes.

### 15 - How does Logseq desktop leave the checkout clean?

Decided by the agent without codekiln: the repository commits `pages/contents.md` and `logseq/custom.css`, ignores Logseq's backup and recycle folders, and turns journals off.

### 16 - How does the parser check get Logseq's source in CI?

Decided by the agent without codekiln: #332 makes the check obtain the pinned revision itself when no local checkout has it, so CI and new machines need no manual setup.

## Open Questions

None.
