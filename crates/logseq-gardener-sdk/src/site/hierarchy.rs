//! Navigation-only namespace destinations derived from visible source titles.

use std::collections::{BTreeMap, BTreeSet};

use crate::page_names::lookup_key;

use super::{SiteError, digest, render::escape};

pub(super) struct Node {
    pub title: String,
    pub route: String,
}

pub(super) struct Hierarchy {
    nodes: BTreeMap<String, Node>,
    children: BTreeMap<String, Vec<String>>,
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
            nodes.insert(key, Node { title, route });
        }
        let mut children: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (key, node) in &nodes {
            if let Some(parent) = ancestors(&node.title)
                .into_iter()
                .rev()
                .map(|prefix| lookup_key(&prefix))
                .find(|prefix| nodes.contains_key(prefix))
            {
                children.entry(parent).or_default().push(key.clone());
            }
        }
        for entries in children.values_mut() {
            entries.sort_by(|left, right| nodes[left].title.cmp(&nodes[right].title));
        }
        Ok(Self { nodes, children })
    }

    pub fn entries(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn breadcrumbs(&self, title: &str) -> String {
        let mut html =
            String::from("<nav class=\"namespace-breadcrumbs\" aria-label=\"Breadcrumb\"><ol>");
        let parts = title.split('/').collect::<Vec<_>>();
        for (position, label) in parts.iter().enumerate() {
            let label = escape(label);
            if position + 1 == parts.len() {
                html.push_str(&format!(
                    "<li><span aria-current=\"page\">{label}</span></li>"
                ));
            } else if let Some(node) = self.nodes.get(&lookup_key(&parts[..=position].join("/"))) {
                let label = escape(node.title.rsplit('/').next().unwrap_or(&node.title));
                html.push_str(&format!(
                    "<li><a href=\"{}\">{label}</a></li>",
                    escape(&node.route)
                ));
            } else {
                html.push_str(&format!("<li><span>{label}</span></li>"));
            }
        }
        html.push_str("</ol></nav>");
        html
    }

    pub fn child_links(&self, title: &str) -> String {
        let key = lookup_key(title);
        let Some(children) = self.children.get(&key) else {
            return String::new();
        };
        let depth = title.split('/').count();
        let mut html = String::from(
            "<section class=\"namespace-hierarchy\" aria-label=\"Hierarchy\"><h2>Hierarchy</h2><ul>",
        );
        for child in children {
            let node = &self.nodes[child];
            let label = node
                .title
                .split('/')
                .skip(depth)
                .collect::<Vec<_>>()
                .join("/");
            html.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>",
                escape(&node.route),
                escape(&label)
            ));
        }
        html.push_str("</ul></section>");
        html
    }
}
