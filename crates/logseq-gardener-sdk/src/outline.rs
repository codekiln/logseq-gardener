//! The blocks of one page's outline: depth, ancestry, task markers, headings,
//! authored collapse and identifiers that stay the same across builds.
//!
//! ```
//! use logseq_gardener_sdk::outline::{BlockId, Outline};
//!
//! let parsed = lsdoc::parse_to_projection("- Setup\n\t- TODO install\n");
//! let outline = Outline::from_blocks(&parsed.blocks);
//! let child = outline.find(&BlockId::parse("1.1").unwrap()).unwrap();
//! assert_eq!(outline.ancestors(child).count(), 1);
//! ```
use std::fmt;

use crate::ast::Block;

/// A task marker at the start of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskMarker {
    Todo,
    Doing,
    Now,
    Later,
    Wait,
    Waiting,
    InProgress,
    Done,
    /// `CANCELED` or `CANCELLED`.
    Canceled,
}

impl TaskMarker {
    /// Read a marker word as the parser reports it; unknown words give `None`.
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word.trim().to_ascii_uppercase().as_str() {
            "TODO" => Self::Todo,
            "DOING" => Self::Doing,
            "NOW" => Self::Now,
            "LATER" => Self::Later,
            "WAIT" => Self::Wait,
            "WAITING" => Self::Waiting,
            "IN-PROGRESS" => Self::InProgress,
            "DONE" => Self::Done,
            "CANCELED" | "CANCELLED" => Self::Canceled,
            _ => return None,
        })
    }

    /// The marker word Logseq shows.
    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Doing => "DOING",
            Self::Now => "NOW",
            Self::Later => "LATER",
            Self::Wait => "WAIT",
            Self::Waiting => "WAITING",
            Self::InProgress => "IN-PROGRESS",
            Self::Done => "DONE",
            Self::Canceled => "CANCELED",
        }
    }
}

/// A block's identifier: its `id::` UUID, otherwise its position path.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    /// A valid hyphenated UUID, lower-cased.
    Uuid(String),
    /// 1-based positions among siblings from the top level down, written `3.1.2`.
    Path(Vec<u32>),
}

impl BlockId {
    /// Read a UUID (any case) or a dotted path of positive integers.
    pub fn parse(text: &str) -> Option<Self> {
        if let Some(id) = uuid(text) {
            return Some(Self::Uuid(id));
        }
        let path = text
            .split('.')
            .map(|part| {
                (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                    .then(|| part.parse::<u32>().ok())
                    .flatten()
                    .filter(|n| *n > 0)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self::Path(path))
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid(id) => f.write_str(id),
            Self::Path(path) => {
                for (i, n) in path.iter().enumerate() {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    write!(f, "{n}")?;
                }
                Ok(())
            }
        }
    }
}

/// One block of an outline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutlineBlock {
    /// 1 at the top level.
    pub depth: usize,
    /// Index of the parent in [`Outline::blocks`].
    pub parent: Option<usize>,
    pub has_children: bool,
    pub marker: Option<TaskMarker>,
    /// The priority letter, such as `A`.
    pub priority: Option<String>,
    /// `h1`–`h6` from leading `#` or the `heading::` property.
    pub heading: Option<u8>,
    /// `collapsed:: true` on a block that has children.
    pub collapsed: bool,
    pub id: BlockId,
    /// The position path, also for blocks identified by UUID.
    pub path: Vec<u32>,
    /// Byte offset of the bullet in the page source, when the parser reports it.
    pub start: Option<usize>,
    /// Index of the bullet in the slice given to [`Outline::from_blocks`].
    pub index: usize,
}

/// The bullets of one page in document order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outline {
    pub blocks: Vec<OutlineBlock>,
}

impl Outline {
    /// Read the parser's flat bullet list. A bullet's parent is the nearest earlier
    /// bullet with a lower level, so skipped indentation levels add no depth.
    pub fn from_blocks(source: &[Block]) -> Self {
        let mut blocks: Vec<OutlineBlock> = Vec::new();
        let mut levels: Vec<(u32, usize)> = Vec::new();
        let mut child_counts: Vec<u32> = Vec::new();
        let mut top_count = 0;
        for (index, block) in source.iter().enumerate() {
            let Block::Bullet {
                level,
                size,
                marker,
                priority,
                span,
                ..
            } = block
            else {
                continue;
            };
            while levels.last().is_some_and(|(previous, _)| previous >= level) {
                levels.pop();
            }
            let parent = levels.last().map(|(_, parent)| *parent);
            let position = match parent {
                Some(parent) => {
                    blocks[parent].has_children = true;
                    child_counts[parent] += 1;
                    child_counts[parent]
                }
                None => {
                    top_count += 1;
                    top_count
                }
            };
            let depth = levels.len() + 1;
            let property = |key: &str| bullet_property(source, index, key);
            let heading = size.map(|n| n.clamp(1, 6) as u8).or_else(|| {
                property("heading").and_then(|value| {
                    if value.eq_ignore_ascii_case("true") {
                        Some(depth.saturating_add(1).min(6) as u8)
                    } else {
                        value.parse::<u8>().ok().filter(|n| (1..=6).contains(n))
                    }
                })
            });
            let mut path = parent.map(|p| blocks[p].path.clone()).unwrap_or_default();
            path.push(position);
            let id = ["custom-id", "custom_id", "id"]
                .into_iter()
                .find_map(property)
                .and_then(uuid)
                .map_or_else(|| BlockId::Path(path.clone()), BlockId::Uuid);
            blocks.push(OutlineBlock {
                depth,
                parent,
                has_children: false,
                marker: marker.as_deref().and_then(TaskMarker::parse),
                priority: priority.clone(),
                heading,
                collapsed: property("collapsed").is_some_and(|v| v.eq_ignore_ascii_case("true")),
                id,
                start: span.as_ref().map(|span| span.0),
                index,
                path,
            });
            child_counts.push(0);
            levels.push((*level, blocks.len() - 1));
        }
        for block in &mut blocks {
            block.collapsed &= block.has_children;
        }
        Self { blocks }
    }

    /// The block an identifier names.
    pub fn find(&self, id: &BlockId) -> Option<usize> {
        self.blocks.iter().position(|block| match id {
            BlockId::Uuid(_) => &block.id == id,
            BlockId::Path(path) => &block.path == path,
        })
    }

    /// The block whose bullet starts at a source byte offset.
    pub fn at_start(&self, start: usize) -> Option<&OutlineBlock> {
        self.blocks.iter().find(|block| block.start == Some(start))
    }

    /// Ancestors from the parent up to the top level.
    pub fn ancestors(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.blocks.get(index).and_then(|b| b.parent), |i| {
            self.blocks[*i].parent
        })
    }
}

/// A property of the bullet at `index`, from the block after it, or after its quote
/// when the bullet's content is a quote (`- > …` with `id::` on the next line).
pub(crate) fn bullet_property<'a>(source: &'a [Block], index: usize, key: &str) -> Option<&'a str> {
    let next = match (source.get(index), source.get(index + 1)) {
        (Some(Block::Bullet { inline, .. }), Some(Block::Quote { .. })) if inline.is_empty() => {
            index + 2
        }
        _ => index + 1,
    };
    let Some(Block::Properties { props, .. }) = source.get(next) else {
        return None;
    };
    props
        .iter()
        .rev()
        .find(|p| p.0.trim().eq_ignore_ascii_case(key))
        .map(|p| p.1.trim())
}

pub(crate) fn uuid(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() == 36
        && value.bytes().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        }))
    .then(|| value.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outline(source: &str) -> Outline {
        Outline::from_blocks(&lsdoc::parse_to_projection(source).blocks)
    }

    #[test]
    fn nested_headings_and_tasks() {
        let outline = outline("- # Setup\n\t- TODO [#A] install\n\t\t- DONE read docs\n");
        let depths: Vec<_> = outline.blocks.iter().map(|b| b.depth).collect();
        assert_eq!(depths, [1, 2, 3]);
        assert_eq!(outline.blocks[0].heading, Some(1));
        assert_eq!(outline.blocks[1].marker, Some(TaskMarker::Todo));
        assert_eq!(outline.blocks[1].priority.as_deref(), Some("A"));
        assert_eq!(outline.blocks[1].parent, Some(0));
        assert_eq!(outline.blocks[2].marker, Some(TaskMarker::Done));
        assert_eq!(outline.blocks[2].parent, Some(1));
        assert!(outline.blocks[0].has_children && !outline.blocks[2].has_children);
    }

    #[test]
    fn heading_property() {
        let outline = outline(
            "- top\n  heading:: true\n\t- a\n\t\t- b\n\t\t\t- c\n\t\t\t\t- d\n\t\t\t\t\t- six\n\t\t\t\t\t  heading:: true\n- three\n  heading:: 3\n- seven\n  heading:: 7\n",
        );
        let headings: Vec<_> = outline.blocks.iter().map(|b| b.heading).collect();
        assert_eq!(
            headings,
            [Some(2), None, None, None, None, Some(6), Some(3), None]
        );
    }

    #[test]
    fn markers() {
        assert_eq!(TaskMarker::parse("CANCELLED"), Some(TaskMarker::Canceled));
        assert_eq!(
            TaskMarker::parse("in-progress"),
            Some(TaskMarker::InProgress)
        );
        assert_eq!(TaskMarker::parse("MAYBE"), None);
    }

    #[test]
    fn collapsed_needs_children() {
        let outline = outline(
            "- leaf\n  collapsed:: true\n- parent\n  collapsed:: true\n\t- child\n- open\n  collapsed:: false\n\t- child\n",
        );
        let collapsed: Vec<_> = outline.blocks.iter().map(|b| b.collapsed).collect();
        assert_eq!(collapsed, [false, true, false, false, false]);
    }

    #[test]
    fn position_paths_and_uuids() {
        let source = "- one\n- two\n- three\n\t- a\n\t- b\n\t  id:: 6ADD3FFB-E52B-4E8F-908A-9958186E4DFB\n\t\t- deep\n";
        let first = outline(source);
        assert_eq!(first, outline(source));
        let a = first.find(&BlockId::parse("3.1").unwrap()).unwrap();
        assert_eq!(first.blocks[a].id.to_string(), "3.1");
        assert_eq!(first.ancestors(a).collect::<Vec<_>>(), [2]);
        let b = first
            .find(&BlockId::parse("6add3ffb-e52b-4e8f-908a-9958186e4dfb").unwrap())
            .unwrap();
        assert_eq!(
            first.find(&BlockId::parse("6ADD3FFB-E52B-4E8F-908A-9958186E4DFB").unwrap()),
            Some(b)
        );
        assert_eq!(first.blocks[b].path, [3, 2]);
        let deep = first.find(&BlockId::parse("3.2.1").unwrap()).unwrap();
        assert_eq!(first.ancestors(deep).collect::<Vec<_>>(), [b, 2]);
    }

    #[test]
    fn skipped_levels_add_no_depth() {
        let outline = outline("- one\n\t\t- skipped\n\t- two\n");
        assert_eq!(outline.blocks[1].depth, 2);
        assert_eq!(outline.blocks[2].id, BlockId::Path(vec![1, 2]));
    }

    #[test]
    fn unknown_identifiers() {
        let outline = outline("- one\n");
        assert_eq!(outline.find(&BlockId::parse("2").unwrap()), None);
        for text in ["", "1..2", "0", "a.1", "1.", "6add3ffb-e52b-4e8f-908a"] {
            assert_eq!(BlockId::parse(text), None, "{text}");
        }
    }
}
