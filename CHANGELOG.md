# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- measure the right sidebar in the parity harness (#236)
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
- add the Logseq parity comparison harness (#131)
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
- *(cli)* publish selected namespaces through lsg
- publish selected Logseq namespaces as local HTML
- derive Markdown page titles through the SDK
- load Markdown garden sources through the SDK
- select publishing namespaces in the SDK (#13)
- add the lsg CLI and Rust SDK

### Changed

- 🐛 fix: distinguish hosted recording links from local downloads
- Remove review administration from publishing verification
- Clarify page-title decoding and validation sources

### Fixed

- support the weekday token E in journal title formats (#276)
- honor Logseq's all-pages-public setting when selecting pages (#273)
- remember the reader’s Properties choice (#183)
- keep outline disclosure controls reachable (#181)
- count every published page in parity:compare (#253)
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
