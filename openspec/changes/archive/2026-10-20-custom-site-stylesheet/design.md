## Context

`publish_site` plans selected pages and assets before creating a fresh output directory. `html_document` supplies built-in CSS. All generated HTML files share the output root, so one relative stylesheet link serves the index and document pages.

## Goals / Non-Goals

**Goals:** Let SDK and CLI callers customize appearance with a local stylesheet, preserve default publication, and reject invalid stylesheet input before creating output.

**Non-Goals:** HTML template customization and automatic copying of stylesheet dependencies.

## Decisions

Add a defaultable `SiteOptions` containing `stylesheet: Option<PathBuf>` and a `publish_site_with_options` entry point. Keep `publish_site` as a wrapper with default options, preserving existing callers and generated output. A new API accommodates future optional settings without changing the existing signature.

Validate the input with `symlink_metadata`, require a regular file with a case-insensitive `.css` extension, reject symlinks in every path component, and read its contents during planning. Copy those contents unchanged to `custom.css` in the output root. A fixed filename avoids source-path disclosure and HTML attribute injection. The stylesheet link still uses the renderer's HTML escaping. Link it after the built-in style so ordinary cascade rules allow overrides. `SiteReport.assets` continues to count graph assets only.

Expose the same choice as `lsg publish --stylesheet PATH`, using native path handling. Fatal stylesheet errors follow existing publication failure reporting: exit one, empty stdout, useful stderr. Help and the local-site guide explain that CSS resource URLs resolve relative to the copied file. Owners choose and trust the CSS; the publisher copies only the specified file and the browser handles resources referenced by it.

## Risks / Trade-offs

- CSS can reference other resources → document browser resolution and keep copying explicit.
- Filesystem changes during generation can affect validation → retain the existing requirement that source paths remain stable during generation; keep stylesheet contents in memory before writing.
- A new options entry point adds public API surface → retain one implementation shared with the existing function.

## Migration Plan

Existing callers continue unchanged. Callers wanting custom styles use the new entry point or CLI flag. Omitting the option restores built-in presentation.

## Resolved Questions

### 1 - Where does the copied stylesheet live?

At `custom.css` beside every generated HTML document, linked after the built-in style.

### 2 - Are stylesheets included in the asset count?

The existing count describes referenced garden assets. A custom stylesheet is a publication option and does not change that count.

## Open Questions

None.
