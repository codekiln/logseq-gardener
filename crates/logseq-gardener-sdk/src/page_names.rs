//! Candidate lookup for stored page titles and leading Markdown aliases.

use std::collections::{BTreeMap, BTreeSet};

use unicode_normalization::UnicodeNormalization;

use crate::ast::{Block, Inline, Url};
use crate::garden::GardenDocument;

/// Lookup spelling shared by titles, aliases and queries.
///
/// Surrounding whitespace and one boundary slash on each end are removed before
/// lowercasing and NFC normalization. Display titles and filenames are unchanged.
pub fn lookup_key(name: &str) -> String {
    let name = name.trim();
    let name = name.strip_prefix('/').unwrap_or(name);
    let name = name.strip_suffix('/').unwrap_or(name);
    name.to_lowercase().nfc().collect()
}

/// Extract aliases from the last leading `alias` property.
///
/// Plain comma-separated text and direct page references supply names. Quoted
/// values remain literal, including the quotes. Formatted and code spans do not
/// supply inner names; a value with no parsed names remains one literal name.
/// Nested properties and `aliases` metadata do not supply aliases. Returned names
/// are sorted, normalized and deduplicated, omitting blanks and the page itself.
pub fn page_aliases(document: &GardenDocument, title: &str) -> Vec<String> {
    let Some(Block::Properties { props, .. }) = document.parsed.blocks.first() else {
        return Vec::new();
    };
    let Some(property) = props
        .iter()
        .rev()
        .find(|property| property.0.eq_ignore_ascii_case("alias"))
    else {
        return Vec::new();
    };
    let value = property.1.trim();
    let mut names = Vec::new();
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        names.push(value.to_owned());
    } else {
        for inline in lsdoc::inline(value, "markdown") {
            match inline {
                Inline::Plain { text, .. } => names.extend(
                    text.split([',', '，'])
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .map(str::to_owned),
                ),
                Inline::Link {
                    url: Url::PageRef { v },
                    image: false,
                    ..
                } => names.push(v),
                _ => {}
            }
        }
        if names.is_empty() && !value.is_empty() {
            names.push(value.to_owned());
        }
    }
    let title = lookup_key(title);
    names
        .into_iter()
        .map(|name| lookup_key(&name))
        .filter(|name| !name.is_empty() && name != &title)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Deterministic lookup over caller-supplied stored-page identifiers.
///
/// A caller controls which documents enter the index. Publishing inserts only
/// visible selected pages. Different identifiers remain distinct candidates even
/// when an alias overlaps a title; repeated claims by one identifier are unique.
#[derive(Debug, Clone)]
pub struct PageNameIndex<T> {
    names: BTreeMap<String, Vec<T>>,
}

impl<T> Default for PageNameIndex<T> {
    fn default() -> Self {
        Self {
            names: BTreeMap::new(),
        }
    }
}

impl<T: Ord + Clone> PageNameIndex<T> {
    /// Insert the title and leading aliases for one stored page.
    pub fn insert(&mut self, id: T, title: &str, document: &GardenDocument) {
        for name in std::iter::once(lookup_key(title)).chain(page_aliases(document, title)) {
            if name.is_empty() {
                continue;
            }
            let candidates = self.names.entry(name).or_default();
            if let Err(position) = candidates.binary_search(&id) {
                candidates.insert(position, id.clone());
            }
        }
    }

    /// Return sorted distinct candidates: empty means unavailable, one is unique,
    /// and multiple candidates are ambiguous. No filesystem access is performed.
    pub fn candidates(&self, query: &str) -> &[T] {
        self.names
            .get(&lookup_key(query))
            .map_or(&[], Vec::as_slice)
    }
}
