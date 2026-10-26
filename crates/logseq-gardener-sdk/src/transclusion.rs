//! Block references and embeds as Logseq shows them: what a reference displays, what
//! an embed holds, and the nesting limit references and embeds share.
//!
//! ```
//! use logseq_gardener_sdk::transclusion::{Nesting, ReferenceOutcome, ReferenceTargets};
//!
//! let page = lsdoc::parse_to_projection(
//!     "- DONE ship [[Release]]\n  id:: 6512a1b2-0000-4000-8000-000000000001\n\t- notes\n",
//! );
//! let mut targets = ReferenceTargets::default();
//! targets.add_page("release-plan".to_owned(), &page.blocks);
//! let ReferenceOutcome::Resolved(target) =
//!     targets.reference("6512A1B2-0000-4000-8000-000000000001", &[])
//! else {
//!     panic!("the reference resolves");
//! };
//! assert_eq!(target.page, "release-plan");
//! assert!(target.content.marker.is_some());
//! assert!(matches!(
//!     targets.reference("6512a1b2-0000-4000-8000-000000000001", &vec![Nesting::Reference; 5]),
//!     ReferenceOutcome::TooDeep
//! ));
//! ```
use std::collections::BTreeMap;

use crate::ast::{Block, Inline};
use crate::garden::GardenDocument;
use crate::outline::{Outline, TaskMarker, bullet_property, uuid};
use crate::page_names::PageNameIndex;

/// Logseq stops expanding when a reference or embed would be the sixth level.
pub const NESTING_LIMIT: usize = 5;

/// What a block reference shows: the target's first line, its task marker and its
/// heading level. When the first line is empty, as for a bullet whose content is a quote,
/// Logseq shows the block's body instead. Properties and children are left out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReferenceContent {
    pub marker: Option<TaskMarker>,
    pub heading: Option<u8>,
    pub inline: Vec<Inline>,
    /// The blocks after the first line, without properties; empty unless `inline` is.
    pub body: Vec<Block>,
}

/// A published block that references and block embeds can name.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTarget<K> {
    /// The page that owns the block, in the caller's own identifier.
    pub page: K,
    /// Byte offset of the block's bullet in the page source.
    pub start: usize,
    pub content: ReferenceContent,
}

/// One level of the chain of references and embeds being expanded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nesting<K> {
    Reference,
    /// A block embed (`start` is the bullet) or a page embed (`start` is `None`).
    Embed {
        page: K,
        start: Option<usize>,
    },
}

/// Whether a reference or an embed reached the nesting limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transclusion {
    Reference,
    Embed,
}

#[derive(Debug, PartialEq)]
pub enum ReferenceOutcome<'a, K> {
    Resolved(&'a ReferenceTarget<K>),
    /// Unknown, malformed, withheld, or owned by more than one published block.
    Invalid,
    TooDeep,
}

/// The block UUIDs of the published pages. A UUID with more than one owner names
/// no block.
#[derive(Clone, Debug)]
pub struct ReferenceTargets<K> {
    targets: BTreeMap<String, Option<ReferenceTarget<K>>>,
}

impl<K> Default for ReferenceTargets<K> {
    fn default() -> Self {
        Self {
            targets: BTreeMap::new(),
        }
    }
}

impl<K: Clone + PartialEq> ReferenceTargets<K> {
    /// Register the outline blocks of one published page.
    pub fn add_page(&mut self, page: K, blocks: &[Block]) {
        for (id, start, content) in page_targets(blocks) {
            let target = ReferenceTarget {
                page: page.clone(),
                start,
                content,
            };
            self.targets
                .entry(id)
                .and_modify(|value| *value = None)
                .or_insert(Some(target));
        }
    }

    /// The block a UUID names, in any case; `None` when no single published block owns it.
    pub fn get(&self, id: &str) -> Option<&ReferenceTarget<K>> {
        self.targets.get(&uuid(id)?)?.as_ref()
    }

    /// Whether the block at `start` on `page` is the one owner of `id`.
    pub fn owns(&self, id: &str, page: &K, start: usize) -> bool {
        self.get(id)
            .is_some_and(|target| &target.page == page && target.start == start)
    }

    /// Resolve a block reference inside `chain`, the references and embeds around it.
    pub fn reference(&self, id: &str, chain: &[Nesting<K>]) -> ReferenceOutcome<'_, K> {
        if chain.len() >= NESTING_LIMIT {
            return ReferenceOutcome::TooDeep;
        }
        self.get(id)
            .map_or(ReferenceOutcome::Invalid, ReferenceOutcome::Resolved)
    }
}

/// The blocks of a page that can be targets: `(uuid, bullet start, display content)`.
/// A bullet is a target when it carries a valid `id::`, `custom-id::` or `custom_id::`.
pub fn page_targets(blocks: &[Block]) -> Vec<(String, usize, ReferenceContent)> {
    fn visit(blocks: &[Block], result: &mut Vec<(String, usize, ReferenceContent)>, depth: usize) {
        if depth > 64 {
            return;
        }
        let outline = Outline::from_blocks(blocks);
        for item in &outline.blocks {
            let Block::Bullet {
                inline,
                span: Some(span),
                ..
            } = &blocks[item.index]
            else {
                continue;
            };
            let id = ["custom-id", "custom_id", "id"]
                .into_iter()
                .find_map(|key| bullet_property(blocks, item.index, key))
                .and_then(uuid);
            if let Some(id) = id {
                let content = ReferenceContent {
                    marker: item.marker,
                    heading: item.heading,
                    inline: inline.clone(),
                    body: body(blocks, item.index, inline),
                };
                result.push((id, span.0, content));
            }
        }
        for block in blocks {
            if let Block::Quote { children, .. } | Block::Custom { children, .. } = block {
                visit(children, result, depth + 1);
            }
        }
    }
    let mut result = Vec::new();
    visit(blocks, &mut result, 0);
    result.sort_by_key(|(_, start, _)| *start);
    result
}

/// What a block reference shows for the bullet at `index`.
pub fn reference_content(blocks: &[Block], index: usize) -> Option<ReferenceContent> {
    let Block::Bullet { inline, .. } = blocks.get(index)? else {
        return None;
    };
    let outline = Outline::from_blocks(blocks);
    let item = outline.blocks.iter().find(|item| item.index == index)?;
    Some(ReferenceContent {
        marker: item.marker,
        heading: item.heading,
        inline: inline.clone(),
        body: body(blocks, index, inline),
    })
}

/// The body Logseq shows for a reference whose first line is empty.
fn body(blocks: &[Block], index: usize, inline: &[Inline]) -> Vec<Block> {
    if !inline.is_empty() {
        return Vec::new();
    }
    blocks[index + 1..]
        .iter()
        .take_while(|block| !matches!(block, Block::Bullet { .. }))
        .filter(|block| !matches!(block, Block::Properties { .. }))
        .cloned()
        .collect()
}

/// A published page that embeds can show.
#[derive(Clone, Copy, Debug)]
pub struct EmbedPage<'a> {
    pub title: &'a str,
    pub document: &'a GardenDocument,
}

/// The published pages and blocks embeds resolve against.
pub struct Embeds<'a, K> {
    pub pages: &'a PageNameIndex<K>,
    pub targets: &'a ReferenceTargets<K>,
    pub documents: &'a BTreeMap<K, EmbedPage<'a>>,
}

#[derive(Debug)]
pub enum EmbedOutcome<'a, K> {
    /// The embedded blocks with their descendants, in source order. `title` is set for
    /// a page embed.
    Outline {
        page: K,
        title: Option<&'a str>,
        start: Option<usize>,
        document: &'a GardenDocument,
        blocks: &'a [Block],
    },
    /// A block embed of an unknown, withheld or ambiguous block, or an argument that
    /// names neither a block nor a page.
    Unavailable,
    /// A page embed of a page with no published file; `name` is the macro's own text.
    MissingPage {
        name: String,
    },
    /// A page embed of the page itself or of a page already being embedded.
    SelfEmbed {
        page: K,
        title: &'a str,
    },
    TooDeep(Transclusion),
}

impl<'a, K: Clone + Ord> Embeds<'a, K> {
    /// Resolve the argument of `{{embed …}}` on `page`, inside `chain`.
    pub fn embed(&self, argument: &str, page: &K, chain: &[Nesting<K>]) -> EmbedOutcome<'a, K> {
        let argument = argument.trim();
        if chain.len() >= NESTING_LIMIT {
            return EmbedOutcome::TooDeep(Transclusion::Embed);
        }
        if let Some(name) = argument
            .strip_prefix("[[")
            .and_then(|v| v.strip_suffix("]]"))
        {
            let [route] = self.pages.candidates(name) else {
                return EmbedOutcome::MissingPage {
                    name: name.to_owned(),
                };
            };
            let Some(embedded) = self.documents.get(route) else {
                return EmbedOutcome::MissingPage {
                    name: name.to_owned(),
                };
            };
            let enclosing = chain.iter().any(
                |level| matches!(level, Nesting::Embed { page, start: None } if page == route),
            );
            if route == page || enclosing {
                return EmbedOutcome::SelfEmbed {
                    page: route.clone(),
                    title: embedded.title,
                };
            }
            return EmbedOutcome::Outline {
                page: route.clone(),
                title: Some(embedded.title),
                start: None,
                document: embedded.document,
                blocks: &embedded.document.parsed.blocks,
            };
        }
        let Some(target) = argument
            .strip_prefix("((")
            .and_then(|v| v.strip_suffix("))"))
            .and_then(|id| self.targets.get(id))
        else {
            return EmbedOutcome::Unavailable;
        };
        let Some(embedded) = self.documents.get(&target.page) else {
            return EmbedOutcome::Unavailable;
        };
        match outline_subtree(&embedded.document.parsed.blocks, target.start, 0) {
            Some(blocks) => EmbedOutcome::Outline {
                page: target.page.clone(),
                title: None,
                start: Some(target.start),
                document: embedded.document,
                blocks,
            },
            None => EmbedOutcome::Unavailable,
        }
    }
}

/// The bullet starting at `start` with its continuations and descendants. The parser
/// stores outline bullets flat, so the subtree runs to the next bullet at its level or above.
pub fn outline_subtree(blocks: &[Block], start: usize, depth: usize) -> Option<&[Block]> {
    if depth > 64 {
        return None;
    }
    for (index, block) in blocks.iter().enumerate() {
        if let Block::Bullet {
            span: Some(span),
            level,
            ..
        } = block
            && span.0 == start
        {
            let end = blocks[index + 1..]
                .iter()
                .position(
                    |block| matches!(block, Block::Bullet { level: next, .. } if next <= level),
                )
                .map_or(blocks.len(), |offset| index + 1 + offset);
            return Some(&blocks[index..end]);
        }
        if let Block::Quote { children, .. } | Block::Custom { children, .. } = block
            && let Some(found) = outline_subtree(children, start, depth + 1)
        {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::{DocumentKind, GardenDocument};

    const A: &str = "6512a1b2-0000-4000-8000-00000000000a";
    const B: &str = "6512a1b2-0000-4000-8000-00000000000b";

    fn blocks(source: &str) -> Vec<Block> {
        lsdoc::parse_to_projection(source).blocks
    }

    fn document(source: &str) -> GardenDocument {
        GardenDocument {
            kind: DocumentKind::Page,
            relative_path: "pages/test.md".into(),
            source: source.into(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    fn plain(nodes: &[Inline]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                Inline::Plain { text, .. } => text.clone(),
                Inline::Link { full, .. } => full.clone(),
                _ => String::new(),
            })
            .collect()
    }

    #[test]
    fn quote_bullets_are_targets_and_duplicates_name_nothing() {
        let page = blocks(&format!(
            "- > a quote\n  id:: {A}\n- first\n  id:: {B}\n- again\n  id:: {B}\n"
        ));
        let mut targets = ReferenceTargets::default();
        targets.add_page("p".to_owned(), &page);
        let quote = targets.get(&A.to_ascii_uppercase()).unwrap();
        assert!(quote.content.inline.is_empty());
        assert!(matches!(
            quote.content.body.as_slice(),
            [Block::Quote { .. }]
        ));
        assert!(targets.get(B).is_none_or(|t| t.content.body.is_empty()));
        assert!(targets.get(B).is_none());
        assert_eq!(targets.reference(B, &[]), ReferenceOutcome::Invalid);
        assert!(targets.owns(A, &"p".to_owned(), quote.start));
    }

    #[test]
    fn duplicates_across_pages_name_nothing() {
        let mut targets = ReferenceTargets::default();
        targets.add_page(1, &blocks(&format!("- one\n  id:: {A}\n")));
        targets.add_page(2, &blocks(&format!("- two\n  id:: {A}\n")));
        assert!(targets.get(A).is_none());
    }

    #[test]
    fn content_is_the_first_line_with_marker_and_heading() {
        let page = blocks(&format!(
            "- DONE figure out [[Bug]]\n  id:: {A}\n  collapsed:: true\n\t- child\n- ## Setup\n  id:: {B}\n"
        ));
        let mut targets = ReferenceTargets::default();
        targets.add_page((), &page);
        let done = &targets.get(A).unwrap().content;
        assert_eq!(done.marker, Some(TaskMarker::Done));
        assert_eq!(plain(&done.inline), "figure out [[Bug]]");
        assert!(matches!(done.inline[1], Inline::Link { .. }));
        assert_eq!(targets.get(B).unwrap().content.heading, Some(2));
        let multi = blocks("- title\n  second line\n\t- child\n");
        let content = reference_content(&multi, 0).unwrap();
        assert_eq!(plain(&content.inline), "title");
        assert!(content.body.is_empty());
        assert_eq!(reference_content(&multi, 1), None);
    }

    fn embeds_fixture(
        sources: &[(&str, &str)],
    ) -> (
        PageNameIndex<String>,
        ReferenceTargets<String>,
        Vec<(String, GardenDocument)>,
    ) {
        let mut pages = PageNameIndex::default();
        let mut targets = ReferenceTargets::default();
        let mut documents = Vec::new();
        for (title, source) in sources {
            let document = document(source);
            pages.insert(title.to_string(), title, &document);
            targets.add_page(title.to_string(), &document.parsed.blocks);
            documents.push((title.to_string(), document));
        }
        (pages, targets, documents)
    }

    #[test]
    fn embeds_resolve_blocks_pages_and_the_self_check() {
        let (pages, targets, documents) = embeds_fixture(&[
            ("Home", "- {{embed [[Tool]]}}\n"),
            (
                "Tool",
                &format!("- intro\n- # Heading\n  id:: {A}\n\t- child\n- after\n"),
            ),
        ]);
        let documents: BTreeMap<_, _> = documents
            .iter()
            .map(|(title, document)| (title.clone(), EmbedPage { title, document }))
            .collect();
        let embeds = Embeds {
            pages: &pages,
            targets: &targets,
            documents: &documents,
        };
        let home = "Home".to_owned();
        let EmbedOutcome::Outline { blocks, title, .. } =
            embeds.embed(&format!("(({A}))"), &home, &[])
        else {
            panic!("block embed resolves");
        };
        assert_eq!(title, None);
        let Block::Bullet { inline, .. } = &blocks[0] else {
            panic!()
        };
        assert_eq!(plain(inline), "Heading");
        assert_eq!(
            blocks
                .iter()
                .filter(|b| matches!(b, Block::Bullet { .. }))
                .count(),
            2
        );
        let EmbedOutcome::Outline { title, blocks, .. } = embeds.embed("[[tool]]", &home, &[])
        else {
            panic!("page embed resolves");
        };
        assert_eq!(title, Some("Tool"));
        assert_eq!(
            blocks
                .iter()
                .filter(|b| matches!(b, Block::Bullet { .. }))
                .count(),
            4
        );
        assert!(matches!(
            embeds.embed("[[Home]]", &home, &[]),
            EmbedOutcome::SelfEmbed { title: "Home", .. }
        ));
        let inside_tool = [Nesting::Embed {
            page: "Tool".to_owned(),
            start: None,
        }];
        assert!(matches!(
            embeds.embed("[[Tool]]", &home, &inside_tool),
            EmbedOutcome::SelfEmbed { .. }
        ));
        assert!(matches!(
            embeds.embed("((6512a1b2-0000-4000-8000-0000000000ff))", &home, &[]),
            EmbedOutcome::Unavailable
        ));
        assert!(matches!(
            embeds.embed("plain words", &home, &[]),
            EmbedOutcome::Unavailable
        ));
        assert!(matches!(
            embeds.embed("[[Private]]", &home, &[]),
            EmbedOutcome::MissingPage { name } if name == "Private"
        ));
    }

    #[test]
    fn the_sixth_level_is_too_deep() {
        let names: Vec<String> = (1..=7).map(|n| format!("P{n}")).collect();
        let sources: Vec<(String, String)> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let next = names.get(i + 1).map_or(String::from("- end\n"), |next| {
                    format!("- {{{{embed [[{next}]]}}}}\n")
                });
                (name.clone(), next)
            })
            .collect();
        let borrowed: Vec<(&str, &str)> = sources
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let (pages, targets, documents) = embeds_fixture(&borrowed);
        let documents: BTreeMap<_, _> = documents
            .iter()
            .map(|(title, document)| (title.clone(), EmbedPage { title, document }))
            .collect();
        let embeds = Embeds {
            pages: &pages,
            targets: &targets,
            documents: &documents,
        };
        let mut chain = Vec::new();
        let mut expanded = 0;
        for name in &names[1..] {
            match embeds.embed(&format!("[[{name}]]"), &names[0], &chain) {
                EmbedOutcome::Outline { page, .. } => {
                    expanded += 1;
                    chain.push(Nesting::Embed { page, start: None });
                }
                EmbedOutcome::TooDeep(kind) => {
                    assert_eq!(kind, Transclusion::Embed);
                    break;
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(expanded, 5);
        assert_eq!(targets.reference(A, &chain), ReferenceOutcome::TooDeep);
    }
}
