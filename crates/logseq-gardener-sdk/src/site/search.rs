//! Search projection over selected originals, independently of generated HTML.
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::render::{BlockTarget, outline_targets};
use crate::ast::{Block, Inline, Url};
use crate::garden::GardenDocument;
use crate::page_names::{PageNameIndex, lookup_key, page_alias_spellings};

/// Opt-in source-search settings. Publication selection and privacy still apply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// Canonical page titles whose bodies are omitted while titles remain searchable.
    pub excluded_page_bodies: BTreeSet<String>,
}

#[derive(Serialize)]
pub(super) struct Page {
    id: String,
    title: String,
    href: String,
    aliases: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct Content {
    id: String,
    page: String,
    href: String,
    text: String,
    ancestors: Vec<String>,
    position: usize,
}

pub(super) struct Search {
    pages: Vec<Page>,
    content: Vec<Content>,
    pub anchors: BTreeMap<String, BTreeSet<usize>>,
    admitted: BTreeMap<String, BTreeSet<usize>>,
}

fn compact(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Omitted nodes introduce a word boundary, so adjacent fragments cannot invent terms.
fn plain(nodes: &[Inline], names: &PageNameIndex<String>, depth: usize) -> String {
    if depth > 64 {
        return String::new();
    }
    let mut text = String::new();
    for node in nodes {
        match node {
            Inline::Plain { text: value, .. }
            | Inline::Code { text: value, .. }
            | Inline::Verbatim { text: value, .. } => text.push_str(value),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. } => {
                text.push_str(&plain(children, names, depth + 1))
            }
            Inline::Link {
                url: Url::PageRef { v },
                label,
                image: false,
                ..
            } if names.candidates(v).len() == 1 => {
                if label.is_empty() {
                    text.push_str(v);
                } else {
                    text.push_str(&plain(label, names, depth + 1));
                }
            }
            // Tags are references too: an unavailable name must never become a term.
            Inline::Tag { children, .. } => {
                let value = plain(children, names, depth + 1);
                if names.candidates(&value).len() == 1 {
                    text.push_str(&value);
                } else {
                    text.push(' ');
                }
            }
            // Explicit external-link display text is authored prose; URLs and asset
            // labels are omitted. Block refs contribute no copied target label.
            Inline::Link {
                url:
                    Url::Complex {
                        protocol: Some(protocol),
                        ..
                    },
                label,
                image: false,
                ..
            } if matches!(protocol.to_ascii_lowercase().as_str(), "http" | "https") => {
                text.push_str(&plain(label, names, depth + 1));
            }
            _ => text.push(' '),
        }
    }
    text
}

impl Search {
    pub fn new(
        selected: &[(String, String, &GardenDocument)],
        names: &PageNameIndex<String>,
        targets: &BTreeMap<String, Option<BlockTarget>>,
        options: &SearchOptions,
    ) -> Self {
        let mut search = Self {
            pages: Vec::new(),
            content: Vec::new(),
            anchors: BTreeMap::new(),
            admitted: BTreeMap::new(),
        };
        let excluded: BTreeSet<_> = options
            .excluded_page_bodies
            .iter()
            .map(|s| lookup_key(s))
            .collect();
        for (title, route, document) in selected {
            search.pages.push(Page {
                id: route.clone(),
                title: title.clone(),
                href: route.clone(),
                aliases: page_alias_spellings(document, title)
                    .into_iter()
                    .filter(|alias| names.candidates(alias) == [route.clone()])
                    .collect(),
            });
            if excluded.contains(&lookup_key(title)) {
                continue;
            }
            let uuid_by_start: BTreeMap<_, _> = outline_targets(&document.parsed.blocks)
                .into_iter()
                .filter(|(id, start, _)| {
                    targets
                        .get(id)
                        .and_then(Option::as_ref)
                        .is_some_and(|t| &t.route == route && t.start == *start)
                })
                .map(|(id, start, _)| (start, id))
                .collect();
            search.bullets(
                &document.parsed.blocks,
                route,
                names,
                &uuid_by_start,
                &[],
                0,
            );
        }
        search.pages.sort_by(|a, b| a.id.cmp(&b.id));
        search
            .content
            .sort_by(|a, b| (&a.page, a.position).cmp(&(&b.page, b.position)));
        search
    }

    fn bullets(
        &mut self,
        blocks: &[Block],
        route: &str,
        names: &PageNameIndex<String>,
        uuids: &BTreeMap<usize, String>,
        inherited: &[String],
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        let mut parents: Vec<(u32, String)> = Vec::new();
        let mut excluded_level = None;
        for (index, block) in blocks.iter().enumerate() {
            match block {
                Block::Bullet {
                    level,
                    inline,
                    span: Some(span),
                    ..
                } => {
                    if excluded_level.is_some_and(|excluded| *level > excluded) {
                        continue;
                    }
                    excluded_level = None;
                    while parents.last().is_some_and(|(l, _)| l >= level) {
                        parents.pop();
                    }
                    let excluded = matches!(blocks.get(index + 1), Some(Block::Properties { props, .. })
                        if props.iter().rev().find(|p| p.0.trim().eq_ignore_ascii_case("search-exclude"))
                            .is_some_and(|p| p.1.trim().eq_ignore_ascii_case("true")));
                    if excluded {
                        excluded_level = Some(*level);
                        continue;
                    }
                    let text = compact(&plain(inline, names, 0));
                    if !text.is_empty() {
                        let fragment = uuids.get(&span.0).map_or_else(
                            || format!("source-block-{}", span.0),
                            |id| format!("block-{id}"),
                        );
                        self.anchors
                            .entry(route.to_owned())
                            .or_default()
                            .insert(span.0);
                        self.content.push(Content {
                            id: format!("{route}#source-block-{}", span.0),
                            page: route.to_owned(),
                            href: format!("{route}#{fragment}"),
                            text: text.clone(),
                            ancestors: inherited
                                .iter()
                                .cloned()
                                .chain(parents.iter().map(|(_, s)| s.clone()))
                                .filter(|s| !s.is_empty())
                                .collect(),
                            position: span.0,
                        });
                    }
                    parents.push((*level, text.chars().take(160).collect()));
                }
                Block::Quote { children, .. } if excluded_level.is_none() => {
                    let ancestors: Vec<_> = inherited
                        .iter()
                        .cloned()
                        .chain(parents.iter().map(|(_, s)| s.clone()))
                        .collect();
                    self.bullets(children, route, names, uuids, &ancestors, depth + 1);
                }
                // Custom containers include queries and unsupported directives.
                _ => {}
            }
        }
    }

    pub fn admit(&mut self, route: &str, emitted: &BTreeSet<usize>) {
        self.admitted.insert(route.to_owned(), emitted.clone());
    }

    pub fn artifacts(mut self) -> BTreeMap<String, String> {
        self.content.retain(|record| {
            self.admitted
                .get(&record.page)
                .is_some_and(|anchors| anchors.contains(&record.position))
        });
        let records =
            serde_json::to_string(&(self.pages, self.content)).expect("search strings serialize");
        // Same non-security fingerprint as publication routes. Policy/version are
        // included so any projection-contract change invalidates this identity.
        let engine = include_str!("search.mjs");
        let revision = super::digest(&format!("search-v1:{records}:{engine}"));
        let engine_route = format!("lsg-search-engine-{revision}.mjs");
        let (pages, content): (serde_json::Value, serde_json::Value) =
            serde_json::from_str(&records).expect("serialized records");
        let count = pages.as_array().unwrap().len() + content.as_array().unwrap().len();
        let corpus = serde_json::json!({"format_version": 1, "revision": revision, "pages": pages, "content": content}).to_string();
        let corpus_route = format!("lsg-search-{revision}.json");
        let manifest = serde_json::json!({"format_version": 1, "revision": revision, "corpus": {"href": corpus_route, "bytes": corpus.len(), "records": count}, "engine": engine_route}).to_string();
        BTreeMap::from([
            (format!("lsg-search-manifest-{revision}.json"), manifest),
            (corpus_route, corpus),
            (engine_route, engine.to_owned()),
        ])
    }
}
