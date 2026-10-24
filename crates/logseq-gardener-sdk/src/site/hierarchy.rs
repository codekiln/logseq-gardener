//! Navigation-only namespace destinations derived from visible source titles.

use std::collections::{BTreeMap, BTreeSet};

use crate::page_names::lookup_key;

use super::{SiteError, digest, render::escape};

pub(super) struct Node {
    pub title: String,
    pub route: String,
    pub generated: bool,
}

pub(super) struct Hierarchy {
    nodes: BTreeMap<String, Node>,
}

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
        aliases: &BTreeSet<String>,
    ) -> Result<Self, SiteError> {
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
        let mut prefixes: BTreeMap<String, String> = BTreeMap::new();
        for title in sources.values() {
            for prefix in ancestors(title) {
                let key = lookup_key(&prefix);
                if nodes.contains_key(&key) || withheld.contains(&key) || aliases.contains(&key) {
                    continue;
                }
                prefixes
                    .entry(key)
                    .and_modify(|previous| {
                        if prefix < *previous {
                            previous.clone_from(&prefix);
                        }
                    })
                    .or_insert(prefix);
            }
        }
        let mut routes = sources.clone();
        for (key, title) in prefixes {
            let route = format!("p-{}.html", digest(&title));
            if routes.insert(route.clone(), title.clone()).is_some() {
                return Err(SiteError::Collision { title });
            }
            nodes.insert(
                key,
                Node {
                    title,
                    route,
                    generated: true,
                },
            );
        }
        Ok(Self { nodes })
    }

    pub fn entries(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    fn path_links(&self, title: &str, current: bool) -> String {
        let mut html = String::from("<span class=\"namespace-path\">");
        let parts = title.split('/').collect::<Vec<_>>();
        for (position, label) in parts.iter().enumerate() {
            if position > 0 {
                html.push_str("<span class=\"namespace-separator\" aria-hidden=\"true\">/</span>");
            }
            if let Some(node) = self.nodes.get(&lookup_key(&parts[..=position].join("/"))) {
                let label = escape(node.title.rsplit('/').next().unwrap_or(&node.title));
                let marker = if current && position + 1 == parts.len() {
                    " aria-current=\"page\""
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<a{marker}{} href=\"{}\">{label}</a>",
                    super::page_status::link_attributes(node.generated, &node.title),
                    escape(&node.route)
                ));
            } else {
                html.push_str(&format!(
                    "<span class=\"namespace-unavailable\">{}</span>",
                    escape(label)
                ));
            }
        }
        if let Some(node) = self.nodes.get(&lookup_key(title)) {
            html.push_str(super::page_status::badge(node.generated));
        }
        html.push_str("</span>");
        html
    }

    pub fn bottom_view(&self, title: &str) -> String {
        let key = lookup_key(title);
        let Some(current) = self.nodes.get(&key) else {
            return String::new();
        };
        let prefix = format!("{key}/");
        let descendants = self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(name, _)| name.starts_with(&prefix))
            .map(|(_, node)| node)
            .filter(|node| !node.generated);
        let mut html = String::from(
            "<section class=\"namespace-hierarchy\" aria-label=\"Hierarchy\"><details open><summary>Hierarchy</summary><ul>",
        );
        for node in std::iter::once(current).chain(descendants) {
            html.push_str("<li>");
            html.push_str(&self.path_links(&node.title, node.route == current.route));
            html.push_str("</li>");
        }
        html.push_str("</ul></details></section>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_descendant_enumeration_has_no_namespace_depth_cutoff() {
        let title = std::iter::once("Root".to_owned())
            .chain((0..128).map(|index| format!("N{index}")))
            .collect::<Vec<_>>()
            .join("/");
        let sources = BTreeMap::from([(format!("p-{}.html", digest(&title)), title.clone())]);
        let hierarchy = Hierarchy::new(&sources, &BTreeSet::new(), &BTreeSet::new()).unwrap();
        let html = hierarchy.bottom_view("Root");
        assert_eq!(html.matches("<li>").count(), 2);
        assert_eq!(hierarchy.entries().count(), 129);
        for prefix in ancestors(&title) {
            assert!(html.contains(&format!("href=\"p-{}.html\"", digest(&prefix))));
        }
        assert!(html.contains(&format!("href=\"p-{}.html\">N127</a>", digest(&title))));
        assert_eq!(html.matches("<ul>").count(), 1);
        assert_eq!(hierarchy.bottom_view(&title).matches("<li>").count(), 1);
    }
}
