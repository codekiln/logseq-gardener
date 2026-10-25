//! Candidate lookup for stored page titles and leading Markdown aliases.

use std::collections::{BTreeMap, BTreeSet};

use unicode_normalization::UnicodeNormalization;

use crate::ast::{Block, Inline, Url};
use crate::garden::GardenDocument;
use crate::journals::{JournalDate, JournalFormats, parse_journal_reference};

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
    let title = lookup_key(title);
    alias_names(document)
        .into_iter()
        .map(|name| lookup_key(&name))
        .filter(|name| !name.is_empty() && name != &title)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Aliases as the author spelled them, in property order, one per lookup key.
///
/// Uses the same property rules as [`page_aliases`], omitting blanks and the page itself.
pub fn page_alias_spellings(document: &GardenDocument, title: &str) -> Vec<String> {
    let mut seen = BTreeSet::from([lookup_key(title)]);
    alias_names(document)
        .into_iter()
        .map(|name| name.trim().to_owned())
        .filter(|name| !lookup_key(name).is_empty() && seen.insert(lookup_key(name)))
        .collect()
}

fn alias_names(document: &GardenDocument) -> Vec<String> {
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
    names
}

/// Deterministic lookup over caller-supplied stored-page identifiers.
///
/// A caller controls which documents enter the index. Publishing inserts only
/// visible selected pages. Different identifiers remain distinct candidates even
/// when an alias overlaps a title; repeated claims by one identifier are unique.
#[derive(Debug, Clone)]
pub struct PageNameIndex<T> {
    names: BTreeMap<String, Vec<T>>,
    journals: BTreeMap<JournalDate, Vec<T>>,
    journal_formats: JournalFormats,
}

impl<T> Default for PageNameIndex<T> {
    fn default() -> Self {
        Self {
            names: BTreeMap::new(),
            journals: BTreeMap::new(),
            journal_formats: JournalFormats::default(),
        }
    }
}

impl<T: Ord + Clone> PageNameIndex<T> {
    /// An empty index that reads journal references with the garden's formats.
    pub fn with_journal_formats(journal_formats: JournalFormats) -> Self {
        Self {
            journal_formats,
            ..Self::default()
        }
    }

    /// Insert the title and leading aliases for one stored page.
    pub fn insert(&mut self, id: T, title: &str, document: &GardenDocument) {
        for name in std::iter::once(lookup_key(title)).chain(page_aliases(document, title)) {
            if name.is_empty() {
                continue;
            }
            insert_sorted(self.names.entry(name).or_default(), id.clone());
        }
    }

    /// Let references written as `date` in any accepted journal format find `id`.
    ///
    /// Insert the journal's title and aliases with [`Self::insert`] as well.
    pub fn insert_journal(&mut self, id: T, date: JournalDate) {
        insert_sorted(self.journals.entry(date).or_default(), id);
    }

    /// Return sorted distinct candidates: empty means unavailable, one is unique,
    /// and multiple candidates are ambiguous. A query that names no title or alias
    /// falls back to the journal with the date it spells, as Logseq resolves
    /// `[[Feb 1st, 2025]]`. No filesystem access is performed.
    pub fn candidates(&self, query: &str) -> &[T] {
        if let Some(candidates) = self.names.get(&lookup_key(query)) {
            return candidates;
        }
        parse_journal_reference(query, &self.journal_formats)
            .and_then(|date| self.journals.get(&date))
            .map_or(&[], Vec::as_slice)
    }
}

fn insert_sorted<T: Ord>(candidates: &mut Vec<T>, id: T) {
    if let Err(position) = candidates.binary_search(&id) {
        candidates.insert(position, id);
    }
}
