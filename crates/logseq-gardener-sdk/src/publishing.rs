//! Namespace selection for publishing logical garden pages.
//!
//! Selection uses exact, case-sensitive logical names. Resolve filenames,
//! aliases, and graph naming rules before applying this policy. A selected page
//! still needs publication-property checks, and every referenced page or asset
//! needs its own visibility check before its content enters the site.

use std::fmt;

/// Namespace roots to include and exclude from a publication.
///
/// Empty inclusion selects nothing. Exclusion always wins. A root matches
/// itself and slash-separated descendants, but not names with a similar prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceSelection {
    included: Vec<String>,
    excluded: Vec<String>,
}

/// The selector list containing an invalid namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorList {
    /// Roots requested for inclusion.
    Include,
    /// Roots requested for exclusion.
    Exclude,
}

/// An invalid selector, identified without disclosing its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNamespace {
    /// The list containing the invalid selector.
    pub list: SelectorList,
    /// The zero-based position in that list.
    pub index: usize,
}

impl fmt::Display for InvalidNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = match self.list {
            SelectorList::Include => "include",
            SelectorList::Exclude => "exclude",
        };
        write!(
            f,
            "invalid namespace in {list} selector at index {}",
            self.index
        )
    }
}

impl std::error::Error for InvalidNamespace {}

impl NamespaceSelection {
    /// Build a policy from logical namespace roots.
    ///
    /// # Errors
    ///
    /// Returns the list and position of the first malformed selector. Each
    /// slash-separated segment must be nonempty, have no surrounding whitespace,
    /// and contain no control characters. Inputs are preserved without trimming
    /// or case conversion. Duplicate roots are allowed.
    pub fn new(included: &[&str], excluded: &[&str]) -> Result<Self, InvalidNamespace> {
        for (list, roots) in [
            (SelectorList::Include, included),
            (SelectorList::Exclude, excluded),
        ] {
            for (index, root) in roots.iter().enumerate() {
                if !valid_name(root) {
                    return Err(InvalidNamespace { list, index });
                }
            }
        }
        Ok(Self {
            included: included.iter().map(|root| (*root).to_owned()).collect(),
            excluded: excluded.iter().map(|root| (*root).to_owned()).collect(),
        })
    }

    /// Whether a valid logical page name is included and not excluded.
    ///
    /// Malformed candidates return false. This method does not load or modify
    /// garden files, evaluate page properties, or expand references.
    #[must_use]
    pub fn includes(&self, page_name: &str) -> bool {
        valid_name(page_name)
            && self
                .included
                .iter()
                .any(|root| in_namespace(page_name, root))
            && !self
                .excluded
                .iter()
                .any(|root| in_namespace(page_name, root))
    }
}

fn valid_name(name: &str) -> bool {
    name.split('/').all(|segment| {
        !segment.is_empty() && segment.trim() == segment && !segment.chars().any(char::is_control)
    })
}

fn in_namespace(page_name: &str, root: &str) -> bool {
    page_name == root
        || page_name
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_roots_and_descendants_at_slash_boundaries() {
        let policy = NamespaceSelection::new(&["My/AI", "研究/庭"], &[]).unwrap();
        for name in ["My/AI", "My/AI/Rule", "研究/庭", "研究/庭/花"] {
            assert!(policy.includes(name), "{name}");
        }
        for name in ["My", "My/AIM", "My/AI-other", "my/ai", "研究/庭園"] {
            assert!(!policy.includes(name), "{name}");
        }
    }

    #[test]
    fn exclusion_wins_over_more_specific_inclusion() {
        let policy =
            NamespaceSelection::new(&["My", "My/Private/Public"], &["My/Private"]).unwrap();
        for name in ["My/Private", "My/Private/Note", "My/Private/Public"] {
            assert!(!policy.includes(name), "{name}");
        }
        assert!(policy.includes("My/AI"));
        assert!(policy.includes("My/Privateer"));
    }

    #[test]
    fn empty_inclusion_selects_nothing() {
        for excluded in [&[][..], &["My/Private"][..]] {
            let policy = NamespaceSelection::new(&[], excluded).unwrap();
            assert!(!policy.includes("My"));
            assert!(!policy.includes("My/Private"));
        }
    }

    #[test]
    fn invalid_configuration_identifies_list_and_position_without_text() {
        for invalid in [
            "",
            "/My",
            "My/",
            "My//AI",
            " My",
            "My /AI",
            "My/ AI",
            "My\nAI",
            "My/\u{2003}AI",
        ] {
            for list in [SelectorList::Include, SelectorList::Exclude] {
                let roots = ["Valid", invalid];
                let result = match list {
                    SelectorList::Include => NamespaceSelection::new(&roots, &[]),
                    SelectorList::Exclude => NamespaceSelection::new(&["My"], &roots),
                };
                let error = result.unwrap_err();
                assert_eq!(error, InvalidNamespace { list, index: 1 });
                assert_eq!(
                    error.to_string(),
                    format!(
                        "invalid namespace in {} selector at index 1",
                        if list == SelectorList::Include {
                            "include"
                        } else {
                            "exclude"
                        }
                    )
                );
            }
        }
    }

    #[test]
    fn malformed_candidates_are_unselected() {
        let policy = NamespaceSelection::new(&["My"], &[]).unwrap();
        for name in ["My/", "My//AI", "My/ Secret", "My/AI\n", ""] {
            assert!(!policy.includes(name), "{name:?}");
        }
    }

    #[test]
    fn order_and_duplicates_do_not_change_selection() {
        let first = NamespaceSelection::new(&["My", "Research"], &["My/Private", "Research/Draft"])
            .unwrap();
        let second = NamespaceSelection::new(
            &["Research", "My", "My"],
            &["Research/Draft", "My/Private", "My/Private"],
        )
        .unwrap();
        for name in [
            "My",
            "My/AI",
            "My/Private",
            "Research",
            "Research/Draft/Note",
            "Other",
        ] {
            assert_eq!(first.includes(name), second.includes(name), "{name}");
        }
    }
}
