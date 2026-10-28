//! The click-to-load player and timestamp seeking, loaded only by pages with a video.

pub(super) const SCRIPT_ROUTE: &str = "media.js";
pub(super) const SCRIPT: &str = include_str!("media.js");
pub(super) const STYLES_ROUTE: &str = "media.css";
pub(super) const STYLES: &str = include_str!("media.css");
/// Present in a page's main content when it shows a video placeholder.
pub(super) const MARKER: &str = "class=\"lsg-video\"";

/// The head tags that load the media styles and script.
pub(super) fn head(main: &str) -> String {
    if main.contains(MARKER) {
        format!(
            "<link rel=\"stylesheet\" href=\"{STYLES_ROUTE}\"><script defer src=\"{SCRIPT_ROUTE}\"></script>"
        )
    } else {
        String::new()
    }
}
