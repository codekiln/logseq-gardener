//! A page's namespace hierarchy, answered as Logseq's Hierarchy section lists it.
//!
//! The index holds one [`NamespaceEntry`] per page and every `/` prefix those
//! pages imply. [`NamespaceIndex::hierarchy`] picks the page or its first
//! namespaced alias and returns its descendants, or its parent's path for a leaf.
//!
//! ```
//! use logseq_gardener_sdk::namespaces::{NamespaceEntry, NamespaceIndex, RowFilter};
//!
//! let entry = |title: &str| NamespaceEntry::new(title, title, Vec::new());
//! let index = NamespaceIndex::build(
//!     [entry("tmux/Config"), entry("tmux/Config/local")],
//!     [],
//! );
//! let rows = index.hierarchy("tmux/Config", RowFilter::StoredOnly).unwrap().rows;
//! assert_eq!(rows[0].name, "tmux/Config/local");
//! let leaf = index.hierarchy("tmux/Config/local", RowFilter::StoredOnly).unwrap();
//! assert_eq!(leaf.rows[0].name, "tmux/Config");
//! ```

use std::collections::{BTreeMap, BTreeSet};

use crate::page_names::lookup_key;

/// One page: its key, its title and its aliases in authored order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceEntry {
    pub key: String,
    pub title: String,
    pub aliases: Vec<String>,
}

impl NamespaceEntry {
    pub fn new(key: impl Into<String>, title: impl Into<String>, aliases: Vec<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            aliases,
        }
    }
}

/// Which descendant rows a caller wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowFilter {
    /// Only descendants that are pages; prefixes without a page stay out.
    StoredOnly,
    /// Every descendant, including prefixes without a page.
    All,
}

/// One segment of a row: its label in the row's spelling and the prefix it leads to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub label: String,
    pub prefix: String,
}

/// One row: a full namespace name and whether a page holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub stored: bool,
}

impl Row {
    /// The row split on `/`, each segment with its lower-cased cumulative prefix.
    pub fn segments(&self) -> Vec<Segment> {
        let mut prefix = String::new();
        self.name
            .split('/')
            .map(|label| {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(label);
                Segment {
                    label: label.to_owned(),
                    prefix: lookup_key(&prefix),
                }
            })
            .collect()
    }
}

/// The subject a page's hierarchy is about and the rows it lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hierarchy {
    pub subject: String,
    pub rows: Vec<Row>,
}

#[derive(Default)]
struct Name {
    /// Keys of the entries whose title is this name.
    pages: BTreeSet<String>,
    /// Spellings of this name as a prefix of other titles, with how many use each.
    prefixes: BTreeMap<String, usize>,
}

/// Namespace names from page titles, kept one entry at a time.
#[derive(Default)]
pub struct NamespaceIndex {
    entries: BTreeMap<String, NamespaceEntry>,
    names: BTreeMap<String, Name>,
    withheld: BTreeSet<String>,
}

fn ancestors(title: &str) -> impl Iterator<Item = &str> {
    title.match_indices('/').map(|(end, _)| &title[..end])
}

impl NamespaceIndex {
    /// Index `entries`. Names in `withheld` (lookup keys) never become rows or prefixes.
    pub fn build(
        entries: impl IntoIterator<Item = NamespaceEntry>,
        withheld: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut index = Self {
            withheld: withheld.into_iter().collect(),
            ..Self::default()
        };
        for entry in entries {
            index.replace(entry);
        }
        index
    }

    /// Add or swap one page's entry, touching only the names its titles imply.
    pub fn replace(&mut self, entry: NamespaceEntry) {
        self.remove(&entry.key.clone());
        let title = entry.title.clone();
        self.names
            .entry(lookup_key(&title))
            .or_default()
            .pages
            .insert(entry.key.clone());
        for prefix in ancestors(&title) {
            let key = lookup_key(prefix);
            if self.withheld.contains(&key) {
                continue;
            }
            *self
                .names
                .entry(key)
                .or_default()
                .prefixes
                .entry(prefix.to_owned())
                .or_default() += 1;
        }
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Drop one page's entry and the prefixes only it implied.
    pub fn remove(&mut self, key: &str) {
        let Some(entry) = self.entries.remove(key) else {
            return;
        };
        let title_key = lookup_key(&entry.title);
        if let Some(name) = self.names.get_mut(&title_key) {
            name.pages.remove(key);
        }
        self.prune(&title_key);
        for prefix in ancestors(&entry.title) {
            let name_key = lookup_key(prefix);
            if let Some(name) = self.names.get_mut(&name_key)
                && let Some(count) = name.prefixes.get_mut(prefix)
            {
                *count -= 1;
                if *count == 0 {
                    name.prefixes.remove(prefix);
                }
            }
            self.prune(&name_key);
        }
    }

    fn prune(&mut self, key: &str) {
        if self
            .names
            .get(key)
            .is_some_and(|name| name.pages.is_empty() && name.prefixes.is_empty())
        {
            self.names.remove(key);
        }
    }

    /// How many names the index holds, pages and prefixes together.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    fn spelling(&self, key: &str) -> Option<(String, bool)> {
        let name = self.names.get(key)?;
        match name.pages.first() {
            Some(page) => Some((self.entries[page].title.clone(), true)),
            None => name.prefixes.keys().next().map(|p| (p.clone(), false)),
        }
    }

    fn descendants(&self, key: &str, filter: RowFilter) -> Vec<Row> {
        let prefix = format!("{key}/");
        self.names
            .range(prefix.clone()..)
            .take_while(|(name, _)| name.starts_with(&prefix))
            .filter_map(|(name, _)| self.spelling(name))
            .filter(|(_, stored)| *stored || filter == RowFilter::All)
            .map(|(name, stored)| Row { name, stored })
            .collect()
    }

    /// The hierarchy Logseq shows on `page`, a page title or namespace prefix.
    pub fn hierarchy(&self, page: &str, filter: RowFilter) -> Option<Hierarchy> {
        let key = lookup_key(page);
        let (title, _) = self.spelling(&key)?;
        let has_children = !self.descendants(&key, RowFilter::All).is_empty();
        let subject = if title.contains('/') || has_children {
            title
        } else {
            let entry = &self.entries[self.names[&key].pages.first()?];
            entry
                .aliases
                .iter()
                .find(|alias| alias.contains('/'))?
                .clone()
        };
        let subject_key = lookup_key(&subject);
        let mut rows = self.descendants(&subject_key, filter);
        if rows.is_empty() {
            // As Logseq, the parent row takes the parent page's own spelling.
            let (parent, _) = subject.rsplit_once('/')?;
            let (name, stored) = self
                .spelling(&lookup_key(parent))
                .unwrap_or_else(|| (parent.to_owned(), false));
            rows.push(Row { name, stored });
        }
        Some(Hierarchy { subject, rows })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str) -> NamespaceEntry {
        NamespaceEntry::new(title, title, Vec::new())
    }

    fn index(titles: &[&str]) -> NamespaceIndex {
        NamespaceIndex::build(titles.iter().map(|title| entry(title)), [])
    }

    fn names(index: &NamespaceIndex, page: &str, filter: RowFilter) -> Vec<String> {
        index
            .hierarchy(page, filter)
            .map(|h| h.rows.into_iter().map(|row| row.name).collect())
            .unwrap_or_default()
    }

    #[test]
    fn descendants_leave_out_the_own_row() {
        let index = index(&["tmux/Config", "tmux/Config/local"]);
        assert_eq!(
            names(&index, "tmux/Config", RowFilter::StoredOnly),
            ["tmux/Config/local"]
        );
    }

    #[test]
    fn a_leaf_shows_its_parent() {
        let index = index(&["tmux/Config", "tmux/Config/local"]);
        let hierarchy = index
            .hierarchy("tmux/Config/local", RowFilter::StoredOnly)
            .unwrap();
        assert_eq!(hierarchy.subject, "tmux/Config/local");
        assert_eq!(
            hierarchy.rows,
            [Row {
                name: "tmux/Config".into(),
                stored: true
            }]
        );
    }

    #[test]
    fn a_plain_page_has_no_hierarchy() {
        let index = index(&["Abbreviation", "tmux/Config"]);
        assert_eq!(index.hierarchy("Abbreviation", RowFilter::All), None);
        assert_eq!(index.hierarchy("Missing", RowFilter::All), None);
    }

    #[test]
    fn a_page_takes_its_first_namespaced_alias() {
        let index = NamespaceIndex::build(
            [NamespaceEntry::new(
                "agi",
                "AGI",
                vec!["General AI".into(), "AI/General/Intelligence".into()],
            )],
            [],
        );
        let hierarchy = index.hierarchy("AGI", RowFilter::StoredOnly).unwrap();
        assert_eq!(hierarchy.subject, "AI/General/Intelligence");
        assert_eq!(
            hierarchy.rows,
            [Row {
                name: "AI/General".into(),
                stored: false
            }]
        );
    }

    #[test]
    fn a_sibling_with_a_shared_prefix_is_not_a_row() {
        let index = index(&["A/B/C", "A/B/Cousin", "A/B/C/D"]);
        assert_eq!(names(&index, "A/B/C", RowFilter::All), ["A/B/C/D"]);
    }

    #[test]
    fn prefixes_without_a_page_follow_the_filter() {
        let index = index(&["tmux", "tmux/Command/kill-session"]);
        assert_eq!(
            names(&index, "tmux", RowFilter::StoredOnly),
            ["tmux/Command/kill-session"]
        );
        assert_eq!(
            names(&index, "tmux", RowFilter::All),
            ["tmux/Command", "tmux/Command/kill-session"]
        );
        let row = &index.hierarchy("tmux", RowFilter::StoredOnly).unwrap().rows[0];
        assert_eq!(row.segments()[1].label, "Command");
        assert_eq!(row.segments()[1].prefix, "tmux/command");
        assert_eq!(
            names(&index, "tmux/Command", RowFilter::StoredOnly),
            ["tmux/Command/kill-session"]
        );
    }

    #[test]
    fn a_leaf_under_a_prefix_without_a_page_still_shows_its_parent() {
        let index = index(&["tmux/Command/kill-session"]);
        let hierarchy = index
            .hierarchy("tmux/Command/kill-session", RowFilter::StoredOnly)
            .unwrap();
        assert_eq!(
            hierarchy.rows,
            [Row {
                name: "tmux/Command".into(),
                stored: false
            }]
        );
    }

    #[test]
    fn the_parent_row_takes_the_parent_spelling() {
        let index = index(&["node", "Node/One"]);
        assert_eq!(names(&index, "Node/One", RowFilter::StoredOnly), ["node"]);
    }

    #[test]
    fn labels_keep_the_row_spelling() {
        let index = index(&["Tools/Code", "tools/code/Rust"]);
        let row = &index.hierarchy("Tools/Code", RowFilter::All).unwrap().rows[0];
        let labels = row
            .segments()
            .into_iter()
            .map(|s| s.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, ["tools", "code", "Rust"]);
    }

    #[test]
    fn rows_sort_by_lower_cased_name() {
        let index = index(&["R", "R/b", "R/A", "R/a/z", "R/C"]);
        assert_eq!(
            names(&index, "R", RowFilter::All),
            ["R/A", "R/a/z", "R/b", "R/C"]
        );
    }

    #[test]
    fn withheld_names_give_no_rows_and_no_prefixes() {
        let index = NamespaceIndex::build(
            [entry("Root"), entry("Root/Private/PublicChild")],
            ["root/private".to_owned()],
        );
        assert_eq!(
            names(&index, "Root", RowFilter::All),
            ["Root/Private/PublicChild"]
        );
    }

    #[test]
    fn replacing_one_entry_touches_only_its_names() {
        let mut index = index(&["A", "A/B", "X/Y"]);
        let before = index.len();
        index.replace(NamespaceEntry::new("A/B", "A/C", Vec::new()));
        assert_eq!(names(&index, "A", RowFilter::All), ["A/C"]);
        assert_eq!(index.len(), before);
        assert_eq!(names(&index, "X", RowFilter::All), ["X/Y"]);
        let fresh = NamespaceIndex::build([entry("A"), entry("X/Y"), entry("A/C")], []);
        assert_eq!(index.len(), fresh.len());
        index.remove("X/Y");
        assert_eq!(index.hierarchy("X", RowFilter::All), None);
        assert_eq!(index.len(), 2);
    }
}
