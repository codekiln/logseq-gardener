# Verification: read-task-markers-and-alias-values

Logseq's parser check now agrees with the SDK on task markers, priorities, and the values of `alias::`, `aliases::` and `tags::`. Checked on 2026-11-01 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` (before) and from this branch (after).

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| Case graph: every task marker, lower-case `todo`, `TODOs`, priorities before and after a marker, `alias:: Plain Name, Other Name`, `alias:: #[[Tag Alias]], #short`, `alias:: [[Outer [[Inner]] Name]]`, a block-level `alias::` | 47 differences | agree: 5 pages, 24 blocks |
| Edge graph: `tags:: foo [[bar]] baz`, `tags:: **bold**`, quoted and numeric values, `aliases:: x, y`, `#[[d e]]`, a nested reference in `type::`, a heading task, `NOW [#C]` | 27 differences | 3 differences, all in [Issue #378](https://github.com/codekiln/logseq-gardener/issues/378) |
| `scripts/logseq-parser/fixtures/tag-alias` | 5 differences | agree: 2 pages, 3 blocks |
| `docs` | agree: 89 pages, 4440 blocks | agree: 89 pages, 4440 blocks |
| Encode garden | 1972 differences | 882 differences |

On the encode garden, the block references that Logseq reads and the SDK did not fall from 1104 to 24. The remaining 882 differences come from other rules: aliases that another page declares, journal dates in property values, backticks inside references, and the namespace parent `MacOS ` of `[[MacOS /Applications Folder]]`, which keeps its trailing space in Logseq.

## Publishing the encode garden

`lsg publish --journals --filename-format triple-lowbar` with one `--include` for each of the 1045 top-level names, run twice with each build:

| | Before | After |
| --- | --- | --- |
| Pages | 8731 | 8731 |
| Journals | 512 | 512 |
| Diagnostics | 35671 | 35671 |
| Wall time | 16.1 s, 20.5 s | 18.4 s, 18.7 s |

Five published pages change. `cybernetics` and `Spotify/Playlist` gain a Pages tagged with section, because `tags:: [[philosophy/process]] [[ai/co-creation]] [[creativity]] [[cybernetics]]` and `tags:: #Interviews, #[[Spotify Playlists]]` now name each page. `Programming`, `Diagram` and `Diagram/Mermaid` each gain one linked reference. `mise` keeps 589 linked references. The page graph data changes because task markers now add references to the built-in pages, which the graph shows only with built-in pages on.
