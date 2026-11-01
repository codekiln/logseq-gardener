//! Derived rows from build plugins, rendered as outline bullets.
//!
//! The rows were checked and their links resolved by [`crate::contributions::resolve`]; this
//! module only writes markup. Rows carry no block anchor and are added after every index is
//! built, so they reach no reference, search record, graph edge or embed.
use std::collections::BTreeMap;

use crate::contributions::{PlacedRows, ResolvedNode, ResolvedRow};

use super::render::escape;

/// The attribute every derived row list carries, and that marks a page needing [`STYLES`].
pub(super) const MARKER: &str = "class=\"lsg-derived";

/// Bullet styles for derived rows, added only to pages that show them, so other pages keep
/// their bytes. The values copy the outline's own bullet.
pub(super) const STYLES: &str = "<style data-lsg-derived>.lsg-derived .lsg-dot{position:absolute;left:0;top:var(--lsg-bullet-top);width:14px;height:14px;border-radius:50%;display:flex;align-items:center;justify-content:center}.lsg-derived .lsg-dot::after{content:\"\";width:6px;height:6px;border-radius:50%;background:var(--lsg-surface);opacity:.8}</style>";

/// A page's derived rows: the markup after each target block's children, keyed by the block's
/// bullet offset, and the markup at the end of the outline.
#[derive(Default)]
pub(super) struct PageRows {
    pub blocks: BTreeMap<usize, String>,
    pub end: String,
}

pub(super) fn page_rows(placed: &[PlacedRows]) -> PageRows {
    let mut rows = PageRows::default();
    for placement in placed {
        let list = list(
            &placement.plugin,
            &placement.rows,
            placement.block.is_none(),
        );
        match placement.block {
            Some(start) => rows.blocks.entry(start).or_default().push_str(&list),
            None => rows.end.push_str(&list),
        }
    }
    rows
}

fn list(plugin: &str, rows: &[ResolvedRow], top: bool) -> String {
    let mut html = format!(
        "<ul class=\"{}lsg-derived\" data-lsg-plugin=\"{}\">",
        if top { "outline " } else { "" },
        escape(plugin)
    );
    items(rows, &mut html);
    html.push_str("</ul>");
    html
}

fn items(rows: &[ResolvedRow], html: &mut String) {
    for row in rows {
        html.push_str("<li><span class=\"lsg-dot\"></span>");
        for node in &row.content {
            match node {
                ResolvedNode::Text(text) => html.push_str(&escape(text)),
                ResolvedNode::Page { href, label } => {
                    // A link to a whole page is a page link, as in Logseq; one to a block is not.
                    let class = if href.contains('#') {
                        ""
                    } else {
                        " class=\"page-ref\""
                    };
                    html.push_str(&format!(
                        "<a{class} href=\"{}\">{}</a>",
                        escape(href),
                        escape(label)
                    ));
                }
                ResolvedNode::External { href, label } => {
                    let new_tab = if href.to_ascii_lowercase().starts_with("mailto:") {
                        ""
                    } else {
                        " class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\""
                    };
                    html.push_str(&format!(
                        "<a{new_tab} href=\"{}\">{}</a>",
                        escape(href),
                        escape(label)
                    ));
                }
            }
        }
        if !row.children.is_empty() {
            html.push_str("<ul>");
            items(&row.children, html);
            html.push_str("</ul>");
        }
        html.push_str("</li>");
    }
}
