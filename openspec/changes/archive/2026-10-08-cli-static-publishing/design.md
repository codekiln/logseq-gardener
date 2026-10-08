## Context

The SDK's `site::publish_site` already publishes local HTML with selected-title links and requested assets. The CLI currently provides help and version reporting.

## Goals / Non-Goals

Goals: expose the SDK through `lsg publish`, retain offline help discovery, and report publication outcomes clearly.

Non-Goals: complete graph resolution, precise private-block visibility, Markdown export, and public hosting.

## Decisions

Use `lsg publish --graph PATH --output PATH --filename-format legacy|triple-lowbar` with repeated `--include` and `--exclude`. The [project proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md) already reserves `--graph` for garden selection. `--output` makes the destination clear. `PathBuf` arguments keep filesystem paths in their native operating-system representation. Namespace strings remain UTF-8. Selection stays case-sensitive, exclusions win, and an empty include list selects nothing.

Call `NamespaceSelection::new` and `site::publish_site` directly. The SDK owns visibility, rendering, assets, collisions, and destination protection. Help uses the existing section engine at the root and publish paths; help never needs publishing arguments or filesystem access.

Return a structured CLI response containing stdout and stderr. Human and JSON success reports include page/asset counts, withheld pages, skipped journals, and diagnostic count; JSON also carries format_version, command_path, output_directory and index_path. Unsupported constructs emit local stderr diagnostics and exit zero. Invalid options and selectors exit two; SDK generation failures exit one with empty stdout. Reuse the output writer and escape controls in human reports and diagnostics. JSON serialization uses valid JSON escapes for controls, including Unicode C1 characters. Closed stdout remains success; a closed diagnostics pipe must not suppress the success report.

## Risks / Trade-offs

Filesystem write failure can leave partial new output → explain recovery with a fresh destination. SDK limitations may surprise readers → document withheld whole pages, skipped journals, unresolved aliases, and fallback references. Native paths can be lossy in JSON display → filesystem operations retain native arguments, while JSON paths use a display string.

## Migration Plan

Add the command alongside help and version and update focused instructions. Existing SDK callers continue unchanged.

## Resolved Questions

### 1 - What command describes this HTML workflow?

`lsg publish` follows the project's current publishing brief. `lsg export` remains the older Markdown-export proposal.

### 2 - What distinguishes unsupported content from failure?

A completed SDK report exits zero with diagnostic count and stderr details. A failed SDK call exits one with empty stdout.

## Open Questions

None.
