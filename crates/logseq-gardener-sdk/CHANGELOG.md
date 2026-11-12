# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.50](https://github.com/codekiln/logseq-gardener/compare/v0.0.49...v0.0.50) - 2026-11-12

### Changed

- prepare v0.0.50 (#617)

## [0.0.49](https://github.com/codekiln/logseq-gardener/compare/v0.0.48...v0.0.49) - 2026-11-12

### Added

- make published gardens installable as apps (#611)

## [0.0.48](https://github.com/codekiln/logseq-gardener/compare/v0.0.47...v0.0.48) - 2026-11-09

### Added

- show the full-recording link and playback status in the episode player (#594)

### Performance

- bring the page-load values back within their budgets (#597)

## [0.0.47](https://github.com/codekiln/logseq-gardener/compare/v0.0.46...v0.0.47) - 2026-11-09

### Added

- write rel=canonical on pages published with --readable-urls (#592)

## [0.0.46](https://github.com/codekiln/logseq-gardener/compare/v0.0.45...v0.0.46) - 2026-11-09

### Fixed

- redraw the Page graph card after a plugin navigation (#583)

## [0.0.45](https://github.com/codekiln/logseq-gardener/compare/v0.0.44...v0.0.45) - 2026-11-09

### Fixed

- reconcile older publishing specs with newer ones and the code (#577)

## [0.0.44](https://github.com/codekiln/logseq-gardener/compare/v0.0.43...v0.0.44) - 2026-11-09

### Fixed

- keep page dates when a page file or graph folder is renamed (#571)

## [0.0.43](https://github.com/codekiln/logseq-gardener/compare/v0.0.42...v0.0.43) - 2026-11-09

### Added

- select every page with --include-all (#564)

### Performance

- keep other search shards when a page is added, removed or renamed (#559)

## [0.0.42](https://github.com/codekiln/logseq-gardener/compare/v0.0.41...v0.0.42) - 2026-11-08

### Added

- publish pages that exist only as link targets (#553)

## [0.0.41](https://github.com/codekiln/logseq-gardener/compare/v0.0.40...v0.0.41) - 2026-11-08

### Added

- seek the recording from timestamp-led bullets under Chapters (#549)

## [0.0.40](https://github.com/codekiln/logseq-gardener/compare/v0.0.39...v0.0.40) - 2026-11-08

### Added

- link the previous and next pages in the header (sequence-header plugin) (#548)

## [0.0.39](https://github.com/codekiln/logseq-gardener/compare/v0.0.38...v0.0.39) - 2026-11-08

### Added

- seek the recording from timestamped chapter headings (#544)

## [0.0.38](https://github.com/codekiln/logseq-gardener/compare/v0.0.37...v0.0.38) - 2026-11-08

### Added

- add the podcast-episode plugin with a persistent player (#540)

### Performance

- rewrite only the changed pages and search files after an edit (#537)

## [0.0.37](https://github.com/codekiln/logseq-gardener/compare/v0.0.36...v0.0.37) - 2026-11-08

### Added

- run advanced queries at build time (#485 step 3) (#529)

## [0.0.36](https://github.com/codekiln/logseq-gardener/compare/v0.0.35...v0.0.36) - 2026-11-08

### Fixed

- spell a page as Logseq does when a block writes a property name and a reference in another case (#526)

## [0.0.35](https://github.com/codekiln/logseq-gardener/compare/v0.0.34...v0.0.35) - 2026-11-08

### Fixed

- spell an alias name as Logseq does when files refer to it in another case (#522)

## [0.0.34](https://github.com/codekiln/logseq-gardener/compare/v0.0.33...v0.0.34) - 2026-11-08

### Fixed

- spell a repeated alias name as Logseq does (#516)

## [0.0.33](https://github.com/codekiln/logseq-gardener/compare/v0.0.32...v0.0.33) - 2026-11-08

### Fixed

- read a date name with a wrong weekday as Logseq does (#510)

## [0.0.32](https://github.com/codekiln/logseq-gardener/compare/v0.0.31...v0.0.32) - 2026-11-08

### Fixed

- drop the boundary slashes of a page name, as Logseq does (#504)

## [0.0.31](https://github.com/codekiln/logseq-gardener/compare/v0.0.30...v0.0.31) - 2026-11-07

### Fixed

- create the namespace parents of an alias name, as Logseq does (#498)

## [0.0.30](https://github.com/codekiln/logseq-gardener/compare/v0.0.29...v0.0.30) - 2026-11-07

### Fixed

- read no page from a file that holds only whitespace, as Logseq does (#477)

## [0.0.29](https://github.com/codekiln/logseq-gardener/compare/v0.0.28...v0.0.29) - 2026-11-04

### Added

- report rejected property names as a publish diagnostic (#475)

## [0.0.28](https://github.com/codekiln/logseq-gardener/compare/v0.0.27...v0.0.28) - 2026-11-02

### Added

- redirect old page URLs to readable URLs (#471)

## [0.0.27](https://github.com/codekiln/logseq-gardener/compare/v0.0.26...v0.0.27) - 2026-11-02

### Fixed

- record an alias on the page it names too, as Logseq does (#463)

## [0.0.26](https://github.com/codekiln/logseq-gardener/compare/v0.0.25...v0.0.26) - 2026-11-02

### Fixed

- hide the bullet of a block with no text, as Logseq does (#457)

## [0.0.25](https://github.com/codekiln/logseq-gardener/compare/v0.0.24...v0.0.25) - 2026-11-02

### Fixed

- title a page that refers to itself in another spelling as Logseq does (#454)

## [0.0.24](https://github.com/codekiln/logseq-gardener/compare/v0.0.23...v0.0.24) - 2026-11-02

### Fixed

- keep a shared block id on the block Logseq reads first (#448)

## [0.0.23](https://github.com/codekiln/logseq-gardener/compare/v0.0.22...v0.0.23) - 2026-11-02

### Fixed

- show Logseq's warning on a property name it rejects (#446)

## [0.0.22](https://github.com/codekiln/logseq-gardener/compare/v0.0.21...v0.0.22) - 2026-11-02

### Fixed

- drop the property names Logseq rejects, with their references (#440)

## [0.0.21](https://github.com/codekiln/logseq-gardener/compare/v0.0.20...v0.0.21) - 2026-11-02

### Fixed

- keep the spaces inside a page reference's brackets as Logseq does (#432)

## [0.0.20](https://github.com/codekiln/logseq-gardener/compare/v0.0.19...v0.0.20) - 2026-11-02

### Fixed

- label property links with the name as written, as Logseq does (#429)

## [0.0.19](https://github.com/codekiln/logseq-gardener/compare/v0.0.18...v0.0.19) - 2026-11-02

### Fixed

- list every alias in the page properties box as Logseq does (#424)

## [0.0.18](https://github.com/codekiln/logseq-gardener/compare/v0.0.17...v0.0.18) - 2026-11-02

### Fixed

- name the namespace parent of a reference with a space before its slash as Logseq does (#422)

## [0.0.17](https://github.com/codekiln/logseq-gardener/compare/v0.0.16...v0.0.17) - 2026-11-01

### Fixed

- read nested property references and built-in page names as Logseq does (#415)

## [0.0.16](https://github.com/codekiln/logseq-gardener/compare/v0.0.15...v0.0.16) - 2026-11-01

### Fixed

- show page property rows that link to a page with no file (#413)

## [0.0.15](https://github.com/codekiln/logseq-gardener/compare/v0.0.14...v0.0.15) - 2026-11-01

### Changed

- prepare v0.0.14 (#399)

### Fixed

- report each publish diagnostic once, under the file that holds it (#406)
- keep the remembered Properties choice after plugin host navigation (#405)

### Performance

- share page styles and compress reference fragments (#400)

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
