//! Logseq's global graph and page graph, answered from the reference index that the CLI,
//! the language server and the publisher share.
//!
//! [`GraphIndex`] reads a [`ReferenceIndex`] once. [`GraphIndex::graph`] returns the global
//! graph for Logseq's settings (journals, orphan, built-in and excluded pages), and
//! [`GraphIndex::neighbourhood`] returns one page's page graph. Both follow Logseq's rules:
//! an edge for a block's reference to another page, for each page in `tags::` and for each
//! namespaced page's parent; property names count as references; pages that exist only
//! because a published page names them are nodes without a page.
//!
//! ```
//! use std::collections::BTreeSet;
//! use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
//! use logseq_gardener_sdk::graph::{GraphIndex, GraphSettings, GraphSource};
//! use logseq_gardener_sdk::references::{Contribution, PageEntry, ReferenceIndex};
//!
//! let document = |source: &str| GardenDocument {
//!     relative_path: "pages/foam.md".into(),
//!     kind: DocumentKind::Page,
//!     source: source.to_owned(),
//!     parsed: lsdoc::parse_to_projection(source),
//! };
//! let foam = document("tags:: Tool\n\n- Like [[Obsidian]]\n");
//! let resolve = |name: &str| (name.to_lowercase() == "foam").then_some("Foam");
//! let index = ReferenceIndex::build(
//!     [PageEntry { id: "Foam", title: "Foam".into(), aliases: vec![] }],
//!     vec![Contribution::from_document("Foam", &foam, &resolve, None)],
//! );
//! let none = BTreeSet::new();
//! let graph = GraphIndex::new(&GraphSource {
//!     index: &index,
//!     journals: &none,
//!     hidden: &BTreeSet::new(),
//!     is_journal_name: &|_| false,
//! })
//! .graph(GraphSettings::default());
//! // Obsidian and Tool have no file: they are reference-only nodes.
//! assert!(graph.has_edge("Foam", "Obsidian") && graph.has_edge("Foam", "Tool"));
//! assert!(graph.node("Obsidian").is_some_and(|node| node.page.is_none()));
//! ```
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::page_names::{built_in_page_title, lookup_key, namespace_parent, reference_key};
use crate::references::ReferenceIndex;

/// Which pages the global graph draws: Logseq's `:graph/settings`. Every switch is off by
/// default, as Logseq reads a missing setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GraphSettings {
    /// `:journal?`: journal pages and the references they make.
    pub journals: bool,
    /// `:orphan-pages?`: pages without an edge.
    pub orphan_pages: bool,
    /// `:builtin-pages?`: Logseq's built-in pages such as `TODO` and `card`.
    pub builtin_pages: bool,
    /// `:excluded-pages?`: pages with `exclude-from-graph-view:: true`.
    pub excluded_pages: bool,
}

impl GraphSettings {
    /// Every page of one journals setting, for a layout that covers all of them.
    pub fn everything(journals: bool) -> Self {
        Self {
            journals,
            orphan_pages: true,
            builtin_pages: true,
            excluded_pages: true,
        }
    }
}

/// The graph layout's forces: Logseq's `:graph/forcesettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForceSettings {
    /// `:link-dist`, 70 by default.
    pub link_distance: i32,
    /// `:charge-strength`, −600 by default.
    pub charge_strength: i32,
    /// `:charge-range`, 600 by default.
    pub charge_range: i32,
}

impl Default for ForceSettings {
    fn default() -> Self {
        Self {
            link_distance: 70,
            charge_strength: -600,
            charge_range: 600,
        }
    }
}

/// A journal page.
pub const JOURNAL: u8 = 1;
/// A page another page lists in its `tags::`.
pub const TAG: u8 = 2;
/// The namespace parent of another page.
pub const NAMESPACE_PARENT: u8 = 4;
/// One of Logseq's built-in pages.
pub const BUILT_IN: u8 = 8;
/// A page with `exclude-from-graph-view:: true`.
pub const EXCLUDED: u8 = 16;
/// A page without an edge.
pub const ORPHAN: u8 = 32;
/// A page without a file that exists because a published page names it.
pub const REFERENCE_ONLY: u8 = 64;
/// In a page graph with journals: a node that only a journal's reference brings in, which
/// the page graph leaves out without journals.
pub const JOURNAL_ONLY: u8 = 128;

/// Logseq's built-in pages, lower-cased.
const BUILT_IN_PAGES: [&str; 17] = [
    "now",
    "later",
    "todo",
    "doing",
    "done",
    "wait",
    "waiting",
    "canceled",
    "cancelled",
    "started",
    "in-progress",
    "a",
    "b",
    "c",
    "favorites",
    "contents",
    "card",
];

/// Property names that Logseq does not turn into pages: its editable and hidden built-in
/// properties, apart from `alias`, `aliases` and `tags`, whose values are references. The
/// flashcard properties are [`crate::cards::HIDDEN_PROPERTIES`].
const BUILT_IN_PROPERTIES: [&str; 48] = [
    "title",
    "icon",
    "template",
    "template-including-parent",
    "public",
    "filters",
    "exclude-from-graph-view",
    "logseq.query/nlp-date",
    "macro",
    "filetags",
    "alias",
    "aliases",
    "tags",
    "logseq.color",
    "logseq.table.version",
    "logseq.table.compact",
    "logseq.table.headers",
    "logseq.table.hover",
    "logseq.table.borders",
    "logseq.table.stripes",
    "logseq.table.max-width",
    "id",
    "custom-id",
    "background-color",
    "background_color",
    "heading",
    "collapsed",
    "created-at",
    "updated-at",
    "last-modified-at",
    "created_at",
    "last_modified_at",
    "query-table",
    "query-properties",
    "query-sort-by",
    "query-sort-desc",
    "ls-type",
    "hl-type",
    "hl-page",
    "hl-stamp",
    "hl-color",
    "logseq.macro-name",
    "logseq.macro-arguments",
    "logseq.order-list-type",
    "logseq.tldraw.page",
    "logseq.tldraw.shape",
    "todo",
    "doing",
];

/// Whether a property name becomes a page, as in Logseq's file graphs.
fn property_page(name: &str) -> bool {
    !name.is_empty()
        && !BUILT_IN_PROPERTIES.contains(&name)
        && !crate::cards::is_hidden_property(name)
        && !matches!(name, "now" | "later" | "done")
}

/// Names Logseq never draws: block UUIDs and asset paths.
fn uuid_or_asset(name: &str) -> bool {
    let uuid = name.len() == 36
        && name.char_indices().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        });
    uuid || name == ".."
        || name.starts_with("../assets/")
        || name.starts_with("assets/")
        || [".gif", ".jpg", ".png"]
            .iter()
            .any(|end| name.to_lowercase().ends_with(end))
}

/// A page in a graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode<K> {
    /// The page's title as its file or first reference spells it.
    pub title: String,
    /// The published page; `None` for a reference-only page.
    pub page: Option<K>,
    /// Logseq's link count for the node: every relation that touches it, counted with
    /// repeats, which sizes the node.
    pub degree: u32,
    /// [`JOURNAL`], [`TAG`], [`NAMESPACE_PARENT`], [`BUILT_IN`], [`EXCLUDED`], [`ORPHAN`],
    /// [`REFERENCE_ONLY`] and [`JOURNAL_ONLY`].
    pub flags: u8,
}

/// Nodes, distinct undirected edges between them, and Logseq's `N links` count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph<K> {
    pub nodes: Vec<GraphNode<K>>,
    /// Pairs of node positions, smaller first, sorted, without repeats or self-edges.
    pub edges: Vec<(u32, u32)>,
    /// Every relation Logseq counts in `N links`, with repeats and with relations whose
    /// other end is not drawn.
    pub links: usize,
}

impl<K> Graph<K> {
    /// The node whose title matches, ignoring case.
    pub fn node(&self, title: &str) -> Option<&GraphNode<K>> {
        self.position(title).map(|index| &self.nodes[index])
    }

    /// The position of the node whose title matches, ignoring case.
    pub fn position(&self, title: &str) -> Option<usize> {
        let key = reference_key(title);
        self.nodes
            .iter()
            .position(|node| reference_key(&node.title) == key)
    }

    /// Whether an edge joins the two titled nodes.
    pub fn has_edge(&self, left: &str, right: &str) -> bool {
        match (self.position(left), self.position(right)) {
            (Some(a), Some(b)) => {
                let pair = (a.min(b) as u32, a.max(b) as u32);
                self.edges.binary_search(&pair).is_ok()
            }
            _ => false,
        }
    }

    /// Titles of the nodes joined to the titled node, sorted.
    pub fn neighbours(&self, title: &str) -> Vec<&str> {
        let Some(at) = self.position(title) else {
            return Vec::new();
        };
        let at = at as u32;
        let mut result: Vec<&str> = self
            .edges
            .iter()
            .filter_map(|&(a, b)| match (a == at, b == at) {
                (true, _) => Some(self.nodes[b as usize].title.as_str()),
                (_, true) => Some(self.nodes[a as usize].title.as_str()),
                _ => None,
            })
            .collect();
        result.sort_unstable();
        result
    }
}

impl<K: Clone> Graph<K> {
    /// The page graph without journals: drops [`JOURNAL_ONLY`] nodes and their edges.
    pub fn without_journal_only(&self) -> Self {
        let keep: Vec<bool> = self
            .nodes
            .iter()
            .map(|node| node.flags & JOURNAL_ONLY == 0)
            .collect();
        let mut positions = vec![u32::MAX; self.nodes.len()];
        let mut nodes = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if keep[index] {
                positions[index] = nodes.len() as u32;
                nodes.push(node.clone());
            }
        }
        let edges: Vec<_> = self
            .edges
            .iter()
            .filter(|(a, b)| keep[*a as usize] && keep[*b as usize])
            .map(|&(a, b)| (positions[a as usize], positions[b as usize]))
            .collect();
        let mut degree = vec![0; nodes.len()];
        for &(a, b) in &edges {
            degree[a as usize] += 1;
            degree[b as usize] += 1;
        }
        for (node, degree) in nodes.iter_mut().zip(degree) {
            node.degree = degree;
        }
        Self {
            links: edges.len(),
            nodes,
            edges,
        }
    }
}

/// What a graph is computed from besides the reference index.
pub struct GraphSource<'a, K> {
    /// The index of the published pages.
    pub index: &'a ReferenceIndex<K>,
    /// The published journal pages.
    pub journals: &'a BTreeSet<K>,
    /// [`lookup_key`]s of pages that exist but are not published, such as private pages or
    /// pages outside the selected namespaces. They are never nodes.
    pub hidden: &'a BTreeSet<String>,
    /// Whether a name without a page names a journal, such as `Oct 23rd, 2026`.
    pub is_journal_name: &'a dyn Fn(&str) -> bool,
}

struct NodeInfo<K> {
    title: String,
    page: Option<K>,
    journal: bool,
    excluded: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Relation {
    /// A reference; the flag records whether a journal made it.
    Reference {
        from_journal: bool,
    },
    Tag,
    Namespace,
}

/// The garden's pages and relations, read once from the reference index; it answers the
/// global graph for any settings and any page's page graph.
pub struct GraphIndex<K> {
    nodes: Vec<NodeInfo<K>>,
    /// Every relation as (from, to), with its kind, in the order Logseq would list them.
    relations: Vec<(u32, u32, Relation)>,
    /// Per node: the nodes its blocks name, with namespace parents, for page graphs.
    refs: Vec<Vec<u32>>,
    /// Per node: the nodes whose blocks name it, with namespace parents.
    mentions: Vec<Vec<u32>>,
    /// Per node: the pages in its `tags::`.
    tags: Vec<Vec<u32>>,
    /// Per node: its namespace parent.
    parent: Vec<Option<u32>>,
    /// Per node: whether it is another node's namespace parent.
    is_parent: Vec<bool>,
    by_key: HashMap<String, u32>,
}

struct Builder<'s, 'a, K> {
    source: &'s GraphSource<'a, K>,
    nodes: Vec<NodeInfo<K>>,
    by_key: HashMap<String, u32>,
}

impl<K: Ord + Clone> Builder<'_, '_, K> {
    /// The node for a name: a published page, an existing reference-only page or a new one.
    /// Hidden pages and Logseq's UUID and asset names have none. A name keeps the spaces at
    /// its ends, as in Logseq: `Release Notes ` from `[[Release Notes ]]` and `MacOS ` from
    /// `MacOS /Applications Folder` are nodes apart from `Release Notes` and `MacOS`.
    fn name(&mut self, name: &str) -> Option<u32> {
        let key = reference_key(name);
        if let Some(&found) = self.by_key.get(&key) {
            return Some(found);
        }
        if name.trim().is_empty() || self.source.hidden.contains(&key) || uuid_or_asset(name) {
            return None;
        }
        let index = self.nodes.len() as u32;
        // A built-in page keeps Logseq's spelling; another page takes its first reference's.
        let title = built_in_page_title(name).unwrap_or(name);
        self.nodes.push(NodeInfo {
            title: title.to_owned(),
            page: None,
            journal: (self.source.is_journal_name)(name),
            excluded: false,
        });
        self.by_key.insert(key, index);
        Some(index)
    }
}

impl<K: Ord + Clone> GraphIndex<K> {
    /// Read the pages and relations of a reference index.
    pub fn new(source: &GraphSource<'_, K>) -> Self {
        let index = source.index;
        let contributions = index.contributions();
        let excluded: BTreeSet<&K> = contributions
            .iter()
            .filter(|c| c.excluded_from_graph())
            .map(|c| &c.page)
            .collect();
        let mut builder = Builder {
            source,
            nodes: Vec::new(),
            by_key: HashMap::new(),
        };
        let mut ids = BTreeMap::new();
        for (page, title) in index.pages() {
            let key = reference_key(title);
            if builder.by_key.contains_key(&key) {
                continue;
            }
            let at = builder.nodes.len() as u32;
            builder.nodes.push(NodeInfo {
                title: title.to_owned(),
                page: Some(page.clone()),
                journal: source.journals.contains(page),
                excluded: excluded.contains(page),
            });
            builder.by_key.insert(key, at);
            ids.insert(page.clone(), at);
        }
        let mut relations = Vec::new();
        let mut refs: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        let mut tags: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        for (document, contribution) in contributions.iter().enumerate() {
            let Some(&from) = ids.get(&contribution.page) else {
                continue;
            };
            let from_journal = builder.nodes[from as usize].journal;
            let mut direct = BTreeSet::new();
            let mut expanded = BTreeSet::new();
            for (block, graph) in contribution.graph_refs().iter().enumerate() {
                direct.extend(graph.direct.iter().filter_map(|page| ids.get(page)));
                expanded.extend(
                    index.expanded_refs(document)[block]
                        .iter()
                        .filter_map(|page| ids.get(page)),
                );
                for name in &graph.unresolved {
                    direct.extend(builder.name(name));
                    let mut rest = name.as_str();
                    expanded.extend(builder.name(rest));
                    while let Some(parent) = namespace_parent(rest) {
                        expanded.extend(builder.name(parent));
                        rest = parent;
                    }
                }
                for name in graph.property_names.iter().filter(|n| property_page(n)) {
                    if let Some(node) = builder.name(name) {
                        direct.insert(node);
                        expanded.insert(node);
                    }
                }
            }
            direct.remove(&from);
            expanded.remove(&from);
            relations.extend(
                direct
                    .iter()
                    .map(|&to| (from, to, Relation::Reference { from_journal })),
            );
            refs.entry(from).or_default().extend(expanded);
            let page_tags: BTreeSet<u32> = contribution
                .tags
                .iter()
                .filter_map(|page| ids.get(page).copied())
                .chain(
                    contribution
                        .unresolved_tags()
                        .iter()
                        .filter_map(|name| builder.name(name)),
                )
                .filter(|&tag| tag != from)
                .collect();
            relations.extend(page_tags.iter().map(|&to| (from, to, Relation::Tag)));
            tags.entry(from).or_default().extend(page_tags);
        }
        // Namespace parents; a reference-only child can bring in a reference-only parent.
        let mut parent = Vec::new();
        let mut at = 0;
        while at < builder.nodes.len() {
            let title = builder.nodes[at].title.clone();
            let up = namespace_parent(&title)
                .and_then(|name| builder.name(name))
                .filter(|&up| up as usize != at);
            if let Some(up) = up {
                relations.push((at as u32, up, Relation::Namespace));
            }
            parent.push(up);
            at += 1;
        }
        let count = builder.nodes.len();
        let mut is_parent = vec![false; count];
        for up in parent.iter().flatten() {
            is_parent[*up as usize] = true;
        }
        let mut refs_list = vec![Vec::new(); count];
        let mut mentions = vec![Vec::new(); count];
        for (from, targets) in refs {
            for &to in &targets {
                mentions[to as usize].push(from);
            }
            refs_list[from as usize] = targets.into_iter().collect();
        }
        let mut tags_list = vec![Vec::new(); count];
        for (from, targets) in tags {
            tags_list[from as usize] = targets.into_iter().collect();
        }
        Self {
            nodes: builder.nodes,
            relations,
            refs: refs_list,
            mentions,
            tags: tags_list,
            parent,
            is_parent,
            by_key: builder.by_key,
        }
    }

    fn flags(&self, node: usize) -> u8 {
        let info = &self.nodes[node];
        let mut flags = 0;
        if info.journal {
            flags |= JOURNAL;
        }
        if info.excluded {
            flags |= EXCLUDED;
        }
        if info.page.is_none() {
            flags |= REFERENCE_ONLY;
        }
        if BUILT_IN_PAGES.contains(&lookup_key(&info.title).as_str()) {
            flags |= BUILT_IN;
        }
        flags
    }

    /// The global graph for Logseq's settings.
    pub fn graph(&self, settings: GraphSettings) -> Graph<K> {
        let count = self.nodes.len();
        let mut degree = vec![0_u32; count];
        let mut flags: Vec<u8> = (0..count).map(|node| self.flags(node)).collect();
        let mut linked = vec![false; count];
        let relations: Vec<_> = self
            .relations
            .iter()
            .filter(|(_, _, kind)| {
                settings.journals || *kind != Relation::Reference { from_journal: true }
            })
            .collect();
        for &&(from, to, kind) in &relations {
            degree[from as usize] += 1;
            degree[to as usize] += 1;
            linked[from as usize] = true;
            linked[to as usize] = true;
            match kind {
                Relation::Tag => flags[to as usize] |= TAG,
                Relation::Namespace => flags[to as usize] |= NAMESPACE_PARENT,
                Relation::Reference { .. } => {}
            }
        }
        let mut positions = vec![u32::MAX; count];
        let mut nodes = Vec::new();
        for node in 0..count {
            if !linked[node] {
                flags[node] |= ORPHAN;
            }
            let f = flags[node];
            let shown = (settings.journals || f & JOURNAL == 0)
                && (settings.builtin_pages || f & BUILT_IN == 0)
                && (settings.excluded_pages || f & EXCLUDED == 0)
                && (settings.orphan_pages || f & ORPHAN == 0);
            if shown {
                positions[node] = nodes.len() as u32;
                nodes.push(GraphNode {
                    title: self.nodes[node].title.clone(),
                    page: self.nodes[node].page.clone(),
                    degree: degree[node],
                    flags: f,
                });
            }
        }
        let mut edges: Vec<(u32, u32)> = relations
            .iter()
            .filter_map(|&&(from, to, _)| {
                let (a, b) = (positions[from as usize], positions[to as usize]);
                (a != u32::MAX && b != u32::MAX && a != b).then(|| (a.min(b), a.max(b)))
            })
            .collect();
        edges.sort_unstable();
        edges.dedup();
        Graph {
            nodes,
            edges,
            links: relations.len(),
        }
    }

    /// The node position of a published page.
    fn page_node(&self, page: &K) -> Option<u32> {
        self.nodes
            .iter()
            .position(|node| node.page.as_ref() == Some(page))
            .map(|at| at as u32)
    }

    /// A page's page graph, as Logseq's Page graph card builds it: the page, the pages its
    /// blocks name, the pages that name it and its tags, with the edges among them. With
    /// `journals`, journal pages that name the page are included and flagged
    /// [`JOURNAL_ONLY`] when nothing else brings them in. A journal page's own page graph is
    /// the same either way. Returns `None` for a page the index does not hold.
    pub fn neighbourhood(&self, page: &K, journals: bool) -> Option<Graph<K>> {
        let centre = self.page_node(page)?;
        let mut stamp = vec![0_u8; self.nodes.len()];
        Some(self.neighbourhood_at(centre, journals, &mut stamp))
    }

    /// Every published page's page graph with journals, using one scratch buffer.
    pub fn neighbourhoods(&self) -> impl Iterator<Item = (&K, Graph<K>)> + '_ {
        let mut stamp = vec![0_u8; self.nodes.len()];
        self.nodes.iter().enumerate().filter_map(move |(at, node)| {
            let page = node.page.as_ref()?;
            Some((page, self.neighbourhood_at(at as u32, true, &mut stamp)))
        })
    }

    fn neighbourhood_at(&self, centre: u32, journals: bool, stamp: &mut [u8]) -> Graph<K> {
        // Logseq's "other pages" are the pages the centre names and the pages that name it;
        // a tag alone is a node without edges to them.
        const OTHER: u8 = 1;
        const JOURNAL_MENTION: u8 = 2;
        const TAG_ONLY: u8 = 4;
        const CENTRE: u8 = 5;
        let other = |mark: u8| mark == OTHER || mark == JOURNAL_MENTION;
        let centre_journal = self.nodes[centre as usize].journal;
        let mut order = vec![centre];
        stamp[centre as usize] = CENTRE;
        for &node in &self.refs[centre as usize] {
            if stamp[node as usize] == 0 {
                stamp[node as usize] = OTHER;
                order.push(node);
            }
        }
        for &node in &self.tags[centre as usize] {
            if stamp[node as usize] == 0 {
                stamp[node as usize] = TAG_ONLY;
                order.push(node);
            }
        }
        let mut mentioned = Vec::new();
        for &node in &self.mentions[centre as usize] {
            let journal = self.nodes[node as usize].journal && !centre_journal;
            if journal && !journals {
                continue;
            }
            mentioned.push(node);
            match stamp[node as usize] {
                0 => {
                    stamp[node as usize] = if journal { JOURNAL_MENTION } else { OTHER };
                    order.push(node);
                }
                TAG_ONLY => stamp[node as usize] = OTHER,
                _ => {}
            }
        }
        order[1..].sort_unstable();
        let mut positions: HashMap<u32, u32> = HashMap::with_capacity(order.len());
        for (position, &node) in order.iter().enumerate() {
            positions.insert(node, position as u32);
        }
        let mut edges = Vec::new();
        let mut add = |a: u32, b: u32| {
            if a != b {
                let (a, b) = (positions[&a], positions[&b]);
                edges.push((a.min(b), a.max(b)));
            }
        };
        for &to in self.refs[centre as usize]
            .iter()
            .chain(&self.tags[centre as usize])
            .chain(&mentioned)
        {
            add(centre, to);
        }
        for &node in &order {
            if other(stamp[node as usize]) {
                for &to in &self.refs[node as usize] {
                    if other(stamp[to as usize]) {
                        add(node, to);
                    }
                }
            }
            if let Some(up) = self.parent[node as usize]
                && stamp[up as usize] != 0
            {
                add(node, up);
            }
        }
        edges.sort_unstable();
        edges.dedup();
        let mut degree = vec![0_u32; order.len()];
        for &(a, b) in &edges {
            degree[a as usize] += 1;
            degree[b as usize] += 1;
        }
        let nodes = order
            .iter()
            .enumerate()
            .map(|(position, &node)| {
                let mut flags = self.flags(node as usize);
                if self.tags[centre as usize].contains(&node) {
                    flags |= TAG;
                }
                if self.is_parent[node as usize] {
                    flags |= NAMESPACE_PARENT;
                }
                if stamp[node as usize] == JOURNAL_MENTION {
                    flags |= JOURNAL_ONLY;
                }
                GraphNode {
                    title: self.nodes[node as usize].title.clone(),
                    page: self.nodes[node as usize].page.clone(),
                    degree: degree[position],
                    flags,
                }
            })
            .collect();
        for &node in &order {
            stamp[node as usize] = 0;
        }
        Graph {
            links: edges.len(),
            nodes,
            edges,
        }
    }

    /// The node for a title, ignoring case.
    pub fn contains(&self, title: &str) -> bool {
        self.by_key.contains_key(&reference_key(title))
    }

    /// The title of a node's namespace parent, ignoring case: `A/B` for `A/B/C`, whether or
    /// not either page has a file. `None` for a title without `/` or a title the graph lacks.
    pub fn namespace_parent(&self, title: &str) -> Option<&str> {
        let at = *self.by_key.get(&reference_key(title))?;
        let up = self.parent[at as usize]?;
        Some(&self.nodes[up as usize].title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::{DocumentKind, GardenDocument};
    use crate::references::{Contribution, PageEntry};

    fn document(source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: "pages/x.md".into(),
            kind: DocumentKind::Page,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    /// Published pages and their sources; a title starting `J ` is a journal.
    struct Garden {
        pages: Vec<(&'static str, &'static str)>,
        hidden: BTreeSet<String>,
    }

    impl Garden {
        fn new(pages: &[(&'static str, &'static str)]) -> Self {
            Self {
                pages: pages.to_vec(),
                hidden: BTreeSet::new(),
            }
        }

        fn resolve(&self, name: &str) -> Option<String> {
            if crate::page_names::has_outer_space(name) {
                return None;
            }
            let key = lookup_key(name);
            self.pages
                .iter()
                .find(|(title, _)| lookup_key(title) == key)
                .map(|(title, _)| (*title).to_owned())
        }

        fn contribution(&self, page: &str, source: &str) -> Contribution<String> {
            Contribution::from_document(
                page.to_owned(),
                &document(source),
                &|name| self.resolve(name),
                None,
            )
        }

        fn index(&self) -> ReferenceIndex<String> {
            ReferenceIndex::build(
                self.pages.iter().map(|(title, _)| PageEntry {
                    id: (*title).to_owned(),
                    title: (*title).to_owned(),
                    aliases: vec![],
                }),
                self.pages
                    .iter()
                    .map(|(title, source)| self.contribution(title, source))
                    .collect(),
            )
        }

        fn graph_index(&self, index: &ReferenceIndex<String>) -> GraphIndex<String> {
            let journals = self
                .pages
                .iter()
                .filter(|(title, _)| title.starts_with("J "))
                .map(|(title, _)| (*title).to_owned())
                .collect();
            GraphIndex::new(&GraphSource {
                index,
                journals: &journals,
                hidden: &self.hidden,
                is_journal_name: &|name| name.starts_with("J "),
            })
        }

        fn graph(&self, settings: GraphSettings) -> Graph<String> {
            self.graph_index(&self.index()).graph(settings)
        }
    }

    #[test]
    fn three_relations() {
        let garden = Garden::new(&[
            ("Foam", "tags:: Tool\n\n- Like [[Obsidian]]\n"),
            ("Foam/Plugin", "- A plugin\n"),
            ("Obsidian", "- An editor\n"),
            ("Tool", "- Tools\n"),
        ]);
        let graph = garden.graph(GraphSettings::default());
        assert!(graph.has_edge("Foam", "Obsidian"));
        assert!(graph.has_edge("Foam", "Tool"));
        assert!(graph.has_edge("Foam/Plugin", "Foam"));
        assert_eq!(graph.edges.len(), 3);
        assert!(graph.node("Tool").unwrap().flags & TAG != 0);
        assert!(graph.node("Foam").unwrap().flags & NAMESPACE_PARENT != 0);
        assert_eq!(graph.node("Foam").unwrap().page.as_deref(), Some("Foam"));
    }

    #[test]
    fn distinct_edges_count_every_link() {
        let garden = Garden::new(&[
            ("mise", "tags:: tmux\n\n- Runs [[tmux]]\n"),
            ("tmux", "- A multiplexer\n"),
        ]);
        let graph = garden.graph(GraphSettings::default());
        assert_eq!(graph.edges.len(), 1);
        // The tags property names tmux as a reference and as a tag.
        assert_eq!(graph.links, 2);
        assert_eq!(graph.node("tmux").unwrap().degree, 2);
    }

    #[test]
    fn nested_namespace_edges_follow_what_the_page_links() {
        let pages = [
            ("tmux/Command", "- Commands\n"),
            ("tmux/Command/new-session", "- New\n"),
        ];
        let child_only = Garden::new(&[
            ("notes", "- [[tmux/Command/new-session]]\n"),
            pages[0],
            pages[1],
        ]);
        let graph = child_only.graph(GraphSettings::default());
        assert!(graph.has_edge("notes", "tmux/Command/new-session"));
        // The index adds tmux/Command and tmux to the block's references; the graph does not.
        assert!(!graph.has_edge("notes", "tmux/Command"));
        assert!(!graph.has_edge("notes", "tmux"));
        let both = Garden::new(&[
            (
                "notes",
                "- [[tmux/Command/new-session]]\n- [[tmux/Command]]\n",
            ),
            pages[0],
            pages[1],
        ]);
        assert!(
            both.graph(GraphSettings::default())
                .has_edge("notes", "tmux/Command")
        );
    }

    #[test]
    fn property_names_are_graph_references_only() {
        let garden = Garden::new(&[(
            "2026/t4/w43",
            "prev:: [[2026/t4/w42]]\nnext:: [[2026/t4/w44]]\nicon:: x\n\n- Week\n",
        )]);
        let index = garden.index();
        let graph = garden.graph_index(&index).graph(GraphSettings::default());
        for name in ["prev", "next"] {
            assert!(graph.has_edge("2026/t4/w43", name), "{name}");
            assert!(graph.node(name).unwrap().flags & REFERENCE_ONLY != 0);
        }
        assert!(graph.node("icon").is_none());
        // Linked references stay as they were: nothing links to `prev`.
        assert!(index.title(&"prev".to_owned()).is_none());
    }

    #[test]
    fn default_settings_leave_out_journals_built_ins_excluded_and_orphans() {
        let garden = Garden::new(&[
            ("J Oct 23", "- [[tmux]] and [[Only from journal]]\n"),
            ("tmux", "- [[TODO]] [[mise]]\n"),
            ("mise", "- tools\n"),
            ("Hidden", "exclude-from-graph-view:: true\n\n- [[mise]]\n"),
            ("Alone", "- nothing\n"),
        ]);
        let graph = garden.graph(GraphSettings::default());
        let titles: Vec<_> = graph.nodes.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, ["mise", "tmux"]);
        assert_eq!(graph.edges.len(), 1);
        let all = garden.graph(GraphSettings::everything(true));
        for title in ["J Oct 23", "Hidden", "Alone", "TODO", "Only from journal"] {
            assert!(all.node(title).is_some(), "{title}");
        }
        assert!(all.node("TODO").unwrap().flags & BUILT_IN != 0);
        assert!(all.node("Hidden").unwrap().flags & EXCLUDED != 0);
        assert!(all.node("Alone").unwrap().flags & ORPHAN != 0);
        assert!(all.node("J Oct 23").unwrap().flags & JOURNAL != 0);
        assert!(all.has_edge("J Oct 23", "tmux"));
        // Without journals, the journal's references are left out.
        let journals_off = garden.graph(GraphSettings::everything(false));
        assert!(journals_off.node("J Oct 23").is_none());
        assert!(
            journals_off.node("Only from journal").unwrap().flags & ORPHAN != 0,
            "only a journal names it"
        );
    }

    #[test]
    fn excluded_page_and_its_edges_are_absent() {
        let garden = Garden::new(&[
            ("Hidden", "exclude-from-graph-view:: true\n\n- [[mise]]\n"),
            ("mise", "- [[tmux]]\n"),
            ("tmux", "- tools\n"),
        ]);
        let graph = garden.graph(GraphSettings::default());
        assert!(graph.node("Hidden").is_none());
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn unpublished_pages_and_their_references_never_appear() {
        // The private page is not published, so it has no contribution, and its title is hidden.
        let mut garden = Garden::new(&[("Public", "- [[Secret Neighbour]]\n")]);
        garden.hidden.insert(lookup_key("Secret Neighbour"));
        let graph = garden.graph(GraphSettings::everything(true));
        assert!(graph.node("Secret Neighbour").is_none());
        assert!(graph.node("Secret Project").is_none());
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn reference_only_pages_are_nodes_without_a_page() {
        let garden = Garden::new(&[("Video", "- {{video x}} [[Contrando]] [[x/y/z]]\n")]);
        let graph = garden.graph(GraphSettings::default());
        let node = graph.node("Contrando").unwrap();
        assert!(node.page.is_none() && node.flags & REFERENCE_ONLY != 0);
        // A reference-only namespace child brings in its parents.
        assert!(graph.has_edge("x/y/z", "x/y") && graph.has_edge("x/y", "x"));
        assert!(!graph.has_edge("Video", "x/y"));
    }

    #[test]
    fn built_in_pages_without_a_file_keep_logseqs_spelling() {
        let garden = Garden::new(&[
            ("Alpha", "- #c #todo #favorites #Card #started\n"),
            ("Beta", "- TODO [#C] write\n- STARTED go\n"),
        ]);
        let graph = garden.graph(GraphSettings::everything(true));
        let titles: Vec<&str> = ["c", "todo", "favorites", "card", "started"]
            .iter()
            .map(|name| graph.node(name).unwrap().title.as_str())
            .collect();
        // `STARTED` is no built-in page in Logseq, so the first reference spells it.
        assert_eq!(titles, ["C", "TODO", "Favorites", "card", "started"]);
    }

    #[test]
    fn uuid_and_asset_names_are_never_nodes() {
        let garden = Garden::new(&[(
            "Page",
            "- [[6650a1b2-0000-4000-8000-000000000001]] [[../assets/a.png]] [[ok]]\n",
        )]);
        let graph = garden.graph(GraphSettings::default());
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn page_graph_follows_logseqs_page_graph() {
        let garden = Garden::new(&[
            (
                "Foam",
                "tags:: Tool\n\n- Like [[Obsidian]] and [[VS Code]]\n",
            ),
            ("Obsidian", "- Compare [[VS Code]]\n"),
            ("VS Code", "- An editor\n"),
            ("Notes", "- [[Foam]] notes\n"),
            ("Tool", "- [[Unrelated]]\n"),
            ("Unrelated", "- [[VS Code]]\n"),
        ]);
        let index = garden.index();
        let graph = garden
            .graph_index(&index)
            .neighbourhood(&"Foam".to_owned(), false)
            .unwrap();
        let titles: Vec<_> = graph.nodes.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles[0], "Foam");
        let mut rest = titles[1..].to_vec();
        rest.sort_unstable();
        assert_eq!(rest, ["Notes", "Obsidian", "Tool", "VS Code"]);
        assert!(
            graph.has_edge("Obsidian", "VS Code"),
            "edges among neighbours"
        );
        assert!(graph.has_edge("Foam", "Tool") && graph.has_edge("Notes", "Foam"));
        assert_eq!(graph.edges.len(), 5);
        assert!(graph.node("Tool").unwrap().flags & TAG != 0);
    }

    #[test]
    fn page_graph_journals_toggle() {
        let garden = Garden::new(&[
            ("J Oct 23", "- [[tmux]] with [[mise]]\n"),
            ("tmux", "- [[mise]]\n"),
            ("mise", "- tools\n"),
        ]);
        let index = garden.index();
        let graphs = garden.graph_index(&index);
        let on = graphs.neighbourhood(&"tmux".to_owned(), true).unwrap();
        let off = graphs.neighbourhood(&"tmux".to_owned(), false).unwrap();
        assert!(on.has_edge("J Oct 23", "tmux") && on.has_edge("J Oct 23", "mise"));
        assert!(on.node("J Oct 23").unwrap().flags & JOURNAL_ONLY != 0);
        assert!(off.node("J Oct 23").is_none());
        assert_eq!(on.without_journal_only(), off);
        // A journal page's own page graph does not depend on the switch.
        let journal = "J Oct 23".to_owned();
        assert_eq!(
            graphs
                .neighbourhood(&journal, true)
                .unwrap()
                .without_journal_only(),
            graphs.neighbourhood(&journal, false).unwrap()
        );
        let all: Vec<_> = graphs
            .neighbourhoods()
            .map(|(page, _)| page.clone())
            .collect();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn replacing_a_contribution_updates_the_graph() {
        let garden = Garden::new(&[
            ("Foam", "- [[Obsidian]]\n- [[Obsidian]] again\n"),
            ("Obsidian", "- An editor\n"),
        ]);
        let mut index = garden.index();
        index.replace(garden.contribution("Foam", "- [[Obsidian]] again\n"));
        assert!(
            garden
                .graph_index(&index)
                .graph(GraphSettings::default())
                .has_edge("Foam", "Obsidian"),
            "another block still links Obsidian"
        );
        index.replace(garden.contribution("Foam", "- nothing\n"));
        assert!(
            !garden
                .graph_index(&index)
                .graph(GraphSettings::default())
                .has_edge("Foam", "Obsidian")
        );
    }

    #[test]
    fn namespace_parent_names_the_parent_with_or_without_a_file() {
        let garden = Garden::new(&[
            ("Notes", "- [[Names/Nested]] and [[tmux/Config]]\n"),
            ("tmux/Config", "- Settings\n"),
        ]);
        let index = garden.index();
        let graph = garden.graph_index(&index);
        assert_eq!(graph.namespace_parent("names/nested"), Some("Names"));
        assert_eq!(graph.namespace_parent("TMUX/config"), Some("tmux"));
        assert_eq!(graph.namespace_parent("Names"), None);
        assert_eq!(graph.namespace_parent("Missing/Page"), None);
    }

    #[test]
    fn a_namespace_parent_keeps_the_space_before_its_slash() {
        // Logseq names the parent of `MacOS /Applications Folder` `MacOS `, a page without
        // a file, and keeps it apart from `MacOS`.
        let garden = Garden::new(&[
            (
                "Notes",
                "- [[MacOS /Applications Folder]] and [[Linux /Bin]]\n",
            ),
            ("MacOS", "- A system\n"),
            ("Linux /Bin", "- Programs\n"),
        ]);
        let index = garden.index();
        let graph = garden.graph_index(&index);
        assert_eq!(
            graph.namespace_parent("MacOS /Applications Folder"),
            Some("MacOS ")
        );
        assert_eq!(graph.namespace_parent("Linux /Bin"), Some("Linux "));
        let graph = garden.graph(GraphSettings::everything(true));
        assert!(graph.has_edge("MacOS /Applications Folder", "MacOS "));
        assert!(!graph.has_edge("MacOS /Applications Folder", "MacOS"));
        assert!(!graph.has_edge("Notes", "MacOS"));
        assert_eq!(graph.node("MacOS ").unwrap().page, None);
        assert_eq!(graph.node("MacOS").unwrap().page.as_deref(), Some("MacOS"));
        assert!(graph.node("Linux").is_none());
        // `A` is a page Logseq creates in every graph; the parent `A ` is another page.
        let garden = Garden::new(&[("Notes", "- [[A / B / C]]\n")]);
        let index = garden.index();
        let graph = garden.graph_index(&index);
        assert_eq!(graph.namespace_parent("A / B / C"), Some("A / B "));
        assert_eq!(graph.namespace_parent("A / B "), Some("A "));
    }

    #[test]
    fn a_page_reference_keeps_the_spaces_inside_its_brackets() {
        // Logseq names `Release Notes ` and ` Notes/Setup`, pages without a file, and keeps
        // them apart from `Release Notes` and `Notes/Setup`.
        let garden = Garden::new(&[
            (
                "Plan",
                "- [[Release Notes ]] and [[ Notes/Setup]]\n- [[ ]] names nothing\n",
            ),
            ("Release Notes", "- What changed\n"),
            ("Notes/Setup", "- Steps\n"),
        ]);
        let graph = garden.graph(GraphSettings::everything(true));
        assert!(graph.has_edge("Plan", "Release Notes "));
        assert!(!graph.has_edge("Plan", "Release Notes"));
        assert!(graph.has_edge("Plan", " Notes/Setup"));
        assert!(!graph.has_edge("Plan", "Notes/Setup"));
        assert_eq!(graph.node("Release Notes ").unwrap().page, None);
        assert_eq!(graph.node(" Notes/Setup").unwrap().page, None);
        assert!(graph.has_edge(" Notes/Setup", " Notes"));
        assert!(graph.nodes.iter().all(|node| !node.title.trim().is_empty()));
    }

    #[test]
    fn computing_the_graph_leaves_garden_files_unchanged() {
        let directory = std::env::temp_dir().join(format!("lsg-graph-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("pages")).unwrap();
        let file = directory.join("pages/Foam.md");
        std::fs::write(&file, "- [[Obsidian]]\n").unwrap();
        let before = std::fs::read(&file).unwrap();
        let garden = crate::garden::load_garden(&directory).unwrap();
        let source = &garden.documents[0];
        let contribution = Contribution::from_document("Foam".to_owned(), source, &|_| None, None);
        let index = ReferenceIndex::build(
            [PageEntry {
                id: "Foam".to_owned(),
                title: "Foam".into(),
                aliases: vec![],
            }],
            vec![contribution],
        );
        let graph = GraphIndex::new(&GraphSource {
            index: &index,
            journals: &BTreeSet::new(),
            hidden: &BTreeSet::new(),
            is_journal_name: &|_| false,
        })
        .graph(GraphSettings::default());
        assert!(graph.has_edge("Foam", "Obsidian"));
        assert_eq!(std::fs::read(&file).unwrap(), before);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
