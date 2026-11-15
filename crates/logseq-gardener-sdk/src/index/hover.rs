//! What an editor shows when the pointer rests on a link: a summary of the page or block.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{
    BlockLocation, GardenIndex, IndexedDocument, PageId, SourceRange, Subject, Target, slashes,
};
use crate::ast::Block;
use crate::outline::Outline;
use crate::page_names::{page_alias_spellings, remove_boundary_slashes};
use crate::page_properties::is_hidden_block_property;

/// How many lines of a page or a block a summary shows.
pub const SUMMARY_LINES: usize = 5;

/// A page, as a hover shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSummary {
    /// The page's title, or the name as written for a page without a file.
    pub title: String,
    /// The page's file, or the file Logseq would create, relative to the garden root.
    pub path: Option<PathBuf>,
    /// Whether the index holds a document at `path`.
    pub exists: bool,
    /// The names in the page's `alias::` property, as written.
    pub aliases: Vec<String>,
    /// The first lines of the page after its properties, as written.
    pub first_lines: Vec<String>,
    /// How many places name the page, as [`GardenIndex::references`] lists them without the
    /// declaration.
    pub references: usize,
}

/// A block, as a hover shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSummary {
    /// The file that holds the block, relative to the garden root.
    pub path: PathBuf,
    /// The title of the page that holds the block.
    pub page: Option<String>,
    /// The first line of each parent block, outermost first.
    pub parents: Vec<String>,
    /// The block's lines without indentation, its dash and the properties Logseq hides.
    pub text: Vec<String>,
    /// How many block references and block embeds name the block.
    pub references: usize,
}

/// What a hover shows for the occurrence under the pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Summary {
    Page(PageSummary),
    /// A name that several pages answer to, with the title and path of each.
    Ambiguous(Vec<(String, PathBuf)>),
    Block(BlockSummary),
    /// A block id that no block writes.
    MissingBlock,
}

fn count(n: usize) -> String {
    match n {
        0 => "No references".to_owned(),
        1 => "1 reference".to_owned(),
        n => format!("{n} references"),
    }
}

impl Summary {
    /// The summary as Markdown. Paths use `/` on every system.
    pub fn markdown(&self) -> String {
        match self {
            Self::Page(page) => {
                let mut parts = vec![format!("**{}**", page.title)];
                if !page.aliases.is_empty() {
                    parts.push(format!("Aliases: {}", page.aliases.join(", ")));
                }
                if let (false, Some(path)) = (page.exists, &page.path) {
                    parts.push(format!(
                        "This page has no file. Go to definition opens `{}`.",
                        slashes(path)
                    ));
                }
                parts.push(count(page.references));
                if !page.first_lines.is_empty() {
                    parts.push("---".to_owned());
                    parts.push(page.first_lines.join("\n"));
                }
                parts.join("\n\n")
            }
            Self::Ambiguous(pages) => {
                let mut text = "Several pages answer to this name:\n".to_owned();
                for (title, path) in pages {
                    text.push_str(&format!("\n- **{title}** in `{}`", slashes(path)));
                }
                text
            }
            Self::Block(block) => {
                let page = block.page.clone().unwrap_or_else(|| slashes(&block.path));
                let mut trail = vec![format!("**{page}**")];
                trail.extend(block.parents.iter().cloned());
                [
                    trail.join(" › "),
                    block.text.join("\n"),
                    count(block.references),
                ]
                .join("\n\n")
            }
            Self::MissingBlock => "No block has this id.".to_owned(),
        }
    }
}

/// The lines of a document that write a property Logseq hides, such as `id::`.
fn hidden_lines(document: &IndexedDocument) -> BTreeSet<usize> {
    document
        .properties
        .iter()
        .filter(|line| is_hidden_block_property(&line.key))
        .map(|line| document.lines.position(line.key_range.start).line)
        .collect()
}

/// A line of a block without the block's indentation and its dash. `indent` is how much
/// white space starts the block's first line.
fn without_bullet(line: &str, indent: usize, first: bool) -> &str {
    let space = line.len() - line.trim_start().len();
    let line = &line[space.min(indent)..];
    if first {
        let line = line.strip_prefix('-').unwrap_or(line);
        line.strip_prefix(' ').unwrap_or(line)
    } else {
        let line = line.strip_prefix(' ').unwrap_or(line);
        line.strip_prefix(' ').unwrap_or(line)
    }
}

fn line_text(document: &IndexedDocument, line: usize) -> &str {
    let (start, end) = document.lines.line_range(line, document.source());
    &document.source()[start..end]
}

impl GardenIndex {
    /// A summary of the page a name leads to.
    pub fn page_summary(&self, name: &str) -> Summary {
        let page = self.page_id(name);
        let references = self.references(&Subject::Page(page.clone()), false).len();
        let document = match &page {
            PageId::Document(path) => self.documents.get(path),
            PageId::Ambiguous(_) => {
                return Summary::Ambiguous(
                    self.page_documents(name)
                        .into_iter()
                        .map(|document| {
                            (
                                document.title.clone().unwrap_or_default(),
                                document.path().to_path_buf(),
                            )
                        })
                        .collect(),
                );
            }
            _ => None,
        };
        let Some(document) = document else {
            let path = self.new_page_path(name);
            return Summary::Page(PageSummary {
                title: match page {
                    PageId::Journal(date) => self.journal_formats.title.format(date),
                    _ => remove_boundary_slashes(name).to_owned(),
                },
                exists: path
                    .as_ref()
                    .is_some_and(|path| self.documents.contains_key(path)),
                path,
                aliases: Vec::new(),
                first_lines: Vec::new(),
                references,
            });
        };
        let title = document.title.clone().unwrap_or_default();
        let after_properties = match document.document.parsed.blocks.first() {
            Some(Block::Properties {
                span: Some(span), ..
            }) => span.1,
            _ => 0,
        };
        let hidden = hidden_lines(document);
        let first = document.lines.position(after_properties).line;
        let first_lines = (first..document.lines.line_count())
            .filter(|line| !hidden.contains(line))
            .filter(|line| {
                document.lines.line_range(*line, document.source()).0 >= after_properties
            })
            .map(|line| line_text(document, line))
            .filter(|text| !text.trim().is_empty())
            .take(SUMMARY_LINES)
            .map(str::to_owned)
            .collect();
        Summary::Page(PageSummary {
            aliases: page_alias_spellings(&document.document, &title),
            title,
            path: Some(document.path().to_path_buf()),
            exists: true,
            first_lines,
            references,
        })
    }

    /// A summary of the block Logseq gives an id.
    pub fn block_summary(&self, id: &str) -> Summary {
        let Some(BlockLocation { document, site }) = self.block(id) else {
            return Summary::MissingBlock;
        };
        let outline = Outline::from_blocks(&document.document.parsed.blocks);
        let position = outline
            .blocks
            .iter()
            .position(|block| block.start == Some(site.block_start));
        let line_of = |offset: usize| document.lines.position(offset).line;
        let first = line_of(site.block_start);
        let indent_of = |line: usize| {
            let text = line_text(document, line);
            text.len() - text.trim_start().len()
        };
        // A block ends where the next bullet of the outline starts.
        let end = position
            .and_then(|position| outline.blocks.get(position + 1)?.start)
            .map_or_else(
                || match position {
                    Some(_) => document.lines.line_count(),
                    None => first + 1,
                },
                line_of,
            );
        let hidden = hidden_lines(document);
        let indent = indent_of(first);
        let mut text: Vec<String> = (first..end.max(first + 1))
            .filter(|line| !hidden.contains(line))
            .map(|line| without_bullet(line_text(document, line), indent, line == first).to_owned())
            .collect();
        while text.last().is_some_and(|line| line.trim().is_empty()) {
            text.pop();
        }
        if text.len() > SUMMARY_LINES {
            text.truncate(SUMMARY_LINES);
            text.push("…".to_owned());
        }
        let mut parents: Vec<String> = position
            .into_iter()
            .flat_map(|position| outline.ancestors(position))
            .filter_map(|parent| outline.blocks[parent].start)
            .map(|start| {
                let line = line_of(start);
                without_bullet(line_text(document, line), indent_of(line), true).to_owned()
            })
            .collect();
        parents.reverse();
        Summary::Block(BlockSummary {
            path: document.path().to_path_buf(),
            page: document.title.clone(),
            parents,
            text,
            references: self.occurrences_of_block(&site.id).len(),
        })
    }

    /// The range of the occurrence at an offset of a document and a summary of the page or
    /// block it names. `None` in plain text and on a property key.
    pub fn summary_at(&self, path: &Path, offset: usize) -> Option<(SourceRange, Summary)> {
        let (_, occurrence) = self.occurrence_at(path, offset)?;
        let summary = match &occurrence.target {
            Target::Page(name) => self.page_summary(name),
            Target::Block(id) => self.block_summary(id),
            Target::Property(_) => return None,
        };
        Some((occurrence.range, summary))
    }
}
