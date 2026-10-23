//! Presentation of destinations derived from visible namespace prefixes.

use super::render::escape;

pub(super) const STYLES: &str = ".namespace-link{text-decoration-line:underline;text-decoration-style:dotted;text-underline-offset:.2em}.page-kind{font-size:.85em;border:1px solid currentColor;border-radius:.2em;padding:0 .25em;margin-inline-start:.4em}.page-sequence{display:flex;flex-wrap:wrap;gap:.75rem 1.5rem;margin-block:1.5rem}.page-sequence a{overflow-wrap:anywhere}";

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

pub(super) fn badge(generated: bool) -> &'static str {
    if generated {
        " <span class=\"page-kind\">Namespace</span>"
    } else {
        ""
    }
}

pub(super) fn notice() -> &'static str {
    "<p class=\"namespace-notice\">Namespace — browse the pages below.</p>"
}
