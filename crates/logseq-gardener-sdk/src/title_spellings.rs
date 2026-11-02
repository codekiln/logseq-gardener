//! The spelling Logseq gives a page that refers to itself in another spelling.
//!
//! When Logseq's file parser reads a page's file, it merges the page with every page that
//! the file's blocks refer to, by lower-cased name, and the later references win
//! (`with-ref-pages` in `deps/graph-parser/src/logseq/graph_parser/extract.cljc`). So a file
//! `Solvespace.md` that holds `alias:: [[SolveSpace]]` gets the title `SolveSpace`. Each
//! reference takes the spelling of a page that already exists (`page-name->map` in
//! `deps/graph-parser/src/logseq/graph_parser/block.cljs`): one that Logseq creates in every
//! graph, such as `TODO`, or one that a file Logseq read earlier refers to. A page whose file
//! does not refer to the page keeps the title of its file name or `title::` property.
//!
//! Within a file, the last block that refers to the page gives the spelling. Within one
//! block, Logseq lists the names written in the text from the last to the first, then the
//! namespace parents, so a namespace parent wins, and otherwise the name written first.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::Block;
use crate::block_ids::parse_order;
use crate::garden::{DocumentKind, GardenDocument};
use crate::page_names::{
    built_in_page_title, has_outer_space, is_linkable_property, linkable_property_names,
    lookup_key, namespace_parent, reference_key,
};
use crate::page_titles::{FilenameFormat, page_title};
use crate::references::Contribution;

/// Record in each page's [`GardenDocument::title_spellings`] the spelling Logseq gives its
/// title when the page refers to itself in another spelling, for both filename formats.
///
/// Reading a file's references costs far more than listing the names it writes, which the
/// parse that loading did already holds. So the pass reads the references of a file only
/// when it writes a page's title, and only for a title that some file writes in another
/// spelling or that Logseq spells as one of its own pages.
pub(crate) fn spell_titles_as_logseq(documents: &mut [GardenDocument]) {
    // Each page's titles under the two filename formats, by lookup key.
    let titles: Vec<BTreeMap<String, String>> = documents
        .iter()
        .map(|document| {
            if document.kind != DocumentKind::Page {
                return BTreeMap::new();
            }
            [FilenameFormat::Legacy, FilenameFormat::TripleLowbar]
                .into_iter()
                .filter_map(|format| page_title(document, format).ok())
                .map(|title| (lookup_key(&title), title))
                .filter(|(key, _)| !key.is_empty())
                .collect()
        })
        .collect();
    let mut spellings: HashMap<&str, Vec<&str>> = HashMap::new();
    for (key, title) in titles.iter().flatten() {
        spellings.entry(key).or_default().push(title);
    }
    // A title that differs from Logseq's spelling of one of its own pages, and a title that
    // some file writes in another spelling.
    let mut respellable: HashSet<&str> = spellings
        .iter()
        .filter(|(key, titles)| {
            built_in_page_title(key).is_some_and(|name| titles.iter().any(|title| *title != name))
        })
        .map(|(key, _)| *key)
        .collect();
    // Per file, the page titles it writes, by lookup key.
    let mut written: Vec<HashSet<&str>> = Vec::with_capacity(documents.len());
    let mut buffer = String::new();
    for document in documents.iter() {
        let mut here = HashSet::new();
        for_each_written_name(document, &mut |name| {
            if let Some((key, titles)) = spellings.get_key_value(key_of(name, &mut buffer)) {
                here.insert(*key);
                if titles.iter().any(|title| *title != name) {
                    respellable.insert(*key);
                }
            }
        });
        written.push(here);
    }
    // The pages that refer to themselves: their index, title key and title.
    let mut own = Vec::new();
    for (index, document) in documents.iter().enumerate() {
        let keys: HashSet<&str> = titles[index]
            .keys()
            .map(String::as_str)
            .filter(|key| respellable.contains(key) && written[index].contains(key))
            .collect();
        if keys.is_empty() {
            continue;
        }
        let here = last_spellings(document, &keys);
        for (key, title) in &titles[index] {
            if here.contains_key(key) {
                own.push((index, key.clone(), title.clone()));
            }
        }
    }
    // The first file Logseq reads that refers to each of those names, with its spelling.
    let names: HashSet<&str> = own.iter().map(|(_, key, _)| key.as_str()).collect();
    let mut first: HashMap<String, String> = HashMap::new();
    for index in parse_order(documents) {
        if first.len() == names.len() {
            break;
        }
        let hits: HashSet<&str> = written[index]
            .iter()
            .copied()
            .filter(|key| names.contains(key) && !first.contains_key(*key))
            .collect();
        if hits.is_empty() {
            continue;
        }
        for (key, spelling) in last_spellings(&documents[index], &hits) {
            first.entry(key).or_insert(spelling);
        }
    }
    let respelled: Vec<(usize, String, String)> = own
        .into_iter()
        .filter_map(|(index, key, title)| {
            // Logseq creates its own pages, such as `TODO`, before it reads any file.
            // Otherwise the first file that refers to the page spells it; that may be the
            // page's own file.
            let spelling = match built_in_page_title(&key) {
                Some(spelling) => spelling.to_owned(),
                None => first[&key].clone(),
            };
            (spelling != title).then_some((index, key, spelling))
        })
        .collect();
    for (index, key, spelling) in respelled {
        documents[index].title_spellings.insert(key, spelling);
    }
}

/// The [`reference_key`] of `name`, written into `buffer` without allocating for an ASCII
/// name.
fn key_of<'b>(name: &str, buffer: &'b mut String) -> &'b str {
    buffer.clear();
    if !name.is_ascii() {
        buffer.push_str(&reference_key(name));
        return buffer;
    }
    let name = if has_outer_space(name) {
        name
    } else {
        name.trim()
    };
    let name = name.strip_prefix('/').unwrap_or(name);
    let name = name.strip_suffix('/').unwrap_or(name);
    buffer.extend(name.chars().map(|letter| letter.to_ascii_lowercase()));
    buffer
}

/// Call `visit` with every name the file may refer to, from the parse that loading already
/// did: its page references and tags, the names in its `alias::`, `aliases::` and `tags::`
/// values, its task markers and priorities, and the namespace parents of each. The names
/// its blocks refer to are among them.
fn for_each_written_name(document: &GardenDocument, visit: &mut dyn FnMut(&str)) {
    let mut with_parents = |name: &str| {
        let mut rest = name;
        visit(rest);
        while let Some(parent) = namespace_parent(rest) {
            visit(parent);
            rest = parent;
        }
    };
    for name in &document.parsed.refs.page {
        with_parents(name);
    }
    for block in &document.parsed.blocks {
        match block {
            Block::Bullet {
                marker, priority, ..
            } => {
                for name in marker.iter().chain(priority) {
                    with_parents(name);
                }
            }
            Block::Properties { props, .. } => {
                for prop in props.iter().filter(|prop| is_linkable_property(&prop.0)) {
                    for name in linkable_property_names(&prop.1) {
                        with_parents(&name);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The spelling the file gives each name in `wanted` that its blocks refer to.
fn last_spellings(document: &GardenDocument, wanted: &HashSet<&str>) -> HashMap<String, String> {
    let names =
        Contribution::from_document(String::new(), document, &|name| Some(name.to_owned()), None);
    let mut spellings = HashMap::new();
    for (position, block) in names.blocks.iter().enumerate() {
        let start = block.anchor.unwrap_or(0);
        let end = names.blocks[position + 1..]
            .iter()
            .find_map(|next| next.anchor)
            .unwrap_or(document.source.len());
        let text = document.source.get(start..end).unwrap_or_default();
        let direct = &names.graph_refs()[position].direct;
        let mut by_key: HashMap<String, Vec<&String>> = HashMap::new();
        for name in &block.refs {
            let key = reference_key(name);
            if wanted.contains(key.as_str()) {
                by_key.entry(key).or_default().push(name);
            }
        }
        for (key, names) in by_key {
            // A namespace parent comes after the written names; among those, the first
            // written comes last.
            let spelling = names
                .iter()
                .rev()
                .find(|name| !direct.contains(**name))
                .or_else(|| {
                    names
                        .iter()
                        .min_by_key(|name| text.find(name.as_str()).unwrap_or(usize::MAX))
                })
                .map(|name| (*name).clone());
            if let Some(spelling) = spelling {
                spellings.insert(key, spelling);
            }
        }
    }
    spellings
}
