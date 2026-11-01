# Contributing

Install the pinned tools with `mise install`, then run `mise run ci`, which runs the same checks as GitHub Actions. List tasks with `mise tasks`; use `mise run <task> --help` for arguments.

To test a change, run `mise run build:release` and then `./target/release/lsg`. In a shell with `mise activate`, under `mise exec` and in mise tasks, a bare `lsg` also runs this checkout's build once it exists, and a globally installed `lsg` before then. A shell that reaches tools through mise shims always runs the installed `lsg`. `command -v lsg` shows which binary a bare `lsg` runs.

The contributor guides are pages in the documentation graph:

- [lsg/How To/Contribute a Change](docs/pages/lsg___How%20To___Contribute%20a%20Change.md) takes a change from its OpenSpec plan to a pull request with a Conventional Emoji title.
- [lsg/Ref/Project Checks](docs/pages/lsg___Ref___Project%20Checks.md) lists the checks `mise run ci` runs and the task that runs each one alone.
- [lsg/How To/Review Help Examples](docs/pages/lsg___How%20To___Review%20Help%20Examples.md) refreshes the help assessment after a change to the help text.
- [lsg/How To/Update Dependencies](docs/pages/lsg___How%20To___Update%20Dependencies.md) keeps the license notices current when a dependency changes.
- [lsg/How To/Finish an OpenSpec Change After Merge](docs/pages/lsg___How%20To___Finish%20an%20OpenSpec%20Change%20After%20Merge.md) syncs and archives an OpenSpec change after its implementation merges.
- [lsg/Architecture](docs/pages/lsg___Architecture.md) explains how the SDK and the command line divide the work.
- [lsg/Releases](docs/pages/lsg___Releases.md) explains how a merged pull request becomes a release.

## Documentation

The documentation is a Logseq graph in `docs/`, and `lsg` publishes it as the documentation site. To edit it in Logseq 0.10 desktop, open the graph menu at the top of the left sidebar, choose Add new graph, then Choose a folder, and select `docs`. Each page is a file in `docs/pages/`. [lsg/How To/Edit the Documentation](docs/pages/lsg___How%20To___Edit%20the%20Documentation.md) explains how to name, tag and write a page, and how to turn one of the remaining Markdown documents in `docs/` into pages.

`mise run docs:site` builds `lsg` from the checkout and publishes the graph, with search, into a new folder under `target/docs-site/`; open its `index.html` to read the site. The task fails when `lsg` reports any diagnostic. `mise run docs:check`, which `mise run ci` runs, builds the site the same way, runs the README commands, and checks page tags, the `:hidden` entries of the remaining Markdown documents, and local links.
