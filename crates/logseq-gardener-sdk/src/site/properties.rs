//! Opt-in leading property display from selected source identities.
use std::collections::{BTreeMap, BTreeSet};

use super::{config::ReferenceConfig, render::escape, sequence};
use crate::{ast::Block, garden::GardenDocument, page_names::PageNameIndex};

pub(super) struct PropertyRow {
    pub html: String,
    pub targets: BTreeSet<String>,
}

#[derive(Default)]
pub(super) struct PropertyView {
    pub rows: Vec<PropertyRow>,
    pub html: String,
    pub targets: BTreeSet<String>,
    pub sequence_keys: BTreeSet<String>,
}

fn internal(key: &str) -> bool {
    matches!(
        key,
        "id" | "custom-id"
            | "custom_id"
            | "title"
            | "alias"
            | "aliases"
            | "public"
            | "background-color"
            | "background_color"
            | "heading"
            | "collapsed"
            | "created-at"
            | "updated-at"
            | "last-modified-at"
            | "created_at"
            | "updated_at"
            | "last_modified_at"
            | "query-table"
            | "query-properties"
            | "query-sort-by"
            | "query-sort-desc"
            | "filters"
            | "ls-type"
            | "hl-type"
            | "hl-page"
            | "hl-stamp"
            | "hl-color"
            | "todo"
            | "doing"
            | "now"
            | "later"
            | "done"
            | "file"
            | "file-path"
            | "file-name"
    ) || key.starts_with("logseq-proxy-")
        || key.starts_with("logseq.")
        || key.starts_with("publishing/")
}

fn value(
    text: &str,
    names: &PageNameIndex<String>,
    routes: &BTreeMap<String, String>,
) -> Option<(String, BTreeSet<String>)> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("https://") {
        let rest = &text[8..];
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        if authority.is_empty()
            || authority.contains('@')
            || text.chars().any(|c| {
                c.is_whitespace()
                    || c.is_control()
                    || matches!(c, '\\' | '"' | '\'' | '<' | '>' | '[' | ']' | '{' | '}')
            })
        {
            return None;
        }
        let url = escape(text);
        return Some((format!("<a href=\"{url}\">{url}</a>"), BTreeSet::new()));
    }
    if text.contains('\\')
        || text
            .split_whitespace()
            .any(|word| word.starts_with('/') || word.starts_with("~/"))
        || ["javascript:", "data:", "vbscript:", "http:"]
            .iter()
            .any(|scheme| lower.contains(scheme))
    {
        return None;
    }
    if [
        "{{", "}}", "((", "))", "![", "](", "file:", "assets/", "../", "logseq:",
    ]
    .iter()
    .any(|token| lower.contains(token))
        || text.chars().any(char::is_control)
    {
        return None;
    }
    let mut html = String::new();
    let mut targets = BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let prefix = &rest[..start];
        if prefix.contains("]]") {
            return None;
        }
        html.push_str(&escape(prefix));
        rest = &rest[start + 2..];
        let end = rest.find("]]")?;
        let name = &rest[..end];
        if name.contains(['[', ']']) {
            return None;
        }
        let [route] = names.candidates(name) else {
            return None;
        };
        let title = routes.get(route)?;
        html.push_str(&format!(
            "<a href=\"{}\">{}</a>",
            escape(route),
            escape(title)
        ));
        targets.insert(route.clone());
        rest = &rest[end + 2..];
    }
    if rest.contains("]]") {
        return None;
    }
    html.push_str(&escape(rest));
    Some((html, targets))
}

pub(super) fn view(
    document: &GardenDocument,
    current_route: &str,
    names: &PageNameIndex<String>,
    routes: &BTreeMap<String, String>,
    config: &ReferenceConfig,
) -> PropertyView {
    let mut result = PropertyView::default();
    if !config.properties {
        return result;
    }
    let Some(Block::Properties { props, .. }) = document.parsed.blocks.first() else {
        return result;
    };
    let mut rows = String::new();
    for (index, property) in props.iter().enumerate() {
        let (key, text) = (&property.0, &property.1);
        let normalized = key.to_lowercase();
        if internal(&normalized)
            || config.hidden_properties.contains(&normalized)
            || props[index + 1..]
                .iter()
                .any(|other| other.0.eq_ignore_ascii_case(key))
            || text.trim().is_empty()
        {
            continue;
        }
        if matches!(normalized.as_str(), "prev" | "next")
            && sequence::resolve(document, &normalized, current_route, names, routes).is_none()
        {
            continue;
        }
        let tagged;
        let text = if normalized == "tags" && !text.contains("[[") && !text.starts_with('"') {
            tagged = text
                .split(',')
                .map(|tag| format!("[[{}]]", tag.trim().trim_start_matches('#')))
                .collect::<Vec<_>>()
                .join(", ");
            &tagged
        } else {
            text
        };
        let Some((html, targets)) = value(text, names, routes) else {
            continue;
        };
        let row = format!(
            "<div class=\"page-property\"><dt>{}</dt><dd>{html}</dd></div>",
            escape(key)
        );
        rows.push_str(&row);
        result.rows.push(PropertyRow {
            html: row,
            targets: targets.clone(),
        });
        result.targets.extend(targets);
        if matches!(normalized.as_str(), "prev" | "next") {
            result.sequence_keys.insert(normalized);
        }
    }
    if !rows.is_empty() {
        let open = if config.properties_collapsed {
            ""
        } else {
            " open"
        };
        result.html = format!(
            "<section class=\"page-properties\" aria-label=\"Page properties\"><details{open}><summary id=\"page-properties\" tabindex=\"0\">Properties</summary><dl>{rows}</dl></details></section>"
        );
    }
    result
}
