//! Presentation of destinations derived from visible namespace prefixes.

use super::render::escape;

pub(super) const STYLES: &str = ".namespace-link{text-decoration-line:underline;text-decoration-style:dotted;text-underline-offset:.2em}.page-sequence{display:flex;flex-wrap:wrap;gap:.75rem 1.5rem;margin-block:1.5rem}.page-sequence a{overflow-wrap:anywhere}";

pub(super) fn link_attributes(generated: bool, title: &str) -> String {
    if generated {
        format!(
            " class=\"namespace-link\" aria-label=\"{} (Namespace)\"",
            escape(title)
        )
    } else {
        String::new()
    }
}

pub(super) fn notice() -> &'static str {
    "<p class=\"namespace-notice\">Namespace — browse the pages below.</p>"
}
