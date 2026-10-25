//! The page a tag points at, shared by the publisher, search and later the CLI.
//!
//! Resolve the returned name with [`crate::page_names::PageNameIndex::candidates`],
//! as page references are resolved, so titles, aliases, case and journal dates
//! agree for both.
//!
//! ```
//! use logseq_gardener_sdk::ast::Inline;
//! use logseq_gardener_sdk::inline_references::tag_page_name;
//!
//! let children = vec![Inline::Plain {
//!     text: "Keyshort".into(),
//!     span: None,
//!     span_map: None,
//! }];
//! assert_eq!(tag_page_name(&children), "Keyshort");
//! ```

use crate::ast::{Inline, Url};

/// Return the page name a parsed tag's children name: `Keyshort` for `#Keyshort`
/// and `multi word` for `#[[multi word]]`.
pub fn tag_page_name(children: &[Inline]) -> String {
    let mut name = String::new();
    collect(children, &mut name, 0);
    name
}

fn collect(nodes: &[Inline], name: &mut String, depth: usize) {
    if depth > 64 {
        return;
    }
    for node in nodes {
        match node {
            Inline::Plain { text, .. }
            | Inline::Code { text, .. }
            | Inline::Verbatim { text, .. }
            | Inline::Target { text, .. } => name.push_str(text),
            Inline::NestedLink { content, .. } => name.push_str(content),
            Inline::Link {
                url: Url::PageRef { v },
                label,
                ..
            } if label.is_empty() => name.push_str(v),
            Inline::Link { label, .. } => collect(label, name, depth + 1),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. }
            | Inline::Tag { children, .. } => collect(children, name, depth + 1),
            Inline::Entity { unicode, .. } => name.push_str(unicode),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Block;
    use crate::garden::{DocumentKind, GardenDocument};
    use crate::page_names::PageNameIndex;

    fn inlines(source: &str) -> Vec<Inline> {
        let parsed = lsdoc::parse_to_projection(source);
        parsed
            .blocks
            .into_iter()
            .flat_map(|block| match block {
                Block::Paragraph { inline, .. } => inline,
                _ => Vec::new(),
            })
            .collect()
    }

    fn tag_names(source: &str) -> Vec<String> {
        inlines(source)
            .iter()
            .filter_map(|node| match node {
                Inline::Tag { children, .. } => Some(tag_page_name(children)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn simple_and_bracketed_tags() {
        assert_eq!(
            tag_names("#Keyshort and #[[multi word]]"),
            ["Keyshort", "multi word"]
        );
    }

    #[test]
    fn tag_and_reference_resolve_to_the_same_aliased_page() {
        let source = "alias:: shortcut\n\n- body";
        let document = GardenDocument {
            relative_path: "pages/Keyshort.md".into(),
            kind: DocumentKind::Page,
            source: source.into(),
            parsed: lsdoc::parse_to_projection(source),
        };
        let mut names = PageNameIndex::default();
        names.insert("p-keyshort.html".to_owned(), "Keyshort", &document);
        let tag = tag_names("#Shortcut").remove(0);
        let reference = inlines("[[shortcut]]")
            .into_iter()
            .find_map(|node| match node {
                Inline::Link {
                    url: Url::PageRef { v },
                    ..
                } => Some(v),
                _ => None,
            })
            .unwrap();
        assert_eq!(names.candidates(&tag), ["p-keyshort.html"]);
        assert_eq!(names.candidates(&tag), names.candidates(&reference));
    }
}
