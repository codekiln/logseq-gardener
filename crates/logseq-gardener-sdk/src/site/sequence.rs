//! Explicit sequence navigation restricted to selected visible source pages.

use std::collections::BTreeMap;

use crate::{
    ast::{Block, Inline, Url},
    garden::GardenDocument,
    page_names::PageNameIndex,
};

use super::render::escape;

fn target(document: &GardenDocument, key: &str) -> Option<String> {
    let Block::Properties { props, .. } = document.parsed.blocks.first()? else {
        return None;
    };
    let value = &props
        .iter()
        .rev()
        .find(|property| property.0.eq_ignore_ascii_case(key))?
        .1;
    let value = value.trim();
    if !value.starts_with("[[") || !value.ends_with("]]") {
        return None;
    }
    let parsed = lsdoc::inline(value, "markdown");
    match parsed.as_slice() {
        [
            Inline::Link {
                url: Url::PageRef { v },
                image: false,
                ..
            },
        ] => Some(v.clone()),
        _ => None,
    }
}

pub(super) fn resolve<'a>(
    document: &GardenDocument,
    key: &str,
    current_route: &str,
    names: &PageNameIndex<String>,
    routes: &'a BTreeMap<String, String>,
) -> Option<(String, &'a str)> {
    let target = target(document, key)?;
    let [route] = names.candidates(&target) else {
        return None;
    };
    if route == current_route {
        return None;
    }
    Some((route.clone(), routes.get(route)?.as_str()))
}

pub(super) fn navigation(
    document: &GardenDocument,
    current_route: &str,
    names: &PageNameIndex<String>,
    routes: &BTreeMap<String, String>,
    omitted: &std::collections::BTreeSet<String>,
) -> String {
    let mut links = String::new();
    for (key, label) in [("prev", "Previous"), ("next", "Next")] {
        if omitted.contains(key) {
            continue;
        }
        let Some((route, title)) = resolve(document, key, current_route, names, routes) else {
            continue;
        };
        links.push_str(&format!(
            "<a rel=\"{key}\" href=\"{}\">{label}: {}</a>",
            escape(&route),
            escape(title)
        ));
    }
    if links.is_empty() {
        String::new()
    } else {
        format!("<nav class=\"page-sequence\" aria-label=\"Page sequence\">{links}</nav>")
    }
}
