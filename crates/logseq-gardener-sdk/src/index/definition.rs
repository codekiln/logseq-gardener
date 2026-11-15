//! Where a link leads: the file and range an editor opens for go to definition.

use std::path::{Path, PathBuf};

use super::{GardenIndex, Occurrence, PageId, SourceRange, Target};
use crate::file_names::{journal_file_path, page_file_path};

/// A place in a file of the garden.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Destination {
    /// The path relative to the garden root.
    pub path: PathBuf,
    /// The start of a page, or the first line of a block from its dash. Empty at offset 0
    /// for a file the index holds no document for.
    pub range: SourceRange,
    /// Whether the index holds a document at the path. `false` for the file Logseq would
    /// create for a page or journal that has none.
    pub exists: bool,
}

impl GardenIndex {
    /// The file Logseq creates for a page name that no file answers to:
    /// `journals/<date in :journal/file-name-format>.md` for a name that spells a date, and
    /// `pages/<file name>.md` otherwise, by [`crate::file_names::page_file_name`]. `None`
    /// for a name with no characters to make a file name from.
    pub fn new_page_path(&self, name: &str) -> Option<PathBuf> {
        match self.page_id(name) {
            PageId::Journal(date) => Some(journal_file_path(date, &self.journal_formats)),
            _ => page_file_path(name, self.settings.filename_format),
        }
    }

    fn page_start(&self, path: &Path) -> Destination {
        Destination {
            path: path.to_path_buf(),
            range: SourceRange::new(0, 0),
            exists: self.documents.contains_key(path),
        }
    }

    /// Where a target leads. A page name leads to the start of its page's file, an alias to
    /// the page that declares it, a name several files answer to leads to each of them, and
    /// a name no file answers to leads to [`Self::new_page_path`]. A block id leads to the
    /// first line of the block Logseq gives the id. A property key, and a block id no block
    /// writes, lead nowhere.
    pub fn destinations(&self, target: &Target) -> Vec<Destination> {
        match target {
            Target::Page(name) => match self.page_id(name) {
                PageId::Document(path) => vec![self.page_start(&path)],
                PageId::Ambiguous(_) => self
                    .page_documents(name)
                    .into_iter()
                    .map(|document| self.page_start(document.path()))
                    .collect(),
                PageId::Journal(_) | PageId::Name(_) => self
                    .new_page_path(name)
                    .map(|path| self.page_start(&path))
                    .into_iter()
                    .collect(),
            },
            Target::Block(id) => self
                .block(id)
                .map(|block| {
                    let source = block.document.source();
                    let line = block.document.lines.position(block.site.block_start).line;
                    let (start, end) = block.document.lines.line_range(line, source);
                    let text = &source[start..end];
                    let dash = start + (text.len() - text.trim_start().len());
                    Destination {
                        path: block.document.path().to_path_buf(),
                        range: SourceRange::new(dash, end),
                        exists: true,
                    }
                })
                .into_iter()
                .collect(),
            Target::Property(_) => Vec::new(),
        }
    }

    /// The occurrence at an offset of a document and where it leads, or `None` when the
    /// offset is in no occurrence.
    pub fn definition_at(
        &self,
        path: &Path,
        offset: usize,
    ) -> Option<(&Occurrence, Vec<Destination>)> {
        let (_, occurrence) = self.occurrence_at(path, offset)?;
        Some((occurrence, self.destinations(&occurrence.target)))
    }
}
