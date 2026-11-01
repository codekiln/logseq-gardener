# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.14](https://github.com/codekiln/logseq-gardener/compare/v0.0.13...v0.0.14) - 2026-11-01

### Changed

- prepare v0.0.14 (#398)

## [0.0.13](https://github.com/codekiln/logseq-gardener/compare/v0.0.12...v0.0.13) - 2026-11-01

### Fixed

- keep indentation inside nested #+BEGIN_SRC blocks (#390)
- show alias, public and other page properties as Logseq does (#389)

## [0.0.12](https://github.com/codekiln/logseq-gardener/compare/v0.0.11...v0.0.12) - 2026-11-01

### Fixed

- give case-folded namespace folders one readable spelling (#383)

## [0.0.11](https://github.com/codekiln/logseq-gardener/compare/v0.0.10...v0.0.11) - 2026-11-01

### Fixed

- read task markers and alias values as Logseq does (#379)

## [0.0.10](https://github.com/codekiln/logseq-gardener/compare/v0.0.9...v0.0.10) - 2026-11-01

### Changed

- prepare v0.0.9 (#373)

### Fixed

- publish unsafe titles at their digest file under --readable-urls (#375)

## [0.0.9](https://github.com/codekiln/logseq-gardener/compare/v0.0.8...v0.0.9) - 2026-11-01

### Changed

- prepare v0.0.9 (#368)

## [0.0.8](https://github.com/codekiln/logseq-gardener/compare/v0.0.7...v0.0.8) - 2026-11-01

### Added

- publish pages at readable namespace URLs (#341)

## [0.0.7](https://github.com/codekiln/logseq-gardener/compare/v0.0.6...v0.0.7) - 2026-11-01

### Fixed

- keep fragment targets below the sticky header (#336)

## [0.0.6](https://github.com/codekiln/logseq-gardener/compare/v0.0.5...v0.0.6) - 2026-11-01

### Fixed

- open Linked References crumbs and bullets in the sidebar (#319)

## [0.0.5](https://github.com/codekiln/logseq-gardener/compare/v0.0.4...v0.0.5) - 2026-11-01

### Fixed

- mark every page link with page-ref for Shift-click (#313)

### Performance

- render each Linked References context once (#308)

## [0.0.4](https://github.com/codekiln/logseq-gardener/compare/v0.0.3...v0.0.4) - 2026-10-31

### Added

- show graph loading with cancel and retry (#299)

### Fixed

- open the site root's home page from nested routes (#295)

## [0.0.3](https://github.com/codekiln/logseq-gardener/compare/v0.0.2...v0.0.3) - 2026-10-31

### Fixed

- open rewritten page links in the right sidebar on Shift-click (#297)

## [0.0.2](https://github.com/codekiln/logseq-gardener/compare/v0.0.1...v0.0.2) - 2026-10-31

### Added

- measure and protect publication speed (#289)

## [0.0.1](https://github.com/codekiln/logseq-gardener/releases/tag/v0.0.1) - 2026-10-31

### Added

- add build plugins that read a publication view (#281)
- load Markdown from every garden folder, as Logseq does (#262)
- publish a site manifest and an optional plugin host (#260)
- publish Logseq flashcards (#248)
- render Logseq queries at build time (#240)
- add Logseq's video embeds and YouTube timestamps (#231)
- add Logseq's View as slides (#227)
- add Logseq's graph view and page graph (#225)
- add Logseq's right sidebar (#221)
- match Logseq search with a sharded index (#216)
- find a page's source file with lsg lookup (#212)
- publish Logseq's All pages table (#202)
- match Logseq's namespace hierarchy section (#201)
- match Logseq's linked references, filters, unlinked references and tagged pages (#187)
- render block references and embeds like Logseq (#184)
- render fenced code and tables like Logseq (#177)
- render inline markup like Logseq (#174)
- seek original recordings from dialogue timestamps (#157)
- publish Logseq's page outline with collapse and block zoom (#158)
- publish Logseq's journals and the journals feed (#149)
- add accessible lazy search dialog (#144)
- add selected-source search contract (#141)
- add Logseq's header, left sidebar and More menu (#138)
- render compact backlink source cards (#104)
- publish linked collapsible page properties (#101)
- add system-aware appearance controls (#84)
- label namespaces and render authored page navigation (#82)
- add filtered published page backlinks
- preserve authored collapsed outlines (#66)
- render full Logseq hierarchies at page bottom
- render published namespace hierarchies
- render selected transclusions and configure the homepage
- *(site)* support custom local stylesheets
- resolve selected garden page aliases
- navigate selected published outline references
- render selected garden audio and HTTPS artwork
- publish selected Logseq namespaces as local HTML
- derive Markdown page titles through the SDK
- load Markdown garden sources through the SDK
- select publishing namespaces in the SDK (#13)
- add the lsg CLI and Rust SDK

### Changed

- 🐛 fix: distinguish hosted recording links from local downloads
- Clarify page-title decoding and validation sources

### Fixed

- support the weekday token E in journal title formats (#276)
- honor Logseq's all-pages-public setting when selecting pages (#273)
- remember the reader’s Properties choice (#183)
- keep outline disclosure controls reachable (#181)
- start a block at every unbulleted heading line (#244)
- break long words in outline blocks on phones (#238)
- publish pages whose titles keep spaces around a slash (#196)
- omit generated intermediate hierarchy rows (#133)
- clarify backlink page and context counts (#98)
- space hierarchy path separators
- render email autolinks as mailto links (#70)
- preserve email addresses in published fallbacks

### Performance

- validate static search loading and latency (#156)
