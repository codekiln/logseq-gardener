//! Navigation-only namespace destinations derived from visible source titles.

use std::collections::{BTreeMap, BTreeSet};

use crate::namespaces::{NamespaceEntry, NamespaceIndex, Row, RowFilter};
use crate::page_names::lookup_key;

#[cfg(test)]
use super::digest;
use super::{references::REFS, render::escape};

/// Rows in the page; Logseq renders all of them, but a namespace root's first paint
/// must not grow with its namespace.
const INLINE_ROWS: usize = 100;
/// Rows in each fragment file after the first [`INLINE_ROWS`].
const ROWS_PER_FILE: usize = 500;

/// A page's Hierarchy section and the fragment files that hold its later rows.
#[derive(Default)]
pub(super) struct Section {
    pub html: String,
    pub files: Vec<(String, String)>,
}

pub(super) struct Node {
    pub title: String,
    pub route: String,
    pub generated: bool,
}

pub(super) struct Hierarchy {
    nodes: BTreeMap<String, Node>,
    index: NamespaceIndex,
}

#[cfg(test)]
fn ancestors(title: &str) -> Vec<String> {
    title
        .match_indices('/')
        .map(|(end, _)| title[..end].to_owned())
        .collect()
}

impl Hierarchy {
    pub fn new(
        sources: &BTreeMap<String, String>,
        withheld: &BTreeSet<String>,
        generated: &BTreeMap<String, (String, String)>,
        alias_spellings: &BTreeMap<String, Vec<String>>,
    ) -> Self {
        // Rows and destinations come from the same titles in one pass.
        let index = NamespaceIndex::build(
            sources.iter().map(|(route, title)| {
                NamespaceEntry::new(
                    route.clone(),
                    title.clone(),
                    alias_spellings
                        .get(&lookup_key(title))
                        .cloned()
                        .unwrap_or_default(),
                )
            }),
            withheld.iter().cloned(),
        );
        let mut nodes = sources
            .iter()
            .map(|(route, title)| {
                (
                    lookup_key(title),
                    Node {
                        title: title.clone(),
                        route: route.clone(),
                        generated: false,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        for (key, (title, route)) in generated {
            nodes.insert(
                key.clone(),
                Node {
                    title: title.clone(),
                    route: route.clone(),
                    generated: true,
                },
            );
        }
        Self { nodes, index }
    }

    pub fn entries(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    fn row(&self, row: &Row, show_brackets: bool) -> String {
        let mut html = String::from("<li>");
        for (position, segment) in row.segments().iter().enumerate() {
            if position > 0 {
                html.push_str("<span class=\"namespace-separator\" aria-hidden=\"true\">/</span>");
            }
            let label = escape(&segment.label);
            match self.nodes.get(&segment.prefix) {
                Some(node) => html.push_str(&format!(
                    "<span class=\"page-reference{}\"><a{} href=\"{}\">{label}</a></span>",
                    if show_brackets { "" } else { " no-brackets" },
                    super::page_status::link_attributes(node.generated, &node.title),
                    escape(&node.route)
                )),
                None => html.push_str(&format!(
                    "<span class=\"namespace-unavailable\">{label}</span>"
                )),
            }
        }
        html.push_str("</li>");
        html
    }

    /// The Hierarchy section for `title` and its fragment files, or nothing when the
    /// page has no hierarchy. `route` names the fragments.
    pub fn bottom_view(&self, title: &str, route: &str, show_brackets: bool) -> Section {
        let Some(hierarchy) = self.index.hierarchy(title, RowFilter::StoredOnly) else {
            return Section::default();
        };
        let rows = hierarchy
            .rows
            .iter()
            .map(|row| self.row(row, show_brackets))
            .collect::<Vec<_>>();
        let inline = rows.len().min(INLINE_ROWS);
        let stem = format!("{REFS}/{}-hierarchy", crate::page_urls::stem(route));
        let name = |position: usize| format!("{stem}-{}.html", position + 1);
        let chunks = rows[inline..].chunks(ROWS_PER_FILE).collect::<Vec<_>>();
        let page_title = escape(title);
        let page_route = escape(route);
        let files = chunks
            .iter()
            .enumerate()
            .map(|(position, chunk)| {
                let next = if position + 1 < chunks.len() {
                    format!(
                        "<p><a href=\"{}\" rel=\"next\">More of the hierarchy</a></p>",
                        name(position + 1)
                    )
                } else {
                    String::new()
                };
                (
                    name(position),
                    format!(
                        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><base href=\"../\"><title>Hierarchy: {page_title}</title></head><body><main><h1>Hierarchy: <a href=\"{page_route}\">{page_title}</a></h1><div class=\"references-fragment\"><ul class=\"namespaces\">{}</ul></div>{next}</main></body></html>\n",
                        chunk.concat()
                    ),
                )
            })
            .collect::<Vec<_>>();
        let (source, fallback) = if files.is_empty() {
            (String::new(), String::new())
        } else {
            (
                format!(" data-hierarchy-src=\"{}\"", name(0)),
                format!(
                    "<noscript><a href=\"{}\">Show more of the hierarchy</a></noscript>",
                    name(0)
                ),
            )
        };
        Section {
            html: format!(
                "<section class=\"page-hierarchy references\" aria-label=\"Hierarchy\"{source}><details class=\"references-section\" open><summary><span class=\"ref-chevron\"></span><h2>Hierarchy</h2></summary><ul class=\"namespaces\">{}</ul>{fallback}</details></section>",
                rows[..inline].concat()
            ),
            files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish(titles: &[String]) -> Hierarchy {
        let sources = titles
            .iter()
            .map(|title| (format!("p-{}.html", digest(title)), title.clone()))
            .collect::<BTreeMap<String, String>>();
        let keys: BTreeSet<String> = titles.iter().map(|title| lookup_key(title)).collect();
        let mut generated = BTreeMap::new();
        for title in titles {
            for prefix in ancestors(title) {
                if !keys.contains(&lookup_key(&prefix)) {
                    generated.insert(
                        lookup_key(&prefix),
                        (prefix.clone(), format!("p-{}.html", digest(&prefix))),
                    );
                }
            }
        }
        Hierarchy::new(&sources, &BTreeSet::new(), &generated, &BTreeMap::new())
    }

    #[test]
    fn full_descendant_enumeration_has_no_namespace_depth_cutoff() {
        let title = std::iter::once("Root".to_owned())
            .chain((0..128).map(|index| format!("N{index}")))
            .collect::<Vec<_>>()
            .join("/");
        let hierarchy = publish(std::slice::from_ref(&title));
        let section = hierarchy.bottom_view("Root", "p-root.html", true);
        assert_eq!(section.html.matches("<li>").count(), 1);
        assert_eq!(hierarchy.entries().count(), 129);
        for prefix in ancestors(&title) {
            assert!(
                section
                    .html
                    .contains(&format!("href=\"p-{}.html\"", digest(&prefix)))
            );
        }
        assert!(
            section
                .html
                .contains(&format!("href=\"p-{}.html\">N127</a>", digest(&title)))
        );
        assert_eq!(section.html.matches("<ul class=\"namespaces\">").count(), 1);
        let leaf = hierarchy.bottom_view(&title, "p-leaf.html", true);
        assert_eq!(leaf.html.matches("<li>").count(), 1);
        assert!(leaf.html.contains(">N126</a>"));
        assert!(!leaf.html.contains(">N127</a>"));
    }

    #[test]
    fn long_lists_continue_in_fragment_files() {
        let titles = std::iter::once("Big".to_owned())
            .chain((0..1_150).map(|index| format!("Big/<{index:04}>")))
            .collect::<Vec<_>>();
        let hierarchy = publish(&titles);
        let section = hierarchy.bottom_view("Big", "p-big.html", false);
        assert_eq!(section.html.matches("<li>").count(), INLINE_ROWS);
        assert!(
            section
                .html
                .contains("data-hierarchy-src=\"_refs/p-big-hierarchy-1.html\"")
        );
        assert!(
            section
                .html
                .contains("<noscript><a href=\"_refs/p-big-hierarchy-1.html\">")
        );
        assert!(section.html.contains("&lt;0099&gt;"));
        assert!(!section.html.contains("&lt;0100&gt;"));
        assert!(section.html.contains("page-reference no-brackets"));
        let names = section
            .files
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "_refs/p-big-hierarchy-1.html",
                "_refs/p-big-hierarchy-2.html",
                "_refs/p-big-hierarchy-3.html"
            ]
        );
        let rows = section
            .files
            .iter()
            .map(|(_, html)| html.matches("<li>").count())
            .collect::<Vec<_>>();
        assert_eq!(rows, [500, 500, 50]);
        let first = &section.files[0].1;
        assert!(first.contains("<base href=\"../\">"));
        assert!(first.contains("&lt;0100&gt;"));
        assert!(first.contains("href=\"_refs/p-big-hierarchy-2.html\" rel=\"next\""));
        assert!(!section.files[2].1.contains("rel=\"next\""));
        assert!(!section.html.contains("<0"));
    }

    #[test]
    fn short_lists_and_plain_pages_write_no_fragments() {
        let hierarchy = publish(&["Plain".to_owned(), "A".to_owned(), "A/B".to_owned()]);
        assert!(
            hierarchy
                .bottom_view("Plain", "p-plain.html", true)
                .html
                .is_empty()
        );
        let section = hierarchy.bottom_view("A", "p-a.html", true);
        assert!(section.files.is_empty());
        assert!(!section.html.contains("data-hierarchy-src"));
        assert!(!section.html.contains("Namespace</span>"));
    }
}
