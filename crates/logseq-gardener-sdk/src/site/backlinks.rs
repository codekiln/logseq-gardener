//! Authored page references from selected source, never from generated navigation.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::config::ReferenceConfig;
use super::render::{self, BlockTarget};
use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::page_names::{PageNameIndex, lookup_key};

struct Context<'a> {
    source: String,
    document: &'a GardenDocument,
    start: usize,
    subtree_end: usize,
    level: u32,
    anchor: Option<usize>,
    properties: bool,
    direct: BTreeSet<String>,
    path: BTreeSet<String>,
    related: BTreeSet<String>,
    ancestors: Vec<(String, Option<usize>)>,
}

pub(super) struct Backlinks<'a> {
    contexts: Vec<Context<'a>>,
    /// The contiguous range of `contexts` each source page produced.
    by_source: BTreeMap<String, std::ops::Range<usize>>,
    incoming: BTreeMap<String, Vec<usize>>,
    titles: BTreeMap<String, String>,
    property_targets: Option<BTreeMap<String, BTreeSet<String>>>,
    property_rows: BTreeMap<String, BTreeMap<String, String>>,
}

impl<'a> Backlinks<'a> {
    pub fn new(
        selected: &[(String, String, &'a GardenDocument)],
        names: &PageNameIndex<String>,
        property_views: Option<&BTreeMap<String, super::properties::PropertyView>>,
        destinations: BTreeMap<String, String>,
    ) -> Self {
        let by_name: BTreeMap<_, _> = destinations
            .iter()
            .map(|(route, title)| (lookup_key(title), route.clone()))
            .collect();
        let resolve = |name: &str| match names.candidates(name) {
            [route] => Some(route.clone()),
            [] => by_name.get(&lookup_key(name)).cloned(),
            _ => None,
        };
        let mut result = Self {
            contexts: Vec::new(),
            by_source: BTreeMap::new(),
            incoming: BTreeMap::new(),
            titles: destinations,
            property_rows: property_views
                .map(|views| {
                    views
                        .iter()
                        .map(|(route, view)| {
                            let mut rows = BTreeMap::<String, String>::new();
                            for row in &view.rows {
                                for target in &row.targets {
                                    rows.entry(target.clone()).or_default().push_str(&row.html);
                                }
                            }
                            (route.clone(), rows)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            property_targets: property_views.map(|views| {
                views
                    .iter()
                    .map(|(route, view)| (route.clone(), view.targets.clone()))
                    .collect()
            }),
        };
        for (_, route, document) in selected {
            let blocks = &document.parsed.blocks;
            let bullets: Vec<_> = blocks
                .iter()
                .enumerate()
                .filter_map(|(index, block)| {
                    if let Block::Bullet {
                        level,
                        span: Some(span),
                        ..
                    } = block
                    {
                        Some((index, *level, span.0))
                    } else {
                        None
                    }
                })
                .collect();
            let mut subtree_ends = vec![blocks.len(); bullets.len()];
            let mut outline_stack: Vec<usize> = Vec::new();
            for (position, &(index, level, _)) in bullets.iter().enumerate() {
                while outline_stack
                    .last()
                    .is_some_and(|&previous| bullets[previous].1 >= level)
                {
                    subtree_ends[outline_stack.pop().unwrap()] = index;
                }
                outline_stack.push(position);
            }
            let mut units = Vec::new();
            if let Some(&(first, _, byte)) = bullets.first() {
                if first > 0 {
                    units.push((
                        0,
                        first,
                        first,
                        0,
                        0,
                        byte,
                        blocks[..first]
                            .iter()
                            .all(|b| matches!(b, Block::Properties { .. })),
                    ));
                }
                for (position, &(start, level, byte)) in bullets.iter().enumerate() {
                    let (end, end_byte) = bullets
                        .get(position + 1)
                        .map(|(i, _, b)| (*i, *b))
                        .unwrap_or((blocks.len(), document.source.len()));
                    let subtree_end = subtree_ends[position];
                    units.push((start, end, subtree_end, level, byte, end_byte, false));
                }
            } else if !blocks.is_empty() {
                let properties = blocks.iter().all(|b| matches!(b, Block::Properties { .. }));
                units.push((
                    0,
                    blocks.len(),
                    blocks.len(),
                    1,
                    0,
                    document.source.len(),
                    properties,
                ));
            }
            let mut parents: Vec<(u32, usize)> = Vec::new();
            let first_context = result.contexts.len();
            for (start, end, subtree_end, level, byte, end_byte, properties) in units {
                let source = document.source.get(byte..end_byte).unwrap_or_default();
                let mut refs = lsdoc::refs(source, "markdown").page;
                // Logseq's tags property also accepts comma-separated bare page names.
                for block in &blocks[start..end] {
                    if let Block::Properties { props, .. } = block {
                        for prop in props.iter().filter(|p| p.0.eq_ignore_ascii_case("tags")) {
                            if !prop.1.contains("[[") && !prop.1.starts_with('"') {
                                refs.extend(
                                    prop.1
                                        .split(',')
                                        .map(|s| s.trim().trim_start_matches('#').to_owned())
                                        .filter(|s| !s.is_empty()),
                                );
                            }
                        }
                    }
                }
                let direct: BTreeSet<_> = if properties {
                    result
                        .property_targets
                        .as_ref()
                        .map(|targets| targets.get(route).cloned().unwrap_or_default())
                        .unwrap_or_else(|| refs.iter().filter_map(|name| resolve(name)).collect())
                } else {
                    refs.iter().filter_map(|name| resolve(name)).collect()
                };
                while parents
                    .last()
                    .is_some_and(|(previous, _)| *previous >= level)
                {
                    parents.pop();
                }
                let mut path = direct.clone();
                path.insert(route.clone());
                let mut ancestors = Vec::new();
                for (_, index) in &parents {
                    path.extend(result.contexts[*index].direct.iter().cloned());
                    let parent = &result.contexts[*index];
                    if let Some(Block::Bullet { inline, .. }) = blocks.get(parent.start) {
                        ancestors.push((render::block_label(inline, 0), parent.anchor));
                    }
                }
                let index = result.contexts.len();
                for target in &direct {
                    if target != route {
                        result
                            .incoming
                            .entry(target.clone())
                            .or_default()
                            .push(index);
                    }
                }
                let anchor = (!properties)
                    .then_some(byte)
                    .filter(|_| matches!(blocks.get(start), Some(Block::Bullet { .. })));
                result.contexts.push(Context {
                    source: route.clone(),
                    document,
                    start,
                    subtree_end,
                    level,
                    anchor,
                    properties,
                    direct,
                    related: path.clone(),
                    path,
                    ancestors,
                });
                if !properties {
                    parents.push((level, index));
                }
            }
            // Aggregate descendants once, rather than rescanning every source context.
            let mut descendants: Vec<usize> = Vec::new();
            for index in (first_context..result.contexts.len()).rev() {
                while descendants.last().is_some_and(|&child| {
                    result.contexts[child].start < result.contexts[index].subtree_end
                }) {
                    let child = descendants.pop().unwrap();
                    let related = result.contexts[child].related.clone();
                    result.contexts[index].related.extend(related);
                }
                descendants.push(index);
            }
            result
                .by_source
                .insert(route.clone(), first_context..result.contexts.len());
        }
        result
    }

    pub fn source_anchors(&self, route: &str) -> BTreeSet<usize> {
        self.contexts[self.by_source.get(route).cloned().unwrap_or_default()]
            .iter()
            .filter(|context| context.direct.iter().any(|target| target != route))
            .flat_map(|context| {
                context
                    .anchor
                    .into_iter()
                    .chain(context.ancestors.iter().filter_map(|(_, anchor)| *anchor))
            })
            .collect()
    }

    /// Number of distinct pages that refer to `target`.
    pub fn page_count(&self, target: &str) -> usize {
        self.incoming
            .get(target)
            .into_iter()
            .flatten()
            .map(|&index| self.contexts[index].source.as_str())
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub fn render(
        &self,
        target: &str,
        config: &ReferenceConfig,
        names: &PageNameIndex<String>,
        targets: &BTreeMap<String, Option<BlockTarget>>,
        documents: &BTreeMap<String, &GardenDocument>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> (String, Vec<String>) {
        let indices = self.incoming.get(target).map_or(&[][..], Vec::as_slice);
        let mut groups: BTreeMap<&str, Vec<&Context<'_>>> = BTreeMap::new();
        let mut options = BTreeSet::new();
        for &index in indices {
            let context = &self.contexts[index];
            groups.entry(&context.source).or_default().push(context);
            options.extend(
                context
                    .related
                    .iter()
                    .filter(|route| route.as_str() != target)
                    .cloned(),
            );
        }
        let mut groups = groups.into_iter().collect::<Vec<_>>();
        groups.sort_by_key(|(route, _)| self.titles.get(*route).cloned().unwrap_or_default());
        let open = if (indices.len() as i128) < i128::from(config.threshold) {
            " open"
        } else {
            ""
        };
        let context_hidden = if groups.len() == indices.len() {
            " hidden"
        } else {
            ""
        };
        let page_label = if groups.len() == 1 { "page" } else { "pages" };
        let mut html = format!(
            "<section class=\"linked-references\" id=\"linked-references\" data-lsg-backlinks aria-label=\"Linked references\"><details class=\"backlinks-disclosure\"{open}><summary>Linked references — <span data-backlinks-page-count>{} {page_label}</span><span data-backlinks-reference-count{context_hidden}> · {} referring contexts</span></summary><p data-backlinks-count-explanation{context_hidden}>A context is a referring block or a page’s properties.</p>",
            groups.len(),
            indices.len()
        );
        html.push_str("<div class=\"backlinks-controls\" data-backlinks-controls hidden><fieldset><legend>Related pages</legend><p>Include or exclude selected related pages.</p><label>Find related pages <input type=\"search\" data-backlinks-search></label><div class=\"backlinks-options\">");
        let mut options = options.into_iter().collect::<Vec<_>>();
        options.sort_by_key(|route| self.titles.get(route).cloned().unwrap_or_default());
        for route in options {
            let Some(title) = self.titles.get(&route) else {
                continue;
            };
            let route = render::escape(&route);
            let title = render::escape(title);
            html.push_str(&format!("<label data-backlinks-option data-backlinks-label=\"{title}\">{title} <select data-backlinks-filter=\"{route}\" aria-label=\"Filter by {title}\"><option value=\"\">Any</option><option value=\"include\">Include</option><option value=\"exclude\">Exclude</option></select></label>"));
        }
        html.push_str("</div><button type=\"button\" data-backlinks-reset>Reset filters</button><p role=\"status\" aria-live=\"polite\" data-backlinks-status></p></fieldset></div>");
        let mut diagnostics = Vec::new();
        if groups.is_empty() {
            html.push_str("<p>No linked references.</p>");
        }
        for (source, contexts) in groups {
            let title = render::escape(self.titles.get(source).map_or(source, String::as_str));
            html.push_str(&format!("<section class=\"backlinks-group\" data-backlinks-group=\"{}\"><h3><a href=\"{}\">{title}</a></h3>", render::escape(source), render::escape(source)));
            let mut rendered_end = 0;
            for context in &contexts {
                if !context.properties && context.start < rendered_end {
                    continue;
                }
                if !context.properties {
                    rendered_end = context.subtree_end;
                }
                let tokens =
                    render::escape(&context.path.iter().cloned().collect::<Vec<_>>().join(" "));
                html.push_str(&format!(
                    "<article data-backlinks-entry data-backlinks-refs=\"{tokens}\">"
                ));
                let visible_properties = context.properties
                    && self
                        .property_targets
                        .as_ref()
                        .and_then(|targets| targets.get(source))
                        .is_some_and(|targets| targets.contains(target));
                let href = match context.anchor {
                    Some(start) => format!("{source}#source-block-{start}"),
                    None if visible_properties => format!("{source}#page-properties"),
                    None => source.to_owned(),
                };
                if context.properties {
                    html.push_str(&format!(
                        "<p class=\"backlink-context\"><a href=\"{}\">{}</a></p>",
                        render::escape(&href),
                        if visible_properties {
                            "Page properties"
                        } else {
                            "Page reference"
                        }
                    ));
                    if let Some(rows) = self
                        .property_rows
                        .get(source)
                        .and_then(|rows| rows.get(target))
                    {
                        html.push_str(&format!("<dl class=\"backlink-properties\">{rows}</dl>"));
                    }
                } else {
                    html.push_str("<p class=\"backlink-context\">");
                    for (label, anchor) in &context.ancestors {
                        let ancestor_href = anchor.map_or_else(
                            || source.to_owned(),
                            |start| format!("{source}#source-block-{start}"),
                        );
                        html.push_str(&format!(
                            "<a href=\"{}\">{}</a><span aria-hidden=\"true\"> › </span>",
                            render::escape(&ancestor_href),
                            render::escape(label)
                        ));
                    }
                    html.push_str(&format!("<a class=\"backlink-source\" href=\"{}\" aria-label=\"Open referring block in {title}\" title=\"Open referring block\">↗</a></p>", render::escape(&href)));
                    let mut copy = GardenDocument {
                        relative_path: context.document.relative_path.clone(),
                        kind: context.document.kind,
                        source: String::new(),
                        parsed: lsdoc::ast::Projection {
                            blocks: context.document.parsed.blocks
                                [context.start..context.subtree_end]
                                .to_vec(),
                            refs: lsdoc::ast::Refs {
                                page: Vec::new(),
                                block: Vec::new(),
                            },
                        },
                    };
                    for block in &mut copy.parsed.blocks {
                        if let Block::Bullet { level, .. } = block {
                            *level = level.saturating_sub(context.level) + 1;
                        }
                    }
                    let rendered = render::render_reference_context(
                        &copy,
                        names,
                        targets,
                        source,
                        documents,
                        render::ReferencePresentation {
                            open_depth: config.open_depth,
                            entries: contexts
                                .iter()
                                .filter(|child| {
                                    !child.properties
                                        && child.start > context.start
                                        && child.start < context.subtree_end
                                })
                                .filter_map(|child| {
                                    child.anchor.map(|start| {
                                        (start, format!("{source}#source-block-{start}"))
                                    })
                                })
                                .collect(),
                            paths: self.contexts
                                [self.by_source.get(source).cloned().unwrap_or_default()]
                            .iter()
                            .filter(|child| {
                                child.start >= context.start && child.start < context.subtree_end
                            })
                            .filter_map(|child| {
                                child.anchor.map(|start| {
                                    (
                                        start,
                                        child.path.iter().cloned().collect::<Vec<_>>().join(" "),
                                    )
                                })
                            })
                            .collect(),
                        },
                        asset,
                    );
                    html.push_str(&rendered.html);
                    diagnostics.extend(rendered.diagnostics);
                }
                html.push_str("</article>");
            }
            html.push_str("</section>");
        }
        html.push_str("</details></section>");
        (html, diagnostics)
    }
}
