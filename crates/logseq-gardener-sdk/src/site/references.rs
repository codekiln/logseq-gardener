//! The sections below a page's outline: Pages tagged with, Linked References and
//! Unlinked References. The SDK reference index answers every question; this module
//! writes each section's header into the page and its body into fragment files that
//! `lsg-references.js` loads when a reader opens the section.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::json;

use super::config::ReferenceConfig;
use super::render;
use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::journals::JournalDate;
use crate::page_names::{PageNameIndex, lookup_key, page_alias_spellings};
use crate::references::{BlockId, Contribution, Filter, PageEntry, ReferenceIndex};
use crate::transclusion::{EmbedPage, ReferenceTargets};

/// The shared references script, written once at the site root.
pub(super) const SCRIPT_ROUTE: &str = "lsg-references.js";

/// `references.js` without comments or indentation, joining lines after `;`, `{`, `}`
/// and `,`, where no statement can end.
pub(super) fn script() -> String {
    let mut script = String::new();
    for line in include_str!("references.js").lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if script.ends_with(|c: char| !";{},".contains(c)) {
            script.push('\n');
        }
        script.push_str(line);
    }
    script.push('\n');
    script
}
/// Section headers, groups, breadcrumbs and the filter dialog, added to every page.
pub(super) const STYLES: &str = include_str!("references.css");

/// The directory that holds every page's fragment files.
pub(super) const REFS: &str = "_refs";
/// Groups per fragment file, as Logseq renders about this many before scrolling.
const GROUPS_PER_FILE: usize = 50;
/// A fragment file closes early once its groups reach this many bytes.
const FILE_BYTES: usize = 400_000;
/// Breadcrumbs show this many of the nearest ancestors.
const CRUMBS: usize = 3;

const FUNNEL: &str = "<svg width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M4 4h16v2.172a2 2 0 0 1-.586 1.414L15 12v7l-6 2v-8.5L4.52 7.572A2 2 0 0 1 4 6.227z\"/></svg>";

/// A page's reference sections: headers for the page and the fragment files.
pub(super) struct Sections {
    pub tagged: String,
    pub linked: String,
    pub unlinked: String,
    pub files: Vec<(String, String)>,
    pub diagnostics: Vec<String>,
}

/// Everything a page's sections need to render.
pub(super) struct Context<'r, 'a> {
    pub config: &'r ReferenceConfig,
    pub names: &'r PageNameIndex<String>,
    pub targets: &'r ReferenceTargets<String>,
    pub documents: &'r BTreeMap<String, EmbedPage<'a>>,
}

pub(super) struct References<'a> {
    index: ReferenceIndex<String>,
    sources: BTreeMap<String, &'a GardenDocument>,
    titles: BTreeMap<String, String>,
    dates: BTreeMap<String, JournalDate>,
    property_targets: Option<BTreeMap<String, BTreeSet<String>>>,
    property_rows: BTreeMap<String, BTreeMap<String, String>>,
    show_brackets: bool,
}

impl<'a> References<'a> {
    pub fn new(
        selected: &[(String, String, &'a GardenDocument)],
        names: &PageNameIndex<String>,
        property_views: Option<&BTreeMap<String, super::properties::PropertyView>>,
        destinations: BTreeMap<String, String>,
        dates: BTreeMap<String, JournalDate>,
        show_brackets: bool,
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
        let property_targets: Option<BTreeMap<_, _>> = property_views.map(|views| {
            views
                .iter()
                .map(|(route, view)| (route.clone(), view.targets.clone()))
                .collect()
        });
        let property_rows = property_views
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
            .unwrap_or_default();
        let contributions = selected
            .iter()
            .map(|(_, route, document)| {
                Contribution::from_document(
                    route.clone(),
                    document,
                    &resolve,
                    property_targets
                        .as_ref()
                        .map(|targets| targets.get(route).cloned().unwrap_or_default())
                        .as_ref(),
                )
            })
            .collect();
        let aliases: BTreeMap<_, _> = selected
            .iter()
            .map(|(title, route, document)| (route, page_alias_spellings(document, title)))
            .collect();
        let pages = destinations.iter().map(|(route, title)| PageEntry {
            id: route.clone(),
            title: title.clone(),
            aliases: aliases.get(route).cloned().unwrap_or_default(),
        });
        Self {
            index: ReferenceIndex::build(pages, contributions),
            sources: selected
                .iter()
                .map(|(_, route, document)| (route.clone(), *document))
                .collect(),
            titles: destinations,
            dates,
            property_targets,
            property_rows,
            show_brackets,
        }
    }

    /// Source anchors a page keeps so that linked references can open its blocks.
    pub fn source_anchors(&self, route: &str) -> BTreeSet<usize> {
        let route = route.to_owned();
        let mut anchors = BTreeSet::new();
        for id in self.index.blocks_of(&route) {
            if !self.index.block(id).refs.iter().any(|page| page != &route) {
                continue;
            }
            anchors.extend(self.index.block(id).anchor);
            for ancestor in self.index.ancestors(id) {
                anchors.extend(self.index.block(ancestor).anchor);
            }
        }
        anchors
    }

    /// The shared reference index, keyed by route.
    pub fn index(&self) -> &ReferenceIndex<String> {
        &self.index
    }

    /// Number of distinct pages that refer to `target`.
    pub fn page_count(&self, target: &str) -> usize {
        self.index.referring_pages(&target.to_owned())
    }

    fn title<'t>(&'t self, route: &'t str) -> &'t str {
        self.titles.get(route).map_or(route, String::as_str)
    }

    /// Groups of blocks by page: journals newest first, then pages by lower-cased title;
    /// within a page, top-level blocks first, then document order.
    fn groups(&self, ids: &[BlockId]) -> Vec<(String, Vec<BlockId>)> {
        let mut groups: BTreeMap<&String, Vec<BlockId>> = BTreeMap::new();
        for &id in ids {
            groups.entry(self.index.page(id)).or_default().push(id);
        }
        let mut groups: Vec<_> = groups
            .into_iter()
            .map(|(route, mut blocks)| {
                blocks.sort_by_key(|&id| {
                    let block = self.index.block(id);
                    (block.parent.is_some(), block.start)
                });
                (route.clone(), blocks)
            })
            .collect();
        groups.sort_by_cached_key(|(route, _)| match self.dates.get(route) {
            Some(date) => (0, Some(std::cmp::Reverse(*date)), String::new()),
            None => (1, None, self.title(route).to_lowercase()),
        });
        groups
    }

    pub fn render(
        &self,
        route: &str,
        context: &Context<'_, '_>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> Sections {
        let target = route.to_owned();
        // One flat directory: a directory per page would cost a `mkdir` for each.
        let prefix = format!("{REFS}/{}", route.trim_end_matches(".html"));
        let mut sections = Sections {
            tagged: self.tagged(&target),
            linked: String::new(),
            unlinked: String::new(),
            files: Vec::new(),
            diagnostics: Vec::new(),
        };
        let linked = self.index.linked(&target);
        if !linked.is_empty() {
            self.linked(&target, &prefix, linked, context, asset, &mut sections);
        }
        self.unlinked(&target, &prefix, context, asset, &mut sections);
        sections
    }

    fn tagged(&self, target: &String) -> String {
        let pages = self.index.tagged_with(target);
        if pages.is_empty() {
            return String::new();
        }
        let mut html = format!(
            "<section class=\"references tagged-pages\"><details class=\"references-section\" open><summary><span class=\"ref-chevron\"></span><h2>Pages tagged with \"{}\"</h2></summary><ul>",
            render::escape(&self.title(target).to_lowercase())
        );
        for page in pages {
            html.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>",
                render::escape(page),
                render::escape(self.title(page))
            ));
        }
        html.push_str("</ul></details></section>");
        html
    }

    fn linked(
        &self,
        target: &String,
        prefix: &str,
        ids: &[BlockId],
        context: &Context<'_, '_>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        sections: &mut Sections,
    ) {
        let groups = self.groups(ids);
        // Blocks are numbered in group order; the number is the block's row in the data.
        let mut numbers = BTreeMap::new();
        for (_, blocks) in &groups {
            for &id in blocks {
                let next = numbers.len();
                numbers.insert(id, next);
            }
        }
        let mut pages: Vec<String> = Vec::new();
        let mut page_numbers: BTreeMap<String, usize> = BTreeMap::new();
        let mut number = |page: &String, pages: &mut Vec<String>| {
            *page_numbers.entry(page.clone()).or_insert_with(|| {
                pages.push(page.clone());
                pages.len() - 1
            })
        };
        let filter = self.index.starting_filter(target);
        let mut rows = vec![json!(null); numbers.len()];
        let mut filtered = 0;
        let files = self.paginate(&groups, |groups, source, blocks, file| {
            for &id in blocks {
                let path = self.index.path_refs(id);
                if !filter.is_empty() && filter.matches(&path) {
                    filtered += 1;
                }
                let path_numbers: Vec<_> = path
                    .iter()
                    .filter(|page| *page != target)
                    .map(|page| number(page, &mut pages))
                    .collect();
                // A block inside another referring block counts toward no filter option.
                let nested = self.index.block(id).parent.is_some_and(|parent| {
                    numbers.contains_key(&BlockId {
                        document: id.document,
                        block: parent,
                    })
                });
                rows[numbers[&id]] = if nested {
                    json!([file, path_numbers])
                } else {
                    let descendants: Vec<_> = self
                        .index
                        .descendant_refs(id)
                        .iter()
                        .filter(|page| *page != target)
                        .map(|page| number(page, &mut pages))
                        .collect();
                    json!([file, path_numbers, descendants])
                };
            }
            groups.group(
                source,
                blocks,
                Some(target),
                &numbers,
                context,
                asset,
                sections,
            )
        });
        let count = ids.len();
        let indexes = |set: &BTreeSet<String>| {
            set.iter()
                .filter_map(|page| page_numbers.get(page).copied())
                .collect::<Vec<_>>()
        };
        let data = json!({
            "count": count,
            "pages": pages.iter().map(|page| json!([self.title(page), page])).collect::<Vec<_>>(),
            "blocks": rows,
            "filter": [indexes(&filter.includes), indexes(&filter.excludes)],
        });
        let first = self.write_files(
            prefix,
            "linked",
            "Linked references",
            target,
            files,
            data,
            sections,
        );
        let open = if (count as i128) < i128::from(context.config.threshold) {
            " open"
        } else {
            ""
        };
        let heading = linked_heading((!filter.is_empty()).then_some(filtered), count);
        sections.linked = format!(
            "<section class=\"references linked-references\" id=\"linked-references\" data-refs=\"linked\" data-src=\"{first}\"><details class=\"references-section\"{open}><summary><span class=\"ref-chevron\"></span><h2>{heading}</h2><button type=\"button\" class=\"filter\" title=\"Filter\" aria-label=\"Filter\" data-state=\"{}\" hidden>{FUNNEL}</button></summary><noscript><a href=\"{first}\">Show linked references</a></noscript><div class=\"references-body\"><p role=\"status\"></p></div></details></section>",
            filter_state(&filter)
        );
    }

    fn unlinked(
        &self,
        target: &String,
        prefix: &str,
        context: &Context<'_, '_>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        sections: &mut Sections,
    ) {
        let ids = self.index.unlinked(target);
        let groups = self.groups(ids);
        let none = BTreeMap::new();
        // The linked copies and the page itself already report these blocks' diagnostics.
        let files = self.paginate(&groups, |groups, source, blocks, _| {
            let mut quiet = Sections {
                tagged: String::new(),
                linked: String::new(),
                unlinked: String::new(),
                files: Vec::new(),
                diagnostics: Vec::new(),
            };
            groups.group(source, blocks, None, &none, context, asset, &mut quiet)
        });
        // A page with no mentions has no file; the script shows the zero count.
        let (source, fallback) = if files.is_empty() {
            (String::new(), "No unlinked references.".to_owned())
        } else {
            let first = self.write_files(
                prefix,
                "unlinked",
                "Unlinked references",
                target,
                files,
                json!({"count": ids.len()}),
                sections,
            );
            (
                format!(" data-src=\"{first}\""),
                format!("<a href=\"{first}\">Show unlinked references</a>"),
            )
        };
        sections.unlinked = format!(
            "<section class=\"references linked-references unlinked-references\" data-refs=\"unlinked\"{source}><details class=\"references-section\"><summary><span class=\"ref-chevron\"></span><h2>Unlinked References</h2></summary><noscript>{fallback}</noscript><div class=\"references-body\"></div></details></section>"
        );
    }

    /// Render groups into files of about [`GROUPS_PER_FILE`] groups. `render` receives
    /// each group and the number of the file it lands in.
    fn paginate(
        &self,
        groups: &[(String, Vec<BlockId>)],
        mut render: impl FnMut(&Self, &str, &[BlockId], usize) -> String,
    ) -> Vec<String> {
        let mut files = Vec::new();
        let mut file = String::new();
        let mut in_file = 0;
        for (source, blocks) in groups {
            if in_file >= GROUPS_PER_FILE || file.len() > FILE_BYTES {
                files.push(std::mem::take(&mut file));
                in_file = 0;
            }
            file.push_str(&render(self, source, blocks, files.len()));
            in_file += 1;
        }
        if !file.is_empty() {
            files.push(file);
        }
        files
    }

    /// Write each file as a readable page that links back to the page and on to the next
    /// file. The first file also carries the section's data. Returns the first file's route.
    #[allow(clippy::too_many_arguments)]
    fn write_files(
        &self,
        prefix: &str,
        kind: &str,
        label: &str,
        target: &str,
        files: Vec<String>,
        mut data: serde_json::Value,
        sections: &mut Sections,
    ) -> String {
        let total = files.len();
        let stem = prefix.trim_start_matches(&format!("{REFS}/"));
        let name = |position: usize| format!("{stem}-{kind}-{}.html", position + 1);
        data["files"] = json!((0..total).map(name).collect::<Vec<_>>());
        // `<` could end the script element; `\u003c` is the same JSON string.
        let data = data.to_string().replace('<', "\\u003c");
        let title = render::escape(self.title(target));
        let target = render::escape(target);
        for (position, body) in files.into_iter().enumerate() {
            let next = if position + 1 < total {
                format!(
                    "<p><a href=\"{REFS}/{}\" rel=\"next\">More {}</a></p>",
                    name(position + 1),
                    label.to_lowercase()
                )
            } else {
                String::new()
            };
            let data = if position == 0 {
                format!(
                    "<script type=\"application/json\" class=\"references-data\">{data}</script>"
                )
            } else {
                String::new()
            };
            sections.files.push((
                format!("{REFS}/{}", name(position)),
                format!(
                    "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><base href=\"../\"><title>{label}: {title}</title>{data}</head><body><main><h1>{label}: <a href=\"{target}\">{title}</a></h1><div class=\"references-fragment\">{body}</div>{next}</main></body></html>\n"
                ),
            ));
        }
        format!("{REFS}/{}", name(0))
    }

    /// One page's group. `target` is set for linked references, which number their blocks,
    /// link their bullets to the source and show breadcrumbs.
    #[allow(clippy::too_many_arguments)]
    fn group(
        &self,
        source: &str,
        blocks: &[BlockId],
        target: Option<&String>,
        numbers: &BTreeMap<BlockId, usize>,
        context: &Context<'_, '_>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        sections: &mut Sections,
    ) -> String {
        let route = render::escape(source);
        let mut html = format!(
            "<div class=\"references-blocks-item\"><details class=\"references-group\" open><summary class=\"foldable-title\"><span class=\"ref-chevron\"></span><a href=\"{route}\">{}</a></summary>",
            render::escape(self.title(source))
        );
        let Some(document) = self.sources.get(source) else {
            html.push_str("</details></div>");
            return html;
        };
        let mut rendered: Vec<(usize, usize)> = Vec::new();
        for &id in blocks {
            let block = self.index.block(id);
            if !block.properties
                && rendered
                    .iter()
                    .any(|(start, end)| block.start >= *start && block.start < *end)
            {
                continue;
            }
            let number = numbers
                .get(&id)
                .map(|n| format!(" data-ref-block=\"{n}\""))
                .unwrap_or_default();
            if block.properties {
                let visible = target.is_some_and(|target| {
                    self.property_targets
                        .as_ref()
                        .and_then(|targets| targets.get(source))
                        .is_some_and(|targets| targets.contains(target))
                });
                html.push_str(&format!(
                    "<div class=\"references-block\"{number}><div class=\"blocks-container\"><p class=\"backlink-context\"><a href=\"{route}{}\">{}</a></p>",
                    if visible { "#page-properties" } else { "" },
                    if visible { "Page properties" } else { "Page reference" }
                ));
                if let Some(rows) = target.and_then(|target| {
                    self.property_rows
                        .get(source)
                        .and_then(|rows| rows.get(target))
                }) {
                    html.push_str(&format!("<dl class=\"backlink-properties\">{rows}</dl>"));
                }
                html.push_str("</div></div>");
                continue;
            }
            rendered.push((block.start, block.subtree_end));
            html.push_str("<div class=\"references-block\">");
            if target.is_some() && block.parent.is_some() {
                html.push_str(&self.breadcrumb(id, source, document, context, asset));
            }
            html.push_str("<div class=\"blocks-container\">");
            let mut copy = GardenDocument {
                relative_path: document.relative_path.clone(),
                kind: document.kind,
                source: String::new(),
                parsed: lsdoc::ast::Projection {
                    blocks: document.parsed.blocks[block.start..block.subtree_end].to_vec(),
                    refs: lsdoc::ast::Refs {
                        page: Vec::new(),
                        block: Vec::new(),
                    },
                },
            };
            for copied in &mut copy.parsed.blocks {
                if let Block::Bullet { level, .. } = copied {
                    *level = level.saturating_sub(block.level) + 1;
                }
            }
            // Referring blocks inside this one carry their numbers and link their bullets.
            let mut paths = BTreeMap::new();
            let mut entries = BTreeMap::new();
            if target.is_some() {
                for &inner in blocks {
                    let inner_block = self.index.block(inner);
                    if inner_block.properties
                        || inner_block.start < block.start
                        || inner_block.start >= block.subtree_end
                    {
                        continue;
                    }
                    if let Some(anchor) = inner_block.anchor {
                        paths.insert(anchor, numbers[&inner].to_string());
                        entries.insert(anchor, format!("{source}#source-block-{anchor}"));
                    }
                }
            }
            let copy_rendered = render::render_reference_context(
                &copy,
                document,
                context.names,
                context.targets,
                source,
                context.documents,
                render::ReferencePresentation {
                    open_depth: context.config.open_depth,
                    show_brackets: self.show_brackets,
                    entries,
                    paths,
                },
                asset,
            );
            html.push_str(&copy_rendered.html);
            sections.diagnostics.extend(copy_rendered.diagnostics);
            html.push_str("</div></div>");
        }
        html.push_str("</details></div>");
        html
    }

    /// The nearest three ancestors, each its first line, with `⋯` for older ones.
    fn breadcrumb(
        &self,
        id: BlockId,
        source: &str,
        document: &GardenDocument,
        context: &Context<'_, '_>,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> String {
        let ancestors = self.index.ancestors(id);
        let mut crumbs = Vec::new();
        if ancestors.len() > CRUMBS {
            crumbs.push("<span class=\"crumb-more\">⋯</span>".to_owned());
        }
        for ancestor in ancestors.iter().take(CRUMBS).rev() {
            let block = self.index.block(*ancestor);
            let Some(Block::Bullet { inline, .. }) = document.parsed.blocks.get(block.start) else {
                continue;
            };
            let label = render::render_crumb(
                inline,
                document,
                context.names,
                context.targets,
                source,
                context.documents,
                self.show_brackets,
                asset,
            );
            crumbs.push(match block.anchor {
                Some(anchor) => format!(
                    "<a href=\"{}#source-block-{anchor}\">{label}</a>",
                    render::escape(source)
                ),
                None => label,
            });
        }
        format!(
            "<div class=\"breadcrumb block-parents\">{}</div>",
            crumbs.join("<span class=\"crumb-sep\" aria-hidden=\"true\"></span>")
        )
    }
}

/// Logseq's count text: `N Linked References`, `1 Linked Reference`, `F of N Linked References`.
pub(super) fn linked_heading(filtered: Option<usize>, count: usize) -> String {
    let noun = if count == 1 {
        "Linked Reference"
    } else {
        "Linked References"
    };
    match filtered {
        Some(filtered) => format!("{filtered} of {count} {noun}"),
        None => format!("{count} {noun}"),
    }
}

fn filter_state(filter: &Filter<String>) -> &'static str {
    match (filter.includes.is_empty(), filter.excludes.is_empty()) {
        (true, true) => "none",
        (false, true) => "include",
        (true, false) => "exclude",
        (false, false) => "mixed",
    }
}
