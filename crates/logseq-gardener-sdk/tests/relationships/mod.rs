//! The page and block relationships the SDK derives from a garden, in the shape the
//! Logseq OG comparisons record.
//!
//! `og_relationships.rs` compares the page relationships with the ones Logseq recorded for
//! the fixture notes. `logseq_parser.rs` writes everything for a whole graph, which
//! `mise run docs:parser-check` compares with what Logseq's own parser builds.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use logseq_gardener_sdk::ast::Block;
use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument, load_garden};
use logseq_gardener_sdk::garden_settings::GardenSettings;
use logseq_gardener_sdk::graph::{GraphIndex, GraphSettings, GraphSource};
use logseq_gardener_sdk::journals::{JournalFormats, journal_identity, parse_journal_reference};
use logseq_gardener_sdk::outline::{BlockId, bullet_properties};
use logseq_gardener_sdk::page_lookup::{MatchedBy, PageCatalog};
use logseq_gardener_sdk::page_names::{lookup_key, page_alias_spellings, page_aliases};
use logseq_gardener_sdk::page_titles::{FilenameFormat, page_title};
use logseq_gardener_sdk::references::{Contribution, PageEntry, ReferenceIndex};
use logseq_gardener_sdk::transclusion::page_targets;
use serde::{Deserialize, Serialize};

/// A page that has a file.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct StoredPage {
    pub name: String,
    pub title: String,
    pub file: String,
    pub aliases: Vec<String>,
    pub namespace: Option<String>,
    pub journal_day: Option<String>,
}

/// An alias name that no page has as its title, with the files that declare it.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct AliasName {
    pub name: String,
    pub title: String,
    pub files: Vec<String>,
}

/// A page without a file that the graph names.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FilelessPage {
    pub name: String,
    pub title: String,
    pub namespace: Option<String>,
}

/// One file's page properties, block identifiers, block references and blocks.
#[derive(Debug, Serialize)]
pub struct Document {
    pub file: String,
    /// Names of the leading page properties, lower-cased.
    pub properties: Vec<String>,
    /// UUIDs that the file's blocks carry in `id::`, lower-cased.
    pub block_ids: Vec<String>,
    /// UUIDs that the file names in block references and embeds, lower-cased.
    pub block_refs: Vec<String>,
    pub blocks: Vec<DocumentBlock>,
}

/// A bullet, or the page's leading content before the first bullet.
#[derive(Debug, Serialize)]
pub struct DocumentBlock {
    /// 1-based line of the bullet; 1 for the leading content.
    pub line: usize,
    /// Pages the block names, with their namespace parents, as lookup keys.
    pub page_refs: Vec<String>,
    /// The block's properties: lower-cased name and trimmed value.
    pub properties: BTreeMap<String, String>,
}

/// Everything the SDK derives for one filename format.
#[derive(Debug, Serialize)]
pub struct Derived {
    pub pages: Vec<StoredPage>,
    pub aliases: Vec<AliasName>,
    pub fileless: Vec<FilelessPage>,
    pub documents: Vec<Document>,
}

fn path(document: &GardenDocument) -> String {
    document.relative_path.to_string_lossy().replace('\\', "/")
}

fn sorted(names: impl IntoIterator<Item = String>) -> Vec<String> {
    names
        .into_iter()
        .filter(|name| !name.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Derive the relationships of the garden at `root`, read with its `logseq/config.edn`.
pub fn derive(root: &Path, format: FilenameFormat) -> Derived {
    let garden = load_garden(root).unwrap();
    let (catalog, diagnostics) = PageCatalog::load(root, format).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let formats = JournalFormats::from_settings(&GardenSettings::read(root).0);

    let mut titles = BTreeMap::new();
    let mut journals = BTreeSet::new();
    let mut days = BTreeMap::new();
    for document in &garden.documents {
        let title = if document.kind == DocumentKind::Journal {
            let identity = journal_identity(document, &formats).unwrap();
            journals.insert(path(document));
            days.insert(
                path(document),
                format!(
                    "{:04}-{:02}-{:02}",
                    identity.date.year(),
                    identity.date.month(),
                    identity.date.day()
                ),
            );
            identity.title
        } else {
            page_title(document, format).unwrap()
        };
        titles.insert(path(document), title);
    }

    // A written name names a page when exactly one page answers to it, as published links do.
    let resolve = |name: &str| match catalog.find(name).as_slice() {
        [page] => Some(page.relative_path.to_string_lossy().replace('\\', "/")),
        _ => None,
    };
    let index = ReferenceIndex::build(
        garden.documents.iter().map(|document| PageEntry {
            id: path(document),
            title: titles[&path(document)].clone(),
            aliases: page_alias_spellings(document, &titles[&path(document)]),
        }),
        garden
            .documents
            .iter()
            .map(|document| Contribution::from_document(path(document), document, &resolve, None))
            .collect(),
    );
    let graph_index = GraphIndex::new(&GraphSource {
        index: &index,
        journals: &journals,
        hidden: &BTreeSet::new(),
        is_journal_name: &|name| parse_journal_reference(name, &formats).is_some(),
    });
    let namespace = |title: &str| graph_index.namespace_parent(title).map(lookup_key);

    let mut pages: Vec<StoredPage> = garden
        .documents
        .iter()
        .map(|document| {
            let file = path(document);
            let title = titles[&file].clone();
            StoredPage {
                name: lookup_key(&title),
                aliases: page_aliases(document, &title),
                namespace: namespace(&title),
                journal_day: days.get(&file).cloned(),
                title,
                file,
            }
        })
        .collect();
    pages.sort_by(|a, b| a.file.cmp(&b.file));

    // Each alias name that is no page's title finds the files that declare it. The first
    // file to declare a name, in path order, gives its spelling.
    let mut spellings = BTreeMap::new();
    for document in &garden.documents {
        for alias in page_alias_spellings(document, &titles[&path(document)]) {
            spellings.entry(lookup_key(&alias)).or_insert(alias);
        }
    }
    let aliases = spellings
        .into_iter()
        .filter_map(|(name, title)| {
            let found = catalog.find(&title);
            found
                .iter()
                .all(|page| page.matched_by == MatchedBy::Alias)
                .then(|| AliasName {
                    name,
                    title,
                    files: found
                        .iter()
                        .map(|page| page.relative_path.to_string_lossy().replace('\\', "/"))
                        .collect(),
                })
        })
        .collect();

    // Names without a file: graph nodes without a page that no catalog entry answers to.
    // An ambiguous alias is such a node too; the alias comparison covers it.
    let graph = graph_index.graph(GraphSettings::everything(true));
    let mut fileless: Vec<FilelessPage> = graph
        .nodes
        .iter()
        .filter(|node| node.page.is_none() && catalog.find(&node.title).is_empty())
        .map(|node| FilelessPage {
            name: lookup_key(&node.title),
            title: node.title.clone(),
            namespace: namespace(&node.title),
        })
        .collect();
    fileless.sort_by(|a, b| a.name.cmp(&b.name));

    let documents = garden.documents.iter().map(document).collect();
    Derived {
        pages,
        aliases,
        fileless,
        documents,
    }
}

/// A file's blocks as the reference index reads them, with every written name kept.
fn document(document: &GardenDocument) -> Document {
    let ast = &document.parsed.blocks;
    let leading = match ast.first() {
        Some(Block::Properties { props, .. }) => props
            .iter()
            .map(|prop| (prop.0.trim(), prop.1.trim()))
            .collect(),
        _ => Vec::new(),
    };
    let contribution = Contribution::from_document(
        path(document),
        document,
        &|name| Some(lookup_key(name)),
        None,
    );
    let blocks = contribution
        .blocks
        .iter()
        .map(|block| {
            // Only a bullet has an anchor; the other unit is the content before the first bullet.
            let properties: Vec<(&str, &str)> = match block.anchor {
                Some(_) => bullet_properties(ast, block.start).collect(),
                None => leading.clone(),
            };
            DocumentBlock {
                line: block
                    .anchor
                    .map_or(1, |byte| document.source[..byte].matches('\n').count() + 1),
                // An embed's block UUID is a block reference; the graph leaves it out too.
                page_refs: sorted(
                    block
                        .refs
                        .iter()
                        .filter(|name| !matches!(BlockId::parse(name), Some(BlockId::Uuid(_))))
                        .cloned(),
                ),
                properties: properties
                    .into_iter()
                    .map(|(name, value)| (name.to_lowercase(), value.to_owned()))
                    .collect(),
            }
        })
        .collect();
    Document {
        file: path(document),
        properties: sorted(leading.iter().map(|(name, _)| name.to_lowercase())),
        block_ids: sorted(page_targets(ast).into_iter().map(|(id, _, _)| id)),
        block_refs: sorted(
            document
                .parsed
                .refs
                .block
                .iter()
                .map(|id| id.trim().to_lowercase()),
        ),
        blocks,
    }
}
