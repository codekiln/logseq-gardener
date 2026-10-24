# Home: Journals

This board shows the published site's home route: the Journals screen, a stack of day pages, newest first.

Overall: board 1440 wide, min-height 1100px, font 'Source Sans 3', text `#11181c`, background `#ffffff`, 16px. Links `#106ba3`, grey brackets `#6b7280`.

## Header

Header height 48px, flex, space-between, padding `0 12px`, background `rgba(255,255,255,0.9)`, border-bottom `1px solid #e9ecef` (not sticky on this board).

- Left (gap 4px): icon buttons "Toggle left sidebar" and "Search".
- Right (gap 4px): text link "Graph" (14px, weight 500, `#687076`, padding `8px 10px`, radius 6px), icon buttons "More" and "Toggle right sidebar".
- Icon buttons: 36x36, radius 6px, colour `#687076`.

## Left sidebar

`nav` (aria-label "Left sidebar"): flex `1 1 246px`, background `#f8f9fa`, border-right `1px solid #e9ecef`, padding `12px 8px`, gap 2px, min-height 1052px. Nav buttons: 15px, gap 10px, padding `6px 10px`, radius 6px, min-height 32px, hover `#e9ecef`.

- "Journals": active (background `#e9ecef`, icon stroke `#3e63dd`).
- "Whiteboards".
- "Flashcards" with badge "430" (12px, weight 600, background `#e9ecef`, colour `#3a3f44`, radius 9px, padding `1px 7px`).
- "All pages" (links to the All pages board).
- Label "FAVORITES" (12px, weight 600, letter-spacing 0.06em, `#687076`).
- Favorites (14px, 6px dot `#9ba1a6`): fd, gh-dash, tuicr, LazyVim, Obsidian, chezmoi, mise, nvim, tmux (links to the Page view board), yazi, Lazygit. No favorite is highlighted here.

## Main column

`main`: padding `32px 24px 64px`; wrapper max-width 810px, centred, column, gap 48px between day articles.

Each day is an `article`: `h1` 30px, weight 300, line-height 1.3, margin-bottom 12px, containing a link in colour `#11181c`. Top-level blocks are 28px, weight 300, line-height 1.5 page links with grey brackets. Block rows: gap 8px, line-height 1.6, margin `2px 0`; bullet dot 6px `#9ba1a6`; children indented margin-left 7px, padding-left 22px, border-left `1px solid #e9ecef`.

### 2026-10-23 Fri

- [[Filed]]
  - [[GitP/A/Session/26/10/22-Thu/Asset/Audio/Trimmed/mp3]]
  - [[Obsidian/Q/Should a team vault in git commit its community plugins' code, or only the list of plugins?]]
  - [[My/Pref/Dev/Tool/Obsidian/Plugins in Git]]
- [[Updated]]
  - GitP episodes
    - [[GitP/A/Session/26/09/25-Fri]]
    - [[GitP/A/Session/26/10/22-Thu]]
  - [[My/Pref/Dev/mise/Tasks]]
  - [[My/Pref/Writing/Use Plain language]]

### 2026-10-22 Thu

- [[Updated]]
  - Music
    - [[GitP/How To/Prepare Podcast Metadata]]
    - [[Microfreak/Preset/016 Voidom]]
    - [[Logseq/Entity/Preset/Synth/Microfreak]]
- [[Filed]]
  - [[GitP/A/Session/26/10/22-Thu]]
