//! Linked references, filters, tagged pages and unlinked mentions, answered from one
//! index that the CLI, the language server and the publisher share.
//!
//! Each document supplies one [`Contribution`]: its referring blocks, its `tags::` and
//! its `filters::`. [`ReferenceIndex`] combines them with the page names, expands a
//! reference to `A/B/C` to `A/B` and `A` as Logseq's parser does, and finds every
//! page's unlinked mentions in one pass over the block text.
//!
//! ```
//! use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
//! use logseq_gardener_sdk::references::{Contribution, PageEntry, ReferenceIndex};
//!
//! let document = |source: &str| GardenDocument {
//!     relative_path: "pages/notes.md".into(),
//!     kind: DocumentKind::Page,
//!     source: source.to_owned(),
//!     parsed: lsdoc::parse_to_projection(source),
//! };
//! let notes = document("- Set up [[mise/Config]]\n- mise runs tasks\n");
//! let resolve = |name: &str| match name.to_lowercase().as_str() {
//!     "mise" => Some("mise"),
//!     "mise/config" => Some("mise/Config"),
//!     _ => None,
//! };
//! let index = ReferenceIndex::build(
//!     [
//!         PageEntry { id: "mise", title: "mise".into(), aliases: vec![] },
//!         PageEntry { id: "mise/Config", title: "mise/Config".into(), aliases: vec![] },
//!         PageEntry { id: "notes", title: "notes".into(), aliases: vec![] },
//!     ],
//!     vec![Contribution::from_document("notes", &notes, &resolve, None)],
//! );
//! // The namespace child's reference also counts for its parent.
//! assert_eq!(index.linked(&"mise").len(), 1);
//! assert_eq!(index.linked(&"mise/Config").len(), 1);
//! // The second block names mise as plain text; `[[mise/Config]]` starts with `[[`.
//! assert_eq!(index.unlinked(&"mise").len(), 1);
//! ```
use std::collections::{BTreeMap, BTreeSet};

use aho_corasick::{AhoCorasick, MatchKind};
use unicode_normalization::UnicodeNormalization;

use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::page_names::{
    is_linkable_property, linkable_property_names, namespace_parent, namespace_parent_key,
};

/// A block that can refer to pages: a bullet with its own lines, or the page's leading
/// properties.
#[derive(Clone, Debug, PartialEq)]
pub struct RefBlock<K> {
    /// The unit's first AST block in its document.
    pub start: usize,
    /// End (exclusive) of the unit's AST blocks and its descendants'.
    pub subtree_end: usize,
    /// Outline level; leading content before the first bullet has level 0.
    pub level: u32,
    /// Byte offset of the bullet in the source; `None` for content before the first bullet.
    pub anchor: Option<usize>,
    /// Whether the unit is the page's leading properties.
    pub properties: bool,
    /// The nearest enclosing bullet, as an index into the same contribution.
    pub parent: Option<usize>,
    /// Pages the block names itself, before namespace expansion.
    pub refs: BTreeSet<K>,
}

/// Included and excluded pages for linked references.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter<K> {
    pub includes: BTreeSet<K>,
    pub excludes: BTreeSet<K>,
}

impl<K> Default for Filter<K> {
    fn default() -> Self {
        Self {
            includes: BTreeSet::new(),
            excludes: BTreeSet::new(),
        }
    }
}

impl<K: Ord> Filter<K> {
    pub fn is_empty(&self) -> bool {
        self.includes.is_empty() && self.excludes.is_empty()
    }

    /// Whether a block with these path references passes the filter.
    pub fn matches(&self, path: &BTreeSet<K>) -> bool {
        self.includes.iter().all(|page| path.contains(page))
            && !self.excludes.iter().any(|page| path.contains(page))
    }
}

/// What one document adds to the index. Replacing it replaces only that document's answers.
#[derive(Clone, Debug, PartialEq)]
pub struct Contribution<K> {
    pub page: K,
    pub blocks: Vec<RefBlock<K>>,
    /// Pages named in the leading `tags::`.
    pub tags: BTreeSet<K>,
    /// The starting filter from the leading `filters::`.
    pub filters: Filter<K>,
    /// Each block's own text, lower-cased, without logbooks or code.
    text: Vec<String>,
    /// Each block's references as the garden graph reads them, in block order.
    graph: Vec<GraphRefs<K>>,
    /// Names in the leading `tags::` that name no known page.
    unresolved_tags: BTreeSet<String>,
    /// The leading properties set `exclude-from-graph-view:: true`.
    exclude_from_graph: bool,
}

/// One block's references as the garden graph reads them. Only [`crate::graph`] uses
/// these; linked references and the other answers read [`RefBlock::refs`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphRefs<K> {
    /// Known pages the block names itself, without the namespace parents that
    /// [`RefBlock::refs`] adds.
    pub direct: BTreeSet<K>,
    /// Names the block writes that name no known page, as written.
    pub unresolved: BTreeSet<String>,
    /// The block's property names, lower-cased, such as `prev` for `prev::`.
    pub property_names: BTreeSet<String>,
}

/// A page the index knows by name: a published page or a generated namespace page.
#[derive(Clone, Debug)]
pub struct PageEntry<K> {
    pub id: K,
    pub title: String,
    pub aliases: Vec<String>,
}

/// A block in the index: its document's position and its position in that contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId {
    pub document: usize,
    pub block: usize,
}

impl<K: Ord + Clone> Contribution<K> {
    /// Collect a document's referring blocks. `resolve` maps an authored page name, or a
    /// namespace parent of one, to its page, and returns `None` for names that are missing
    /// or ambiguous. A namespace parent can end in a space, such as `MacOS ` for
    /// `MacOS /Applications Folder`; no garden page answers to it, as in Logseq
    /// ([`crate::page_names::ends_in_space`]).
    /// `properties` replaces the references of the leading properties, for callers that
    /// show only some property rows.
    pub fn from_document(
        page: K,
        document: &GardenDocument,
        resolve: &dyn Fn(&str) -> Option<K>,
        properties: Option<&BTreeSet<K>>,
    ) -> Self {
        let blocks = &document.parsed.blocks;
        let bullets: Vec<_> = blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| match block {
                Block::Bullet {
                    level,
                    span: Some(span),
                    ..
                } => Some((index, *level, span.0)),
                _ => None,
            })
            .collect();
        let mut subtree_ends = vec![blocks.len(); bullets.len()];
        let mut stack: Vec<usize> = Vec::new();
        for (position, &(index, level, _)) in bullets.iter().enumerate() {
            while stack.last().is_some_and(|&p| bullets[p].1 >= level) {
                subtree_ends[stack.pop().unwrap()] = index;
            }
            stack.push(position);
        }
        // (start, end, subtree_end, level, byte, end_byte, properties)
        let mut units = Vec::new();
        if let Some(&(first, _, byte)) = bullets.first() {
            if first > 0 {
                let properties = leading_properties(&blocks[..first]);
                units.push((0, first, first, 0, 0, byte, properties));
            }
            for (position, &(start, level, byte)) in bullets.iter().enumerate() {
                let (end, end_byte) = bullets
                    .get(position + 1)
                    .map(|(i, _, b)| (*i, *b))
                    .unwrap_or((blocks.len(), document.source.len()));
                units.push((
                    start,
                    end,
                    subtree_ends[position],
                    level,
                    byte,
                    end_byte,
                    false,
                ));
            }
        } else if !blocks.is_empty() {
            let properties = leading_properties(blocks);
            units.push((
                0,
                blocks.len(),
                blocks.len(),
                0,
                0,
                document.source.len(),
                properties,
            ));
        }
        let mut result = Self {
            page,
            blocks: Vec::new(),
            tags: BTreeSet::new(),
            filters: Filter::default(),
            text: Vec::new(),
            graph: Vec::new(),
            unresolved_tags: BTreeSet::new(),
            exclude_from_graph: false,
        };
        let mut parents: Vec<(u32, usize)> = Vec::new();
        for (start, end, subtree_end, level, byte, end_byte, is_properties) in units {
            let source = document.source.get(byte..end_byte).unwrap_or_default();
            // Page properties sit before the first bullet, even when other text follows them.
            let leading = level == 0;
            // Page names inside an advanced query are part of the query, not references.
            let mut names = lsdoc::refs(&without_query_blocks(source), "markdown").page;
            // Logseq's parser also gives a block a reference to its task marker's page and its
            // priority's page: `TODO [#A] write` names `TODO` and `A`.
            if let Some(Block::Bullet {
                marker, priority, ..
            }) = blocks.get(start)
            {
                names.extend(marker.iter().chain(priority).cloned());
            }
            let mut property_names = BTreeSet::new();
            for block in &blocks[start..end] {
                let Block::Properties { props, .. } = block else {
                    continue;
                };
                for prop in props {
                    property_names.insert(prop.0.trim().to_lowercase());
                    if leading
                        && prop.0.eq_ignore_ascii_case("exclude-from-graph-view")
                        && prop.1.trim().eq_ignore_ascii_case("true")
                    {
                        result.exclude_from_graph = true;
                    }
                    // Values name pages; property names are storage details. The block's text
                    // already holds each value's references, read as Logseq reads a property
                    // value: a nested reference `[[Notes on [[Mldoc]]]]` names its outer page
                    // only. Logseq also reads `alias::`, `aliases::` and `tags::` as lists of
                    // page names, bare comma-separated names included.
                    let values = if is_linkable_property(&prop.0) {
                        linkable_property_names(&prop.1)
                    } else {
                        Vec::new()
                    };
                    if prop.0.eq_ignore_ascii_case("tags") && leading {
                        // Like an alias, a value with no names, such as a quoted one, is one
                        // literal tag, though the block does not refer to it.
                        let literal = prop.1.trim();
                        let tags = if values.is_empty() && !literal.is_empty() {
                            vec![literal.to_owned()]
                        } else {
                            values.clone()
                        };
                        for tag in &tags {
                            match resolve(tag) {
                                Some(page) => {
                                    result.tags.insert(page);
                                }
                                None => {
                                    result.unresolved_tags.insert(tag.clone());
                                }
                            }
                        }
                    }
                    names.extend(values);
                    if prop.0.eq_ignore_ascii_case("filters") && leading {
                        for (name, include) in parse_filters(&prop.1) {
                            if let Some(id) = resolve(&name) {
                                if include {
                                    result.filters.includes.insert(id);
                                } else {
                                    result.filters.excludes.insert(id);
                                }
                            }
                        }
                    }
                }
            }
            // A written name names its page without the spaces around it.
            for name in &mut names {
                let trimmed = name.trim();
                if trimmed.len() != name.len() {
                    *name = trimmed.to_owned();
                }
            }
            // Logseq's parser also names each namespace parent, even when the child page
            // itself does not exist: `[[mise/ls]]` refers to `mise`. A parent keeps a space
            // before its slash, so `[[MacOS /Applications Folder]]` refers to `MacOS `, and
            // `resolve` finds no garden page for it.
            let namespaces: Vec<String> = names
                .iter()
                .flat_map(|name| {
                    let mut parents = Vec::new();
                    let mut rest = name.as_str();
                    while let Some(parent) = namespace_parent(rest) {
                        parents.push(parent.to_owned());
                        rest = parent;
                    }
                    parents
                })
                .collect();
            let mut graph = GraphRefs {
                direct: BTreeSet::new(),
                unresolved: BTreeSet::new(),
                property_names,
            };
            for name in &names {
                match resolve(name) {
                    Some(page) => {
                        graph.direct.insert(page);
                    }
                    None => {
                        if !name.is_empty() {
                            graph.unresolved.insert(name.clone());
                        }
                    }
                }
            }
            result.graph.push(graph);
            let refs = match properties {
                Some(targets) if is_properties => targets.clone(),
                _ => names.iter().filter_map(|name| resolve(name)).collect(),
            };
            let refs = refs
                .into_iter()
                .chain(namespaces.iter().filter_map(|name| resolve(name)))
                .collect();
            while parents
                .last()
                .is_some_and(|(previous, _)| *previous >= level)
            {
                parents.pop();
            }
            let bullet = matches!(blocks.get(start), Some(Block::Bullet { .. }));
            let index = result.blocks.len();
            result.blocks.push(RefBlock {
                start,
                subtree_end,
                level,
                anchor: (!is_properties && bullet).then_some(byte),
                properties: is_properties,
                parent: parents.last().map(|(_, index)| *index),
                refs,
            });
            result.text.push(mention_text(source));
            if !is_properties && bullet {
                parents.push((level, index));
            }
        }
        result
    }
}

impl<K> Contribution<K> {
    /// Each block's references as the garden graph reads them, in block order.
    pub fn graph_refs(&self) -> &[GraphRefs<K>] {
        &self.graph
    }

    /// Names in the leading `tags::` that name no known page.
    pub fn unresolved_tags(&self) -> &BTreeSet<String> {
        &self.unresolved_tags
    }

    /// Whether the leading properties set `exclude-from-graph-view:: true`.
    pub fn excluded_from_graph(&self) -> bool {
        self.exclude_from_graph
    }
}

/// The text with each `#+BEGIN_QUERY` … `#+END_QUERY` block blanked out, as Logseq's
/// parser skips query blocks when it collects a block's references.
fn without_query_blocks(source: &str) -> std::borrow::Cow<'_, str> {
    let upper = source.to_ascii_uppercase();
    if !upper.contains("#+BEGIN_QUERY") {
        return source.into();
    }
    let mut text = source.as_bytes().to_vec();
    let mut from = 0;
    while let Some(start) = upper[from..].find("#+BEGIN_QUERY").map(|i| i + from) {
        let end = upper[start..]
            .find("#+END_QUERY")
            .map_or(source.len(), |i| start + i + "#+END_QUERY".len());
        for byte in &mut text[start..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
        from = end;
    }
    String::from_utf8_lossy(&text).into_owned().into()
}

/// Leading page properties, followed by nothing but blank lines.
fn leading_properties(blocks: &[Block]) -> bool {
    matches!(blocks.first(), Some(Block::Properties { .. }))
        && blocks.iter().all(|block| match block {
            Block::Properties { .. } => true,
            Block::Paragraph { inline, .. } => inline
                .iter()
                .all(|node| matches!(node, crate::ast::Inline::Break { .. })),
            _ => false,
        })
}

/// Read Logseq's `filters::` value, `{"page" true, "other" false}`.
fn parse_filters(value: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        let name = &after[..close];
        let tail = after[close + 1..].trim_start_matches([' ', ',', '\t']);
        if tail.starts_with("true") {
            result.push((name.to_owned(), true));
        } else if tail.starts_with("false") {
            result.push((name.to_owned(), false));
        }
        rest = &after[close + 1..];
    }
    result
}

fn normalize(text: &str) -> String {
    text.to_lowercase().nfc().collect()
}

/// A block's text as unlinked mentions see it: lower-cased, with logbook drawers removed,
/// as Logseq searches block content. Code stays: Logseq matches names inside it too.
fn mention_text(source: &str) -> String {
    let mut text = String::new();
    let mut logbook = false;
    for line in source.lines() {
        let trimmed = line.trim_start().trim_start_matches("- ").trim_start();
        if trimmed.eq_ignore_ascii_case(":LOGBOOK:") {
            logbook = true;
            continue;
        }
        if logbook {
            logbook = !trimmed.eq_ignore_ascii_case(":END:");
            continue;
        }
        text.push_str(line);
        text.push('\n');
    }
    normalize(&text)
}

/// Logseq's boundary rule: the name is not preceded by a letter, digit, `#` or `[[`, and
/// not followed by a letter or digit.
fn is_mention(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(|c| c.is_alphanumeric() || c == '#')
        && !text[..start].ends_with("[[")
        && !after.is_some_and(char::is_alphanumeric)
}

/// Linked references, filters, tagged pages and unlinked mentions over a set of documents.
pub struct ReferenceIndex<K> {
    titles: BTreeMap<K, String>,
    /// Each page's namespace ancestors, nearest first.
    parents: BTreeMap<K, Vec<K>>,
    matcher: Option<(AhoCorasick, Vec<Vec<K>>)>,
    contributions: Vec<Contribution<K>>,
    positions: BTreeMap<K, usize>,
    /// Per document and block: its references with namespace ancestors.
    expanded: Vec<Vec<BTreeSet<K>>>,
    /// Per document: (block, page it mentions).
    mentions: Vec<Vec<(usize, K)>>,
    linked: BTreeMap<K, Vec<BlockId>>,
    unlinked: BTreeMap<K, Vec<BlockId>>,
    tagged: BTreeMap<K, Vec<K>>,
}

impl<K: Ord + Clone> ReferenceIndex<K> {
    pub fn build(
        pages: impl IntoIterator<Item = PageEntry<K>>,
        contributions: Vec<Contribution<K>>,
    ) -> Self {
        let mut titles = BTreeMap::new();
        let mut by_title = BTreeMap::new();
        let mut patterns: BTreeMap<String, Vec<K>> = BTreeMap::new();
        for page in pages {
            by_title.insert(namespace_parent_key(&page.title), page.id.clone());
            for name in std::iter::once(&page.title).chain(&page.aliases) {
                let name = normalize(name.trim());
                if !name.is_empty() {
                    let ids = patterns.entry(name).or_default();
                    if !ids.contains(&page.id) {
                        ids.push(page.id.clone());
                    }
                }
            }
            titles.insert(page.id, page.title);
        }
        let parents = titles
            .iter()
            .map(|(id, title)| {
                let mut ancestors = Vec::new();
                let mut name = title.as_str();
                while let Some(parent) = namespace_parent(name) {
                    if let Some(parent_id) = by_title.get(&namespace_parent_key(parent)) {
                        ancestors.push(K::clone(parent_id));
                    }
                    name = parent;
                }
                (id.clone(), ancestors)
            })
            .collect();
        let (names, targets): (Vec<_>, Vec<_>) = patterns.into_iter().unzip();
        let matcher = AhoCorasick::builder()
            .match_kind(MatchKind::Standard)
            .build(&names)
            .ok()
            .map(|automaton| (automaton, targets));
        let mut index = Self {
            titles,
            parents,
            matcher,
            positions: BTreeMap::new(),
            expanded: Vec::new(),
            mentions: Vec::new(),
            contributions: Vec::new(),
            linked: BTreeMap::new(),
            unlinked: BTreeMap::new(),
            tagged: BTreeMap::new(),
        };
        for contribution in contributions {
            index
                .positions
                .insert(contribution.page.clone(), index.contributions.len());
            index.expanded.push(index.expand(&contribution));
            index.mentions.push(index.find_mentions(&contribution));
            index.contributions.push(contribution);
        }
        index.aggregate();
        index
    }

    /// Replace one document's contribution, or add it when the page is new to the index.
    pub fn replace(&mut self, contribution: Contribution<K>) {
        let expanded = self.expand(&contribution);
        let mentions = self.find_mentions(&contribution);
        match self.positions.get(&contribution.page) {
            Some(&position) => {
                self.expanded[position] = expanded;
                self.mentions[position] = mentions;
                self.contributions[position] = contribution;
            }
            None => {
                self.positions
                    .insert(contribution.page.clone(), self.contributions.len());
                self.expanded.push(expanded);
                self.mentions.push(mentions);
                self.contributions.push(contribution);
            }
        }
        self.aggregate();
    }

    /// Bullets already name their namespace parents, from the names their author wrote.
    /// Leading properties may carry resolved pages instead, so their parents come from the
    /// page titles. An alias is a name of its own: `[[mise File Tasks]]` names no namespace.
    fn expand(&self, contribution: &Contribution<K>) -> Vec<BTreeSet<K>> {
        contribution
            .blocks
            .iter()
            .map(|block| {
                let mut refs = block.refs.clone();
                if block.properties {
                    for page in block.refs.iter().filter(|page| *page != &contribution.page) {
                        refs.extend(self.parents.get(page).into_iter().flatten().cloned());
                    }
                }
                refs
            })
            .collect()
    }

    fn find_mentions(&self, contribution: &Contribution<K>) -> Vec<(usize, K)> {
        let Some((automaton, targets)) = &self.matcher else {
            return Vec::new();
        };
        let mut result = BTreeSet::new();
        for (block, text) in contribution.text.iter().enumerate() {
            for found in automaton.find_overlapping_iter(text.as_str()) {
                if !is_mention(text, found.start(), found.end()) {
                    continue;
                }
                for page in &targets[found.pattern().as_usize()] {
                    if page != &contribution.page {
                        result.insert((block, page.clone()));
                    }
                }
            }
        }
        result.into_iter().collect()
    }

    fn aggregate(&mut self) {
        self.linked.clear();
        self.unlinked.clear();
        self.tagged.clear();
        for (document, contribution) in self.contributions.iter().enumerate() {
            for (block, refs) in self.expanded[document].iter().enumerate() {
                for page in refs.iter().filter(|page| *page != &contribution.page) {
                    self.linked
                        .entry(page.clone())
                        .or_default()
                        .push(BlockId { document, block });
                }
            }
            for (block, page) in &self.mentions[document] {
                self.unlinked
                    .entry(page.clone())
                    .or_default()
                    .push(BlockId {
                        document,
                        block: *block,
                    });
            }
            for tag in contribution
                .tags
                .iter()
                .filter(|tag| *tag != &contribution.page)
            {
                self.tagged
                    .entry(tag.clone())
                    .or_default()
                    .push(contribution.page.clone());
            }
        }
        for pages in self.tagged.values_mut() {
            pages.sort_by_cached_key(|page| {
                self.titles
                    .get(page)
                    .map(|title| title.to_lowercase())
                    .unwrap_or_default()
            });
            pages.dedup();
        }
    }

    /// Blocks that refer to the page, an alias of it or a namespace child of it, outside
    /// the page itself, each once, in document order.
    pub fn linked(&self, page: &K) -> &[BlockId] {
        self.linked.get(page).map_or(&[], Vec::as_slice)
    }

    /// Blocks whose text mentions the page's title or an alias, outside the page itself.
    pub fn unlinked(&self, page: &K) -> &[BlockId] {
        self.unlinked.get(page).map_or(&[], Vec::as_slice)
    }

    /// Pages whose `tags::` name the page or an alias, by lower-cased title.
    pub fn tagged_with(&self, page: &K) -> &[K] {
        self.tagged.get(page).map_or(&[], Vec::as_slice)
    }

    /// The page's starting filter from its `filters::`, keeping only resolved names.
    pub fn starting_filter(&self, page: &K) -> Filter<K> {
        self.positions
            .get(page)
            .map(|&position| self.contributions[position].filters.clone())
            .unwrap_or_default()
    }

    /// The page's own blocks, in document order.
    pub fn blocks_of(&self, page: &K) -> Vec<BlockId> {
        self.positions
            .get(page)
            .map(|&document| {
                (0..self.contributions[document].blocks.len())
                    .map(|block| BlockId { document, block })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn block(&self, id: BlockId) -> &RefBlock<K> {
        &self.contributions[id.document].blocks[id.block]
    }

    /// The page that holds the block.
    pub fn page(&self, id: BlockId) -> &K {
        &self.contributions[id.document].page
    }

    /// The block's ancestors in the outline, nearest first.
    pub fn ancestors(&self, id: BlockId) -> Vec<BlockId> {
        let mut result = Vec::new();
        let mut current = self.block(id).parent;
        while let Some(block) = current {
            result.push(BlockId {
                document: id.document,
                block,
            });
            current = self.contributions[id.document].blocks[block].parent;
        }
        result
    }

    /// The pages of the block, its ancestors and its page, used to match filters.
    pub fn path_refs(&self, id: BlockId) -> BTreeSet<K> {
        let mut path = self.expanded[id.document][id.block].clone();
        for ancestor in self.ancestors(id) {
            path.extend(self.expanded[id.document][ancestor.block].iter().cloned());
        }
        path.insert(self.page(id).clone());
        path
    }

    /// Each descendant's pages, one entry per descendant that names a page, which filter
    /// options count with repeats as Logseq does.
    pub fn descendant_refs(&self, id: BlockId) -> Vec<K> {
        let blocks = &self.contributions[id.document].blocks;
        let end = blocks[id.block].subtree_end;
        let mut refs = Vec::new();
        if blocks[id.block].properties {
            return refs;
        }
        for (offset, block) in blocks[id.block + 1..].iter().enumerate() {
            if block.start >= end {
                break;
            }
            refs.extend(
                self.expanded[id.document][id.block + 1 + offset]
                    .iter()
                    .cloned(),
            );
        }
        refs
    }

    /// Filter options for a set of a page's linked references, with counts, most frequent
    /// first and ties by title, leaving out `exclude`. As in Logseq, each block whose parent
    /// is not itself among `blocks` adds its path references once and each descendant's
    /// references; a block inside another referring block adds nothing of its own.
    pub fn related(&self, blocks: &[BlockId], exclude: &K) -> Vec<(K, usize)> {
        let referring: BTreeSet<_> = blocks.iter().copied().collect();
        let mut counts: BTreeMap<K, usize> = BTreeMap::new();
        for &id in blocks {
            let parent = self.block(id).parent.map(|block| BlockId {
                document: id.document,
                block,
            });
            if parent.is_some_and(|parent| referring.contains(&parent)) {
                continue;
            }
            let pages = self
                .path_refs(id)
                .into_iter()
                .chain(self.descendant_refs(id));
            for page in pages.filter(|page| page != exclude) {
                *counts.entry(page).or_default() += 1;
            }
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by_cached_key(|(page, count)| {
            (
                std::cmp::Reverse(*count),
                self.titles
                    .get(page)
                    .map(|title| title.to_lowercase())
                    .unwrap_or_default(),
            )
        });
        result
    }

    /// Logseq's Backlinks count: blocks whose references include the page, counting
    /// namespace children as [`Self::linked`] does and also the page's own blocks.
    pub fn backlink_count(&self, page: &K) -> usize {
        let own = self.positions.get(page).map_or(0, |&document| {
            self.expanded[document]
                .iter()
                .filter(|refs| refs.contains(page))
                .count()
        });
        self.linked(page).len() + own
    }

    /// Number of distinct pages among the page's linked references.
    pub fn referring_pages(&self, page: &K) -> usize {
        self.linked(page)
            .iter()
            .map(|id| id.document)
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub fn title(&self, page: &K) -> Option<&str> {
        self.titles.get(page).map(String::as_str)
    }

    /// Every page the index knows by name, with its title.
    pub fn pages(&self) -> impl Iterator<Item = (&K, &str)> {
        self.titles
            .iter()
            .map(|(page, title)| (page, title.as_str()))
    }

    /// Each document's contribution, in the order the index holds them.
    pub fn contributions(&self) -> &[Contribution<K>] {
        &self.contributions
    }

    /// A document's blocks' references with namespace parents added, by position in
    /// [`Self::contributions`].
    pub fn expanded_refs(&self, document: usize) -> &[BTreeSet<K>] {
        &self.expanded[document]
    }

    /// The page's namespace parents that the index knows, nearest first.
    pub fn namespace_parents(&self, page: &K) -> &[K] {
        self.parents.get(page).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;
    use crate::page_names::{ends_in_space, lookup_key};

    fn document(source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: "pages/x.md".into(),
            kind: DocumentKind::Page,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    const PAGES: [&str; 8] = [
        "mise",
        "mise/Config",
        "mise/Config/mise.toml",
        "tmux",
        "Term",
        "Diataxis/How To",
        "Diataxis",
        "AWS",
    ];

    fn resolve(name: &str) -> Option<String> {
        let key = lookup_key(name);
        if key == "rtx" {
            return Some("mise".to_owned());
        }
        if key == "mise conf" {
            return Some("mise/Config".to_owned());
        }
        PAGES
            .iter()
            .find(|page| lookup_key(page) == key)
            .map(|page| (*page).to_owned())
    }

    fn entries(extra: &[&str]) -> Vec<PageEntry<String>> {
        PAGES
            .iter()
            .chain(extra)
            .map(|title| PageEntry {
                id: (*title).to_owned(),
                title: (*title).to_owned(),
                aliases: if *title == "mise" {
                    vec!["rtx".to_owned()]
                } else {
                    vec![]
                },
            })
            .collect()
    }

    fn contribution(page: &str, source: &str) -> Contribution<String> {
        Contribution::from_document(page.to_owned(), &document(source), &resolve, None)
    }

    fn index(sources: &[(&str, &str)]) -> ReferenceIndex<String> {
        let pages: Vec<&str> = sources.iter().map(|(page, _)| *page).collect();
        ReferenceIndex::build(
            entries(&pages),
            sources
                .iter()
                .map(|(page, source)| contribution(page, source))
                .collect(),
        )
    }

    #[test]
    fn advanced_query_text_names_no_page() {
        let index = index(&[(
            "notes",
            "- Queries\n\t- #+BEGIN_QUERY\n\t  {:title \"t\" :query [[tmux]]}\n\t  #+END_QUERY\n- {{query [[mise]]}}\n- [[AWS]] after\n",
        )]);
        assert!(index.linked(&"tmux".to_owned()).is_empty());
        assert!(index.linked(&"mise".to_owned()).is_empty());
        assert_eq!(index.linked(&"AWS".to_owned()).len(), 1);
    }

    #[test]
    fn backlink_count_adds_the_pages_own_blocks_and_namespace_children() {
        let index = index(&[
            ("notes", "- [[mise]]\n- [[mise/Config]]\n- [[tmux]]\n"),
            ("mise", "- self [[mise]]\n- [[mise/Config]]\n- plain text\n"),
        ]);
        let mise = "mise".to_owned();
        // Two linked blocks on notes, plus two of mise's own blocks.
        assert_eq!(index.linked(&mise).len(), 2);
        assert_eq!(index.backlink_count(&mise), 4);
        // A reference to mise/Config counts for mise/Config, on either page.
        assert_eq!(index.backlink_count(&"mise/Config".to_owned()), 2);
        assert_eq!(index.backlink_count(&"tmux".to_owned()), 1);
        assert_eq!(index.backlink_count(&"AWS".to_owned()), 0);
    }

    #[test]
    fn aliases_namespace_parents_and_repeats_count_once() {
        let index = index(&[
            (
                "notes",
                "- [[rtx]] and [[mise]] twice [[mise]]\n- [[mise/Config/mise.toml]]\n- [[tmux]]\n",
            ),
            ("mise", "- self [[mise]]\n- [[mise/Config]]\n"),
        ]);
        let mise = "mise".to_owned();
        assert_eq!(index.linked(&mise).len(), 2);
        assert_eq!(index.linked(&"mise/Config".to_owned()).len(), 2);
        assert_eq!(index.linked(&"mise/Config/mise.toml".to_owned()).len(), 1);
        // A namespace reference on the parent's own page stays off its section.
        assert!(
            index
                .linked(&mise)
                .iter()
                .all(|id| index.page(*id) != &mise)
        );
    }

    #[test]
    fn a_namespace_parent_keeps_the_space_before_its_slash() {
        // Logseq names the parent of `MacOS /Applications Folder` `MacOS `, which no page
        // file has, so neither the reference nor the child page counts for `MacOS`.
        let titles = ["MacOS", "MacOS /Applications Folder", "notes"];
        let resolve = |name: &str| {
            titles
                .iter()
                .find(|title| !ends_in_space(name) && lookup_key(title) == lookup_key(name))
                .map(|title| (*title).to_owned())
        };
        let sources = [
            ("notes", "- Apps live in [[MacOS /Applications Folder]]\n"),
            ("MacOS /Applications Folder", "- Apps\n"),
        ];
        let index = ReferenceIndex::build(
            titles.iter().map(|title| PageEntry {
                id: (*title).to_owned(),
                title: (*title).to_owned(),
                aliases: vec![],
            }),
            sources
                .iter()
                .map(|(page, source)| {
                    Contribution::from_document(
                        (*page).to_owned(),
                        &document(source),
                        &resolve,
                        None,
                    )
                })
                .collect(),
        );
        let child = "MacOS /Applications Folder".to_owned();
        assert_eq!(index.linked(&child).len(), 1);
        assert!(index.linked(&"MacOS".to_owned()).is_empty());
        assert_eq!(index.backlink_count(&"MacOS".to_owned()), 0);
    }

    #[test]
    fn a_missing_namespace_child_still_names_its_parents() {
        let index = index(&[(
            "notes",
            "- [[mise/ls]] lists tools\n- [[Diataxis/How To/Missing]]\n",
        )]);
        assert_eq!(index.linked(&"mise".to_owned()).len(), 1);
        assert_eq!(index.linked(&"Diataxis/How To".to_owned()).len(), 1);
        assert_eq!(index.linked(&"Diataxis".to_owned()).len(), 1);
    }

    #[test]
    fn an_alias_names_no_namespace() {
        let index = index(&[
            (
                "notes",
                "- [[rtx]] alias\n- [[mise conf]] names the child only\n",
            ),
            ("mise/Config", "alias:: [[mise conf]]\n\n- body\n"),
        ]);
        assert_eq!(index.linked(&"mise".to_owned()).len(), 1);
        assert_eq!(index.linked(&"mise/Config".to_owned()).len(), 1);
        // The page's own alias is a self reference, and adds no parent.
        assert!(
            index
                .blocks_of(&"mise/Config".to_owned())
                .iter()
                .all(|id| !index.linked(&"mise".to_owned()).contains(id))
        );
    }

    #[test]
    fn property_names_are_not_references_but_values_are() {
        let index = index(&[(
            "card",
            "tags:: Term\ncard-last-interval:: 3\n\n- AWS:: [[tmux]]\n",
        )]);
        assert!(index.linked(&"AWS".to_owned()).is_empty());
        assert_eq!(index.linked(&"tmux".to_owned()).len(), 1);
        assert_eq!(index.linked(&"Term".to_owned()).len(), 1);
        assert!(index.block(index.linked(&"Term".to_owned())[0]).properties);
    }

    fn unresolved(page: &Contribution<String>, block: usize) -> Vec<&str> {
        page.graph_refs()[block]
            .unresolved
            .iter()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn task_markers_and_priorities_name_their_pages() {
        let page = contribution(
            "notes",
            "- TODO write\n- [#A] TODO later\n- DONE [#B] [[tmux]]\n- todo lower case\n- TODOs plural\n",
        );
        assert_eq!(unresolved(&page, 0), ["TODO"]);
        // Logseq reads a marker only at the start of the block.
        assert_eq!(unresolved(&page, 1), ["A"]);
        assert_eq!(unresolved(&page, 2), ["B", "DONE"]);
        assert!(unresolved(&page, 3).is_empty());
        assert!(unresolved(&page, 4).is_empty());
        let index = index(&[("notes", "- DOING set up [[mise]]\n"), ("TODO", "- x\n")]);
        assert_eq!(index.linked(&"mise".to_owned()).len(), 1);
    }

    #[test]
    fn alias_and_tags_values_name_pages_as_in_logseq() {
        let page = contribution(
            "notes",
            "alias:: tmux, #[[AWS]]\n\n- body\n  alias:: Term, [[Outer [[mise]] Name]]\n- tags:: plain [[Diataxis]] text\n",
        );
        let graph = page.graph_refs();
        assert_eq!(
            graph[0].direct,
            BTreeSet::from(["AWS".to_owned(), "tmux".to_owned()])
        );
        // A nested reference in the value names its outer page only.
        assert_eq!(graph[1].direct, BTreeSet::from(["Term".to_owned()]));
        assert_eq!(unresolved(&page, 1), ["Outer [[mise]] Name"]);
        // Plain text around a reference names pages, split at commas.
        assert_eq!(graph[2].direct, BTreeSet::from(["Diataxis".to_owned()]));
        assert_eq!(unresolved(&page, 2), ["plain", "text"]);
        // A quoted value names nothing.
        let quoted = contribution("notes", "alias:: \"tmux\"\ntags:: \"Term\"\n\n- body\n");
        assert!(quoted.graph_refs()[0].direct.is_empty());
        assert!(quoted.graph_refs()[0].unresolved.is_empty());
        // As in Logseq, the quoted tags value is still the page's one literal tag.
        assert_eq!(
            quoted.unresolved_tags(),
            &BTreeSet::from(["\"Term\"".to_owned()])
        );
    }

    #[test]
    fn other_property_values_name_pages_as_in_logseq() {
        let page = contribution(
            "notes",
            "type:: [[Notes on [[Mldoc]]]]\n\n- body\n  source:: [[Outer [[Inner]] Name]], #[[Tag [[Deep]]]], [[Plain]]\n- In the text [[Body [[Nested]] Ref]]\n",
        );
        assert_eq!(unresolved(&page, 0), ["Notes on [[Mldoc]]"]);
        assert_eq!(
            unresolved(&page, 1),
            ["Outer [[Inner]] Name", "Plain", "Tag [[Deep]]"]
        );
        // In the block's text, as in Logseq, a nested reference names the inner page too.
        assert_eq!(unresolved(&page, 2), ["Body [[Nested]] Ref", "Nested"]);
        // Logseq reads a property value's page references, tags and nested references only,
        // so the label of a link to a file names no page there.
        let file_link = contribution(
            "notes",
            "source-md:: [Report.md](../assets/Report.md)\n\n- body\n",
        );
        assert!(file_link.graph_refs()[0].unresolved.is_empty());
    }

    #[test]
    fn graph_references_record_property_names_without_changing_linked_references() {
        let source = "tags:: Term, Unknown Tag\nexclude-from-graph-view:: true\n\n- AWS:: [[tmux]]\n- [[mise/Config]] [[Nowhere]]\n";
        let page = contribution("card", source);
        let graph = page.graph_refs();
        assert!(graph[0].property_names.contains("tags"));
        assert!(graph[0].property_names.contains("exclude-from-graph-view"));
        assert!(graph[1].property_names.contains("aws"));
        assert!(graph[0].direct.contains("Term"));
        assert_eq!(
            page.unresolved_tags().iter().collect::<Vec<_>>(),
            ["Unknown Tag"]
        );
        assert!(page.excluded_from_graph());
        // Direct references leave out the namespace parents that `refs` adds.
        assert!(graph[2].direct.contains("mise/Config") && !graph[2].direct.contains("mise"));
        assert!(page.blocks[2].refs.contains("mise"));
        assert!(graph[2].unresolved.contains("Nowhere"));
        let index = index(&[("card", source)]);
        assert!(index.linked(&"AWS".to_owned()).is_empty());
        assert_eq!(index.linked(&"tmux".to_owned()).len(), 1);
        assert_eq!(index.linked(&"mise".to_owned()).len(), 1);
    }

    #[test]
    fn path_refs_related_and_order() {
        let index = index(&[(
            "notes",
            "- [[AWS]] parent\n  - [[Diataxis/How To]] [[mise]]\n    - child [[tmux]]\n- [[mise]] alone\n",
        )]);
        let mise = "mise".to_owned();
        let linked = index.linked(&mise).to_vec();
        assert_eq!(linked.len(), 2);
        let path = index.path_refs(linked[0]);
        for page in ["AWS", "Diataxis/How To", "Diataxis", "notes", "mise"] {
            assert!(path.contains(page), "{page}");
        }
        assert!(!path.contains("tmux"));
        let related = index.related(&linked, &mise);
        assert_eq!(related[0], ("notes".to_owned(), 2));
        // Ties sort by lower-cased title.
        let tied: Vec<_> = related[1..].iter().map(|(page, _)| page.as_str()).collect();
        assert_eq!(tied, ["AWS", "Diataxis", "Diataxis/How To", "tmux"]);
        // A referring block inside another one adds nothing; descendants count with repeats.
        let nested = super::tests::index(&[(
            "notes",
            "- [[mise]] top\n  - [[mise]] [[AWS]] inner\n  - [[AWS]] child\n",
        )]);
        let linked = nested.linked(&mise).to_vec();
        assert_eq!(linked.len(), 2);
        let related = nested.related(&linked, &mise);
        assert_eq!(related[0], ("AWS".to_owned(), 2));
        assert_eq!(related[1], ("notes".to_owned(), 1));
    }

    #[test]
    fn starting_filters() {
        let index = index(&[
            (
                "mise",
                "filters:: {\"diataxis/how to\" true, \"aws\" false, \"unknown\" true}\n\n- body\n",
            ),
            ("Term", "filters:: {}\n\n- body\n"),
        ]);
        let filter = index.starting_filter(&"mise".to_owned());
        assert_eq!(
            filter.includes,
            BTreeSet::from(["Diataxis/How To".to_owned()])
        );
        assert_eq!(filter.excludes, BTreeSet::from(["AWS".to_owned()]));
        assert!(index.starting_filter(&"Term".to_owned()).is_empty());
        assert!(filter.matches(&BTreeSet::from(["Diataxis/How To".to_owned()])));
        assert!(!filter.matches(&BTreeSet::from([
            "Diataxis/How To".to_owned(),
            "AWS".to_owned()
        ])));
    }

    #[test]
    fn tagged_pages_resolve_aliases_and_sort_by_lower_case_title() {
        let index = index(&[
            ("zeta", "tags:: rtx, Term\n\n- body\n"),
            ("Alpha", "tags:: [[mise]]\n\n- body\n"),
            ("gamma", "tags:: rtx, [[Term]], other\n\n- body\n"),
            ("beta", "- tags:: mise\n"),
            ("delta", "tags:: [[Term]]\nsee-also [[mise]]\n- body\n"),
        ]);
        assert_eq!(
            index.tagged_with(&"mise".to_owned()),
            ["Alpha", "gamma", "zeta"]
        );
        assert_eq!(
            index.tagged_with(&"Term".to_owned()),
            ["delta", "gamma", "zeta"]
        );
        assert_eq!(
            linkable_property_names("a, [[b, c]], #d"),
            ["a", "b, c", "d"]
        );
    }

    #[test]
    fn unlinked_mentions_follow_logseqs_boundaries() {
        let index = index(&[(
            "notes",
            "- [[My/Tmux]] link text\n- #tmux tag\n- tmuxinator\n- [[tmux]]\n- uses TMUX daily\n- `tmux` code\n- ```sh\n  tmux\n  ```\n- :LOGBOOK:\n  tmux\n  :END:\n- 2tmux\n",
        )]);
        let found: Vec<usize> = index
            .unlinked(&"tmux".to_owned())
            .iter()
            .map(|id| id.block)
            .collect();
        // Code counts, as in Logseq; logbooks and the other boundaries do not.
        assert_eq!(found, [0, 4, 5, 6]);
    }

    #[test]
    fn replacing_one_contribution_equals_a_full_rebuild() {
        let before = [
            ("a", "- [[mise/Config]] and tmux\n"),
            ("b", "tags:: Term\n\n- [[AWS]]\n"),
        ];
        let after = [
            ("a", "- [[tmux]] and mise\n"),
            ("b", "tags:: Term\n\n- [[AWS]]\n"),
        ];
        let mut changed = index(&before);
        changed.replace(contribution("a", after[0].1));
        let rebuilt = index(&after);
        for page in PAGES.iter().map(|page| (*page).to_owned()) {
            assert_eq!(changed.linked(&page), rebuilt.linked(&page), "{page}");
            assert_eq!(changed.unlinked(&page), rebuilt.unlinked(&page), "{page}");
            assert_eq!(changed.tagged_with(&page), rebuilt.tagged_with(&page));
        }
    }
}
