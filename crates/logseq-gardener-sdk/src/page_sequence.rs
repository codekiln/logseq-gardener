//! The previous and next pages that a page names with its leading `prev::` and `next::`
//! properties.
//!
//! A neighbour comes from the last leading `prev::` or `next::` property, in any case, whose
//! whole value is one page reference such as `[[Book/Chapter 2]]`. The reference names a
//! neighbour only when it resolves to exactly one published page other than the page itself,
//! so a private, excluded, missing or ambiguous target, or the page itself, gives none.
//!
//! The publisher draws the Page sequence region at the bottom of a page from [`neighbour`],
//! and the first-party `sequence-header` plugin contributes the same neighbours to its
//! browser module, so both always name the same pages.
//!
//! ```
//! use std::collections::BTreeMap;
//! use logseq_gardener_sdk::page_names::PageNameIndex;
//! use logseq_gardener_sdk::page_sequence::{Direction, neighbour, written_target};
//!
//! let source = "prev:: [[Book/Chapter 1]]\nnext:: [[Missing]]\n\n- Chapter two\n";
//! let document = logseq_gardener_sdk::garden::GardenDocument {
//!     relative_path: "pages/Book___Chapter 2.md".into(),
//!     kind: logseq_gardener_sdk::garden::DocumentKind::Page,
//!     source: source.to_owned(),
//!     parsed: lsdoc::parse_to_projection(source),
//!     title_spellings: Default::default(),
//! };
//! assert_eq!(written_target(&document, Direction::Next).as_deref(), Some("Missing"));
//!
//! let chapter_one = logseq_gardener_sdk::garden::GardenDocument {
//!     relative_path: "pages/Book___Chapter 1.md".into(),
//!     source: "- Chapter one\n".to_owned(),
//!     parsed: lsdoc::parse_to_projection("- Chapter one\n"),
//!     ..document.clone()
//! };
//! let mut names = PageNameIndex::default();
//! names.insert("one.html".to_owned(), "Book/Chapter 1", &chapter_one);
//! names.insert("two.html".to_owned(), "Book/Chapter 2", &document);
//! let routes = BTreeMap::from([
//!     ("one.html".to_owned(), "Book/Chapter 1".to_owned()),
//!     ("two.html".to_owned(), "Book/Chapter 2".to_owned()),
//! ]);
//! let previous = neighbour(&document, Direction::Previous, "two.html", &names, &routes).unwrap();
//! assert_eq!((previous.route, previous.title), ("one.html", "Book/Chapter 1"));
//! assert!(neighbour(&document, Direction::Next, "two.html", &names, &routes).is_none());
//! ```
use std::collections::BTreeMap;

use crate::{
    ast::{Block, Inline, Url},
    garden::GardenDocument,
    page_names::PageNameIndex,
};

/// Which neighbour of a page: the one `prev::` names or the one `next::` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// The page that `prev::` names.
    Previous,
    /// The page that `next::` names.
    Next,
}

impl Direction {
    /// Both directions, previous first.
    pub const ALL: [Self; 2] = [Self::Previous, Self::Next];

    /// The property name, which is also the link's `rel` value: `prev` or `next`.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Previous => "prev",
            Self::Next => "next",
        }
    }

    /// The direction whose property is named `key`, in any case.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|direction| direction.key().eq_ignore_ascii_case(key))
    }
}

/// A published neighbour: its route and its title, both borrowed from the routes map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbour<'a> {
    /// The neighbour's output route, such as `p-0123456789abcdef.html`.
    pub route: &'a str,
    /// The neighbour's page title.
    pub title: &'a str,
}

/// The page name that the page's last leading property for `direction` names, when its whole
/// value is one page reference; `None` otherwise. The name is as written, before lookup.
#[must_use]
pub fn written_target(document: &GardenDocument, direction: Direction) -> Option<String> {
    let Block::Properties { props, .. } = document.parsed.blocks.first()? else {
        return None;
    };
    let value = &props
        .iter()
        .rev()
        .find(|property| property.0.eq_ignore_ascii_case(direction.key()))?
        .1;
    let value = value.trim();
    if !value.starts_with("[[") || !value.ends_with("]]") {
        return None;
    }
    let parsed = lsdoc::inline(value, "markdown");
    match parsed.as_slice() {
        [
            Inline::Link {
                url: Url::PageRef { v },
                image: false,
                ..
            },
        ] => Some(v.clone()),
        _ => None,
    }
}

/// The published page that the page at `current_route` names for `direction`.
///
/// `names` finds published pages by title or alias, and `routes` gives each published route
/// its title. The neighbour is `None` when the property names no single published page, or
/// names the page itself.
#[must_use]
pub fn neighbour<'a>(
    document: &GardenDocument,
    direction: Direction,
    current_route: &str,
    names: &PageNameIndex<String>,
    routes: &'a BTreeMap<String, String>,
) -> Option<Neighbour<'a>> {
    let target = written_target(document, direction)?;
    let [route] = names.candidates(&target) else {
        return None;
    };
    if route == current_route {
        return None;
    }
    let (route, title) = routes.get_key_value(route)?;
    Some(Neighbour { route, title })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;

    fn document(path: &str, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: path.into(),
            kind: DocumentKind::Page,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
            title_spellings: Default::default(),
        }
    }

    #[test]
    fn sequence_directions_round_trip_their_keys_in_any_case() {
        for direction in Direction::ALL {
            assert_eq!(Direction::from_key(direction.key()), Some(direction));
        }
        assert_eq!(Direction::from_key("PrEv"), Some(Direction::Previous));
        assert_eq!(Direction::from_key("NEXT"), Some(Direction::Next));
        assert_eq!(Direction::from_key("previous"), None);
    }

    #[test]
    fn sequence_neighbours_resolve_through_names_and_skip_self_and_unknown_routes() {
        let page = document("pages/B.md", "prev:: [[a alias]]\nnext:: [[B]]\n\n- Body\n");
        let a = document("pages/A.md", "alias:: [[A alias]]\n\n- A\n");
        let mut names = PageNameIndex::default();
        names.insert("a.html".to_owned(), "A", &a);
        names.insert("b.html".to_owned(), "B", &page);
        let routes = BTreeMap::from([
            ("a.html".to_owned(), "A".to_owned()),
            ("b.html".to_owned(), "B".to_owned()),
        ]);
        assert_eq!(
            neighbour(&page, Direction::Previous, "b.html", &names, &routes),
            Some(Neighbour {
                route: "a.html",
                title: "A"
            })
        );
        assert_eq!(
            neighbour(&page, Direction::Next, "b.html", &names, &routes),
            None
        );
        // A route that names no published title gives no neighbour.
        let empty = BTreeMap::new();
        assert_eq!(
            neighbour(&page, Direction::Previous, "b.html", &names, &empty),
            None
        );
    }
}
