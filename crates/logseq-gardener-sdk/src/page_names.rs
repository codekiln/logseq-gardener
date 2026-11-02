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

/// The namespace parent of a page name, as Logseq's file parser splits it: the text before
/// the last `/`. A space before that slash stays, so the parent of
/// `MacOS /Applications Folder` is `MacOS `, which Logseq keeps apart from `MacOS`.
pub fn namespace_parent(name: &str) -> Option<&str> {
    name.rsplit_once('/').map(|(parent, _)| parent)
}

/// Whether a name ends in a space, as a [`namespace_parent`] can. A written page name has
/// its spaces trimmed, so such a name never names a garden page or a written reference.
pub fn ends_in_space(name: &str) -> bool {
    name.len() != name.trim_end().len()
}

/// [`lookup_key`] that keeps a space at the end of the name, as Logseq does for a
/// [`namespace_parent`]: `MacOS ` and `MacOS` get different keys. For a name without
/// such a space it is [`lookup_key`].
pub fn namespace_parent_key(name: &str) -> String {
    if !ends_in_space(name) {
        return lookup_key(name);
    }
    let name = name.trim_start();
    let name = name.strip_prefix('/').unwrap_or(name);
    name.to_lowercase().nfc().collect()
}

/// Extract aliases from the last leading `alias` property.
///
/// The value's names are [`linkable_property_names`]. Quoted values remain literal,
/// including the quotes, and a value with no names, such as one formatted span, remains
/// one literal name. Nested properties and `aliases` metadata do not supply aliases.
/// Returned names are sorted, normalized and deduplicated, omitting blanks and the page
/// itself.
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
    let names = linkable_property_names(value);
    if names.is_empty() && !value.is_empty() {
        return vec![value.to_owned()];
    }
    names
}

/// The pages Logseq creates in every graph, spelled as Logseq spells them: the task
/// markers, the priorities and three pages of its own (`built-in-pages-names` in
/// Logseq's `logseq.db.default`).
const BUILT_IN_PAGE_TITLES: [&str; 16] = [
    "NOW",
    "LATER",
    "DOING",
    "DONE",
    "CANCELED",
    "CANCELLED",
    "IN-PROGRESS",
    "TODO",
    "WAIT",
    "WAITING",
    "A",
    "B",
    "C",
    "Favorites",
    "Contents",
    "card",
];

/// Logseq's own spelling of a page it creates in every graph, such as `C` for `c` or
/// `TODO` for `todo`, when `name` names one of them in any case.
///
/// Logseq creates these pages before it reads a file, so a reference keeps that spelling
/// whatever the reference's case.
pub fn built_in_page_title(name: &str) -> Option<&'static str> {
    // A namespace parent such as `A ` for `A /Notes` is a page of its own, not `A`.
    let key = namespace_parent_key(name);
    BUILT_IN_PAGE_TITLES
        .iter()
        .copied()
        .find(|title| lookup_key(title) == key)
}

/// Whether Logseq reads a property's value as a list of page names: `alias`, `aliases`
/// and `tags`, in any case.
pub fn is_linkable_property(name: &str) -> bool {
    ["alias", "aliases", "tags"]
        .iter()
        .any(|linkable| name.trim().eq_ignore_ascii_case(linkable))
}

/// The page names in an `alias::`, `aliases::` or `tags::` value, as Logseq's file
/// parser reads them, in written order without repeats.
///
/// A page reference `[[Name]]`, a tag `#name` or `#[[Name]]`, and a nested reference
/// `[[Outer [[Inner]] Name]]`, which names `Outer [[Inner]] Name` only, each supply one
/// name, and plain text between them supplies its comma-separated parts, split on ASCII
/// or full-width commas. Code, formatted spans and other links supply nothing, and a
/// value in double quotes supplies nothing.
pub fn linkable_property_names(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Vec::new();
    }
    let mut names: Vec<String> = Vec::new();
    let mut push = |name: &str| {
        let name = name.trim();
        if !name.is_empty() && !names.iter().any(|known| known == name) {
            names.push(name.to_owned());
        }
    };
    for inline in lsdoc::inline(value, "markdown") {
        match &inline {
            Inline::Plain { text, .. } => text.split([',', '，']).for_each(&mut push),
            Inline::Tag { children, .. } => {
                // Logseq takes a tag's first part: its text, or the reference it wraps.
                match children.first() {
                    Some(Inline::Plain { text, .. }) => push(text),
                    Some(other) => {
                        if let Some(name) = reference_name(other) {
                            push(&name);
                        }
                    }
                    None => {}
                }
            }
            other => {
                if let Some(name) = reference_name(other) {
                    push(&name);
                }
            }
        }
    }
    names
}

/// The page a direct or nested page reference names.
fn reference_name(inline: &Inline) -> Option<String> {
    match inline {
        Inline::Link {
            url: Url::PageRef { v },
            image: false,
            ..
        } => Some(v.clone()),
        Inline::NestedLink { content, .. } => content
            .strip_prefix("[[")
            .and_then(|inner| inner.strip_suffix("]]"))
            .map(str::to_owned),
        _ => None,
    }
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
