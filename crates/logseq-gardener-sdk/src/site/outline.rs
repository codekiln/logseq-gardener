//! Logseq's outline look and the reader controls for published outlines.

/// Reader collapse and block zoom, written once at the site root. `lsg-shell.js` loads
/// it after the page's `load` event, so it is outside the script before first paint.
pub(super) const SCRIPT_ROUTE: &str = "lsg-outline.js";
pub(super) const SCRIPT: &str = include_str!("outline.js");
/// Title, bullets, guide lines, headings and task markers, added to every page.
pub(super) const STYLES: &str = include_str!("outline.css");
