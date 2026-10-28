## MODIFIED Requirements

### Requirement: Show unknown macros as warnings

The publisher SHALL render a macro it has no handler for as `span.warning` with title `Unsupported macro name: <name>` and text `{{<name> <arguments joined by ", ">}}`, in Logseq's warning colours for the active theme with `.1em .4em` padding, and SHALL keep the build diagnostic. `{{renderer …}}` SHALL render nothing and report a diagnostic. The macros `video`, `youtube`, `vimeo`, `bilibili` and `youtube-timestamp` SHALL render as published media embeds. The macros `embed`, `query`, `cards`, `cloze`, `namespace` and `tweet` SHALL keep their current rendering until the issues that own them change it.

#### Scenario: Unknown macro
- **WHEN** a page contains `{{pdf x}}`
- **THEN** the page shows a warning chip with text `{{pdf x}}` and title `Unsupported macro name: pdf`

#### Scenario: Owned macro
- **WHEN** a page contains `{{query (todo now)}}`
- **THEN** it renders as it does today

#### Scenario: Media macro
- **WHEN** a page contains `{{vimeo https://vimeo.com/76979871}}`
- **THEN** it renders a video player area and no warning chip
