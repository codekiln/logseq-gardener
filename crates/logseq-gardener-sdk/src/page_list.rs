//! Logseq's All pages list: every published page, journal and generated namespace page
//! with its kind, icon, backlink count, sort key and git dates.
//!
//! ```
//! use logseq_gardener_sdk::page_list::{Column, PageKind, PageListEntry};
//!
//! let entry = |title: &str, backlinks| PageListEntry {
//!     id: title.to_owned(),
//!     title: title.to_owned(),
//!     kind: PageKind::Page,
//!     icon: None,
//!     backlinks,
//!     sort_key: title.to_lowercase(),
//!     created: None,
//!     updated: None,
//! };
//! let mut list = [entry("Logseq", 6429), entry("Microfreak", 7662)];
//! list.sort_by(|left, right| left.cmp_by(right, Column::Backlinks, true));
//! assert_eq!(list[0].title, "Microfreak");
//! ```
use std::cmp::Ordering;

use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::git_dates::GitDates;
use crate::references::ReferenceIndex;

/// What a listed page is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PageKind {
    /// A page with its own file.
    Page,
    /// A journal day.
    Journal,
    /// A namespace page generated for a prefix that has no file.
    Namespace,
}

/// A page to list: its id in the reference index, its title and kind, and its source
/// file when it has one.
pub struct PageSource<'a, K> {
    pub id: K,
    pub title: String,
    pub kind: PageKind,
    pub document: Option<&'a GardenDocument>,
}

/// One row of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageListEntry<K> {
    pub id: K,
    pub title: String,
    pub kind: PageKind,
    /// The page's `icon::` property.
    pub icon: Option<String>,
    /// Blocks whose references include the page, as [`ReferenceIndex::backlink_count`].
    pub backlinks: usize,
    /// The lower-cased title that orders names.
    pub sort_key: String,
    /// First commit time of the page's file, in seconds since the Unix epoch.
    pub created: Option<i64>,
    /// Last commit time of the page's file, in seconds since the Unix epoch.
    pub updated: Option<i64>,
}

/// A column to order the list by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    Name,
    Backlinks,
    Created,
    Updated,
}

/// List pages with their attributes, in the order given. `dates` is `None` when the
/// garden has no git history; generated namespace pages never carry dates.
pub fn page_list<'a, K: Ord + Clone>(
    pages: impl IntoIterator<Item = PageSource<'a, K>>,
    references: &ReferenceIndex<K>,
    dates: Option<&GitDates>,
) -> Vec<PageListEntry<K>> {
    pages
        .into_iter()
        .map(|page| {
            let file_dates = page.document.zip(dates).and_then(|(document, dates)| {
                dates.get(&document.relative_path.to_string_lossy().replace('\\', "/"))
            });
            PageListEntry {
                backlinks: references.backlink_count(&page.id),
                icon: page.document.and_then(icon),
                sort_key: page.title.to_lowercase(),
                created: file_dates.map(|dates| dates.created),
                updated: file_dates.map(|dates| dates.updated),
                id: page.id,
                title: page.title,
                kind: page.kind,
            }
        })
        .collect()
}

/// The last leading `icon::` property value, when it is not blank.
fn icon(document: &GardenDocument) -> Option<String> {
    let Some(Block::Properties { props, .. }) = document.parsed.blocks.first() else {
        return None;
    };
    props
        .iter()
        .rev()
        .find(|property| property.0.eq_ignore_ascii_case("icon"))
        .map(|property| property.1.trim().to_owned())
        .filter(|value| !value.is_empty())
}

impl<K> PageListEntry<K> {
    /// Logseq's order for a column: names by sort key, numbers and dates by value, entries
    /// without a value after those with one in either direction, ties by sort key.
    pub fn cmp_by(&self, other: &Self, column: Column, descending: bool) -> Ordering {
        let directed = |ordering: Ordering| {
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        };
        let optional = |left: Option<i64>, right: Option<i64>| match (left, right) {
            (Some(left), Some(right)) => directed(left.cmp(&right)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        let primary = match column {
            Column::Name => directed(self.sort_key.cmp(&other.sort_key)),
            Column::Backlinks => directed(self.backlinks.cmp(&other.backlinks)),
            Column::Created => optional(self.created, other.created),
            Column::Updated => optional(self.updated, other.updated),
        };
        primary.then_with(|| self.sort_key.cmp(&other.sort_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;
    use crate::page_names::lookup_key;
    use crate::references::{Contribution, PageEntry};

    fn document(path: &str, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: path.into(),
            kind: if path.starts_with("journals/") {
                DocumentKind::Journal
            } else {
                DocumentKind::Page
            },
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    fn entry(title: &str, backlinks: usize, updated: Option<i64>) -> PageListEntry<String> {
        PageListEntry {
            id: title.to_owned(),
            title: title.to_owned(),
            kind: PageKind::Page,
            icon: None,
            backlinks,
            sort_key: title.to_lowercase(),
            created: updated,
            updated,
        }
    }

    fn list() -> Vec<PageListEntry<String>> {
        let mise = document(
            "pages/mise.md",
            "icon:: 🧭\n\n- self [[mise]]\n- [[tmux/Command]]\n",
        );
        let journal = document(
            "journals/2026_10_23.md",
            "- [[mise]] and [[tmux/Command/kill]]\n",
        );
        let kill = document("pages/tmux___Command___kill.md", "- kill\n");
        let titles = [
            ("mise", "mise"),
            ("2026-10-23 Fri", "journal"),
            ("tmux/Command/kill", "kill"),
            ("tmux/Command", "command"),
            ("tmux", "tmux"),
        ];
        let resolve = |name: &str| {
            titles
                .iter()
                .find(|(title, _)| lookup_key(title) == lookup_key(name))
                .map(|(_, id)| (*id).to_owned())
        };
        let index = ReferenceIndex::build(
            titles.iter().map(|(title, id)| PageEntry {
                id: (*id).to_owned(),
                title: (*title).to_owned(),
                aliases: vec![],
            }),
            vec![
                Contribution::from_document("mise".to_owned(), &mise, &resolve, None),
                Contribution::from_document("journal".to_owned(), &journal, &resolve, None),
                Contribution::from_document("kill".to_owned(), &kill, &resolve, None),
            ],
        );
        let dates = GitDates::default();
        page_list(
            [
                PageSource {
                    id: "mise".to_owned(),
                    title: "mise".to_owned(),
                    kind: PageKind::Page,
                    document: Some(&mise),
                },
                PageSource {
                    id: "journal".to_owned(),
                    title: "2026-10-23 Fri".to_owned(),
                    kind: PageKind::Journal,
                    document: Some(&journal),
                },
                PageSource {
                    id: "command".to_owned(),
                    title: "tmux/Command".to_owned(),
                    kind: PageKind::Namespace,
                    document: None,
                },
            ],
            &index,
            Some(&dates),
        )
    }

    #[test]
    fn kinds_icons_and_sort_keys() {
        let list = list();
        let kinds: Vec<_> = list.iter().map(|entry| entry.kind).collect();
        assert_eq!(
            kinds,
            [PageKind::Page, PageKind::Journal, PageKind::Namespace]
        );
        assert_eq!(list[0].icon.as_deref(), Some("🧭"));
        assert_eq!(list[1].icon, None);
        assert_eq!(list[2].sort_key, "tmux/command");
        // Dates absent from the history leave the entry undated.
        assert_eq!(list[0].updated, None);
    }

    #[test]
    fn backlinks_count_own_blocks_and_namespace_children() {
        let list = list();
        // mise: its own block and the journal's block.
        assert_eq!(list[0].backlinks, 2);
        // tmux/Command: mise's block and the journal's reference to its child.
        assert_eq!(list[2].backlinks, 2);
    }

    #[test]
    fn backlinks_descending_then_name() {
        let mut list = [
            entry("Logseq/Entity", 5902, None),
            entry("Logseq", 6429, None),
            entry("Microfreak", 7662, None),
            entry("a", 0, None),
        ];
        list.sort_by(|left, right| left.cmp_by(right, Column::Backlinks, true));
        let titles: Vec<_> = list.iter().map(|entry| entry.title.as_str()).collect();
        assert_eq!(titles, ["Microfreak", "Logseq", "Logseq/Entity", "a"]);
        list.sort_by(|left, right| left.cmp_by(right, Column::Backlinks, false));
        assert_eq!(list[0].title, "a");
    }

    #[test]
    fn undated_entries_come_last_in_either_direction() {
        let mut list = [
            entry("b", 0, None),
            entry("old", 0, Some(1)),
            entry("a", 0, None),
            entry("new", 0, Some(2)),
        ];
        for descending in [true, false] {
            list.sort_by(|left, right| left.cmp_by(right, Column::Updated, descending));
            let titles: Vec<_> = list.iter().map(|entry| entry.title.as_str()).collect();
            let expected = if descending {
                ["new", "old", "a", "b"]
            } else {
                ["old", "new", "a", "b"]
            };
            assert_eq!(titles, expected);
        }
    }

    #[test]
    fn names_order_by_lower_cased_sort_key() {
        let mut list = [
            entry("b", 0, None),
            entry("A", 0, None),
            entry(")", 0, None),
        ];
        list.sort_by(|left, right| left.cmp_by(right, Column::Name, false));
        let titles: Vec<_> = list.iter().map(|entry| entry.title.as_str()).collect();
        assert_eq!(titles, [")", "A", "b"]);
    }
}
