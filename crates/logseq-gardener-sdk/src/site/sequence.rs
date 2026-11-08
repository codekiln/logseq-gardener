//! The Page sequence region at the bottom of a page, drawn from the neighbours that
//! [`crate::page_sequence`] resolves.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    garden::GardenDocument,
    page_names::PageNameIndex,
    page_sequence::{Direction, neighbour},
};

use super::render::escape;

pub(super) fn navigation(
    document: &GardenDocument,
    current_route: &str,
    names: &PageNameIndex<String>,
    routes: &BTreeMap<String, String>,
    omitted: &BTreeSet<String>,
) -> String {
    let mut links = String::new();
    for direction in Direction::ALL {
        let key = direction.key();
        if omitted.contains(key) {
            continue;
        }
        let Some(found) = neighbour(document, direction, current_route, names, routes) else {
            continue;
        };
        let label = match direction {
            Direction::Previous => "Previous",
            Direction::Next => "Next",
        };
        links.push_str(&format!(
            "<a rel=\"{key}\" href=\"{}\">{label}: {}</a>",
            escape(found.route),
            escape(found.title)
        ));
    }
    if links.is_empty() {
        String::new()
    } else {
        format!("<nav class=\"page-sequence\" aria-label=\"Page sequence\">{links}</nav>")
    }
}
