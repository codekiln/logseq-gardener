## Context

A readable build writes a page to the folders its title names. On macOS and Windows, `CLAUDE/` and `Claude/` are one folder, so the build used to stop whenever two titles spelled a folder two ways. Logseq resolves page names without case, so a garden such as the encode garden has many of these pairs on purpose.

## Decisions

- **The folder takes the spelling of the page it holds.** Logseq shows the parent of `Claude/Artifact` as the existing page `CLAUDE`, so the URL follows the page a reader lands on. A folder with no page takes the spelling that sorts first, which is the spelling `namespace_destinations` already gives a generated namespace page.
- **Spelling is chosen segment by segment.** A folder sits inside its parent's chosen spelling, so `tmux/session/Window/Pane` becomes `tmux/Session/Window/Pane` when only `tmux/session` is spelled two ways.
- **Titles keep their spelling.** Only routes change; headings, search titles, the graph's labels and `lsg-site.json` titles stay as authored.
- **The graph script gets the spellings inline.** The readable graph script computes a node's route from its title. A `Map` of folded path to spelling, written into the script, lets it compute the same route. It holds only folders spelled more than one way, 258 entries for the encode garden. A `Map` keeps a folder named `__proto__` working.
- **Derived files follow the route.** Reference fragments and neighbourhood files take their name from the route's title path, so a moved page's derived files use the respelled path's digest on both the Rust and the JavaScript side.

## Risks

- A link someone copied from a build that never existed cannot break, since these gardens could not build with readable URLs before.
- Adding a page later can change a folder's spelling: creating a page `claude` beside `Claude/Artifact` moves the folder to `claude/`. The `p-<digest>.html` redirects in the encode garden keep older links working.
