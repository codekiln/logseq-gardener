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
    let key = lookup_key(name);
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

/// One name in a page's alias row: the name as shown, the page it opens, and whether the
/// page's own `alias::` value wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasEntry<T> {
    /// The name as the author wrote it, or the title of the page it names.
    pub name: String,
    /// The page the name opens.
    pub page: T,
    /// True when the page's own `alias::` value holds the name; false when the name comes
    /// from another page's `alias::` value.
    pub authored: bool,
}

/// Logseq's alias relation among caller-supplied pages, for the `alias` row of a page's
/// properties.
///
/// Logseq links a page to each of its aliases, and each alias to the page and to the
/// page's other aliases. A page's alias group holds every name one or two links away in
/// either direction (`page-alias-set` and the `alias` rule in Logseq's `frontend.db.model`
/// and `logseq.db.rules`), so it also holds the pages that name this page as their alias.
/// Only the pages the caller inserts take part, so a withheld page adds no name.
#[derive(Debug, Clone)]
pub struct AliasGroups<T> {
    /// Lookup key of a title to the page's identifier and title.
    pages: BTreeMap<String, (T, String)>,
    /// Identifier to the lookup key of the page's title.
    keys: BTreeMap<T, String>,
    /// Identifier to the names of the page's own `alias::` value, in property order.
    own: BTreeMap<T, Vec<String>>,
    /// Lookup key of an alias to its first spelling.
    spellings: BTreeMap<String, String>,
    /// Lookup key of an alias to the sorted identifiers of the pages that claim it.
    claims: BTreeMap<String, Vec<T>>,
    /// Alias links, by lookup key, in both directions.
    out: BTreeMap<String, BTreeSet<String>>,
    into: BTreeMap<String, BTreeSet<String>>,
}

impl<T> Default for AliasGroups<T> {
    fn default() -> Self {
        Self {
            pages: BTreeMap::new(),
            keys: BTreeMap::new(),
            own: BTreeMap::new(),
            spellings: BTreeMap::new(),
            claims: BTreeMap::new(),
            out: BTreeMap::new(),
            into: BTreeMap::new(),
        }
    }
}

impl<T: Ord + Clone> AliasGroups<T> {
    /// Add one page with its title and leading `alias::` value.
    pub fn insert(&mut self, id: T, title: &str, document: &GardenDocument) {
        let page = lookup_key(title);
        if page.is_empty() {
            return;
        }
        self.pages
            .entry(page.clone())
            .or_insert_with(|| (id.clone(), title.to_owned()));
        self.keys.insert(id.clone(), page.clone());
        let spellings = page_alias_spellings(document, title);
        let aliases: Vec<String> = spellings.iter().map(|name| lookup_key(name)).collect();
        // Logseq lists every name the value holds once, in any case, including the page's
        // own title; it links only the other names to the page.
        let mut seen = BTreeSet::new();
        let shown = alias_names(document)
            .into_iter()
            .map(|name| name.trim().to_owned())
            .filter(|name| !lookup_key(name).is_empty() && seen.insert(lookup_key(name)))
            .collect();
        for (alias, spelling) in aliases.iter().zip(&spellings) {
            self.spellings
                .entry(alias.clone())
                .or_insert_with(|| spelling.clone());
            insert_sorted(self.claims.entry(alias.clone()).or_default(), id.clone());
            self.link(&page, alias);
            self.link(alias, &page);
            for other in aliases.iter().filter(|other| *other != alias) {
                self.link(alias, other);
            }
        }
        self.own.insert(id, shown);
    }

    fn link(&mut self, from: &str, to: &str) {
        self.out
            .entry(from.to_owned())
            .or_default()
            .insert(to.to_owned());
        self.into
            .entry(to.to_owned())
            .or_default()
            .insert(from.to_owned());
    }

    /// The names Logseq lists in the `alias` row of page `id`: the names of the page's own
    /// `alias::` value as written, in property order and once in any case, then the other
    /// names in its alias group, by lookup key. A name that spells the page's own title opens
    /// the page.
    ///
    /// A name opens the page with that title when one was inserted. Otherwise it opens the
    /// page that claims it as an alias: page `id` itself when it does, or else the first
    /// claiming page in the alias group. Two pages that claim the same alias are both in
    /// each other's group, so neither page drops the name.
    pub fn entries(&self, id: &T) -> Vec<AliasEntry<T>> {
        let Some(page) = self.keys.get(id) else {
            return Vec::new();
        };
        let none = BTreeSet::new();
        let out = |key: &str| self.out.get(key).unwrap_or(&none);
        let into = |key: &str| self.into.get(key).unwrap_or(&none);
        let mut group: BTreeSet<&String> = out(page).iter().chain(into(page)).collect();
        group.extend(out(page).iter().flat_map(|key| out(key)));
        group.extend(into(page).iter().flat_map(|key| into(key)));
        group.remove(page);
        let target = |key: &str| -> Option<T> {
            if let Some((target, _)) = self.pages.get(key) {
                return Some(target.clone());
            }
            let claims = self.claims.get(key)?;
            if claims.contains(id) {
                return Some(id.clone());
            }
            claims
                .iter()
                .find(|claim| self.keys.get(*claim).is_some_and(|key| group.contains(key)))
                .or(claims.first())
                .cloned()
        };
        let mut seen = BTreeSet::new();
        let mut entries = Vec::new();
        for name in self.own.get(id).into_iter().flatten() {
            let key = lookup_key(name);
            let target = if &key == page {
                Some(id.clone())
            } else {
                target(&key)
            };
            if seen.insert(key)
                && let Some(target) = target
            {
                entries.push(AliasEntry {
                    name: name.clone(),
                    page: target,
                    authored: true,
                });
            }
        }
        seen.insert(page.clone());
        for key in &group {
            if !seen.insert((*key).clone()) {
                continue;
            }
            let name = self
                .pages
                .get(*key)
                .map(|(_, title)| title)
                .or_else(|| self.spellings.get(*key));
            if let (Some(name), Some(target)) = (name, target(key)) {
                entries.push(AliasEntry {
                    name: name.clone(),
                    page: target,
                    authored: false,
                });
            }
        }
        entries
    }
}
