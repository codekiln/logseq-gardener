## ADDED Requirements

### Requirement: Break long words to fit the screen

The publisher SHALL break a word in an outline block that does not fit the block's width, as Logseq's block content does, so no outline block makes the page wider than the screen. A word that fits SHALL NOT break. Code blocks SHALL keep their own horizontal scrolling.

#### Scenario: Long page reference on a phone
- **WHEN** a reader opens `GitP/A/Session/26/10/24-Sat` at 390×844 with every block expanded
- **THEN** the page is 390px wide and each `Ableton/Live/Mode/Arrangement` link ends within the screen
