# Design

## Context

`docs/` holds the Logseq graph (`pages/` and `logseq/`) and three things that are not graph pages: the dependency inventory `docs/dependency-sources.json`, the parity reports and data in `docs/parity/`, and the Logseq reference boards in `docs/logseq-reference/html/`. Scripts read the data files at fixed paths, and graph pages link the boards by their GitHub URLs. The graph's `:hidden` list keeps the three parity reports out of Logseq and `lsg`.

## Goals / Non-Goals

**Goals:**

- Move the graph to `logseq-gardener-docs/` with its history, and keep every task, check, link and release archive working.
- Keep the change a rename plus path updates, so it merges cleanly beside other open work on the graph and the specs.

**Non-Goals:**

- Page dates that follow a rename. `lsg` reads page dates with `git log --no-renames`, so the move restarts every docs page's created and updated dates; following renames is its own change to the SDK.

## Decisions

### Move only the graph

`git mv` moves `docs/pages/` and `docs/logseq/` into `logseq-gardener-docs/`. The data files, parity reports and reference boards stay in `docs/`, as the issue asks. Moving them too would break the GitHub URLs that graph pages, archived changes and outside readers use, and the data files are not part of the graph.

### Empty the `:hidden` list and keep the rule

The parity reports are now outside the graph folder, so Logseq and `lsg` never load them and their `:hidden` entries would name files that do not exist. The list becomes `[]`. `docs:check` still requires a `:hidden` entry for a Markdown file inside `logseq-gardener-docs/` outside `pages/` and `logseq/`, and it now also checks the relative links of every Markdown file under `docs/`, which it checked before as unconverted documents.

### Ship both folders in release archives

`dist-workspace.toml` includes `logseq-gardener-docs` and keeps `docs`, so an archive still carries the dependency inventory that the third-party notices point to, together with the graph.

### Leave OpenSpec records as written

Archived changes keep the `docs/` paths they were written with; they record what was true then. Active changes keep their task and verification text for the same reason. The one exception is a relative Markdown link that would otherwise point at nothing (`parser-license-evidence/experiments/lsdoc-licenses/README.md`). The active change `publish-reference-only-pages` is left untouched because a parallel pull request archives it.

### Update the main spec's Purpose directly

The `docs-graph` Purpose names the folder. OpenSpec ignores a Purpose in a delta, so the main spec's Purpose changes in this pull request, and the requirement changes wait for the sync after merge.

## Risks / Trade-offs

- [Every docs page shows the move's commit as its created and updated date on the site] → Accepted for now; a follow-up issue asks `lsg` to follow renames when it reads page dates, which would restore the earlier dates without another change here.
- [A contributor's Logseq desktop still lists the old `docs` graph] → `CONTRIBUTING.md` and `lsg/How To/Edit the Documentation` say to add `logseq-gardener-docs` as the graph.
- [Open pull requests that edit `docs/pages/` conflict] → Git follows the rename when it merges; a conflict that remains is a path conflict that the later pull request resolves by moving its edit.

## Resolved Questions

### 1 - Do the files in `docs/` that are not graph pages move with the graph?

No. Issue #387 asks to leave them where they are unless they are graph pages, and none of them is.

## Open Questions

None.
