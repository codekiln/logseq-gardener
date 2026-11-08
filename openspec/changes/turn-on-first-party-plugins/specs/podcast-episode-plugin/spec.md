# Spec Delta

## Purpose

A garden that publishes podcast episodes can turn on `podcast-episode`, so a reader can keep a recording playing while moving between pages and can start the recording from a chapter heading, a timestamp-led bullet or a dialogue line.

## ADDED Requirements

### Requirement: Find an episode's recording

The plugin SHALL apply to a published page whose `logseq-entity::` value names `Logseq/Entity/Podcast/Episode`, matched as Logseq matches page names. The recording SHALL be the first audio embed among the descendants of the page's first heading block. A page of that type without such an embed SHALL get no entry and one build diagnostic naming the page. The entry SHALL hold the recording URL and the first heading's text as the title.

#### Scenario: Episode with a recording

- **WHEN** `GitP/A/Session/26/10/24-Sat` is published with the plugin on
- **THEN** its entry holds the URL of `GitP___A___Session___26___10___24-Sat___Asset___Audio___Full.mp3` and the title `The Unstable over USB`

#### Scenario: Older episode shape

- **WHEN** `GitP/A/Session/26/09/24-Thu`, typed only with `[[Logseq/Entity/Podcast/Episode]]`, is published with the plugin on
- **THEN** its entry holds the URL of its `Synth___Full.mp3` recording

#### Scenario: Type without a recording

- **WHEN** `GitP/A` carries the episode type and has no audio embed under its first heading
- **THEN** it gets no entry, requests no plugin file, and the build lists one diagnostic naming `GitP/A`

### Requirement: Read an episode's chapters

The entry SHALL list chapters in page order: each H3 to H6 heading whose text starts with a `M:SS`, `MM:SS` or `H:MM:SS` timestamp, and each bullet at any depth under a `## Chapters` heading whose text starts with one. A chapter SHALL hold its block anchor, its start in seconds, its title and the enclosing chapter. A timestamp elsewhere in a block or inside code SHALL stay text. The entry SHALL stay within 64 KB.

#### Scenario: Chapter heading

- **WHEN** `GitP/A/Session/26/11/01-Sun` has the heading `18:17 Enter Microfreak/Preset/260 8bitfi`
- **THEN** its entry has a chapter starting at 1,097 seconds with that heading's anchor

#### Scenario: Timestamp-led bullet

- **WHEN** a bullet under `## Chapters` on `GitP/A/Session/26/11/01-Sun` reads `05:15 - the bass comes in`
- **THEN** its entry has a chapter starting at 315 seconds whose enclosing chapter is the heading `05:12 Infini Pad as bass`

#### Scenario: Timestamp in code or mid-text

- **WHEN** a block under `## Chapters` reads `` `05:15` in code `` or `the second voice joins at 01:28`
- **THEN** that block adds no chapter

#### Scenario: Largest episode

- **WHEN** `GitP/A/Session/26/10/20-Tue`, with about 120 timestamps, is published with the plugin on
- **THEN** its entry is under 64 KB and the page stays within the `[desktop]` and `[phone]` budgets

### Requirement: Keep the recording playing across pages

When a reader starts an episode's recording, the module SHALL play it in one `<audio>` element in a persistent region outside `<main>`, with the native controls and the episode's title linked to its page. The recording SHALL keep playing through same-document navigation, Back and Forward. Opening another episode SHALL leave the playing recording in place until the reader starts the other episode's recording.

#### Scenario: Follow a link while playing

- **WHEN** a reader starts the recording on `GitP/A/Session/26/10/24-Sat` and follows a link to `GitP/A`
- **THEN** the same `<audio>` element keeps playing without restarting

#### Scenario: Open another episode

- **WHEN** the recording of `GitP/A/Session/26/10/24-Sat` is playing and the reader opens `GitP/A/Session/26/10/29-Thu` and then starts that page's recording
- **THEN** the player keeps the first recording until the reader starts the second, then plays the second from its start

### Requirement: Seek from chapters and timestamp-led bullets

On an episode page, the module SHALL turn the timestamp of each chapter in the entry into a control that plays the recording in the player from that chapter's start. The player SHALL show the current chapter and its enclosing chapters while the recording plays.

#### Scenario: Seek from a heading

- **WHEN** a reader selects the timestamp of `18:17 Enter Microfreak/Preset/260 8bitfi` on `GitP/A/Session/26/11/01-Sun`
- **THEN** the player plays the recording from 18:17 and shows that chapter

#### Scenario: Seek from a bullet

- **WHEN** a reader selects `05:15` in `05:15 - the bass comes in` on `GitP/A/Session/26/11/01-Sun`
- **THEN** the player plays the recording from 5:15 and shows the bullet's chapter with its enclosing chapter

#### Scenario: Hand-annotated chapter

- **WHEN** a reader selects a timestamp-led bullet in the hand-annotated chapter of `GitP/A/Session/26/10/29-Thu`
- **THEN** the player plays the recording from that bullet's time

### Requirement: Seek from dialogue lines

On an episode page, the module SHALL handle the `lsg:media-seek` event that a dialogue line fires: it SHALL cancel the event and play the page's recording in the player from the line's time.

#### Scenario: Dialogue line

- **WHEN** a reader selects a dialogue timestamp on `GitP/A/Session/26/10/24-Sat`
- **THEN** the player plays the recording from that time and the page's inline `<audio>` stays paused

### Requirement: Page without the module

When the module is absent, fails to load or throws during setup, an episode page SHALL show its inline `<audio>` element and its timestamps as text, as the same page does without the plugin.

#### Scenario: Module fails

- **WHEN** the module's request fails on `GitP/A/Session/26/10/24-Sat`
- **THEN** the inline recording plays from its own controls and dialogue lines seek it, as without the plugin
