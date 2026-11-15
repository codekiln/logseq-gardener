//! Where a page, a block or a property key is named across the garden.

use std::path::{Path, PathBuf};

use super::{
    GardenIndex, IndexedDocument, Occurrence, OccurrenceKind, PageId, SourceRange, Target,
};
use crate::ast::Block;
use crate::page_names::{lookup_key, reference_key};

/// Which of a page's names an occurrence writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamedBy {
    /// The page's title, in any letter case.
    Title,
    /// One of the names in the page's `alias::` property.
    Alias,
    /// A journal's date in a format other than its title, or the date of a journal that
    /// has no file in any format.
    Date,
}

/// One place that names a page.
#[derive(Debug, Clone, Copy)]
pub struct PageReference<'a> {
    pub document: &'a IndexedDocument,
    pub occurrence: &'a Occurrence,
    pub named_by: NamedBy,
    /// Whether this is a name in the page's own `alias::` property, which gives the page
    /// the name.
    pub declaration: bool,
}

/// What find references lists the places of.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    Page(PageId),
    /// A block, by its id in lower case.
    Block(String),
    /// A property, by its key in lower case.
    Property(String),
}

/// A range of a file of the garden.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reference {
    /// The path relative to the garden root.
    pub path: PathBuf,
    pub range: SourceRange,
}

/// Whether an occurrence sits in the value of the document's leading `alias::` property.
fn in_own_alias(document: &IndexedDocument, occurrence: &Occurrence) -> bool {
    let Some(Block::Properties {
        span: Some(span), ..
    }) = document.document.parsed.blocks.first()
    else {
        return false;
    };
    occurrence.range.end <= span.1
        && document.properties.iter().any(|line| {
            line.key == "alias"
                && line.value_range.start <= occurrence.range.start
                && occurrence.range.end <= line.value_range.end
        })
}

impl GardenIndex {
    /// Every place that names a page by its title, by one of its aliases or, for a journal,
    /// by its date, in path and then source order. The names in the page's own `alias::`
    /// property are included and marked as declarations.
    pub fn page_references(&self, page: &PageId) -> Vec<PageReference<'_>> {
        let own = match page {
            PageId::Document(path) => self.documents.get(path),
            _ => None,
        };
        let title = own.and_then(|document| document.title.as_deref().map(lookup_key));
        self.occurrences_of_page(page)
            .into_iter()
            .map(|(document, occurrence)| {
                let key = match &occurrence.target {
                    Target::Page(name) => reference_key(name),
                    _ => String::new(),
                };
                let named_by = match (page, own) {
                    (PageId::Journal(_), _) => NamedBy::Date,
                    (_, Some(_)) if title.as_deref() == Some(key.as_str()) => NamedBy::Title,
                    (_, Some(own)) if own.aliases.contains(&key) => NamedBy::Alias,
                    (_, Some(own)) if own.journal.is_some() => NamedBy::Date,
                    _ => NamedBy::Title,
                };
                let declaration = own.is_some_and(|own| {
                    own.path() == document.path() && in_own_alias(document, occurrence)
                }) && named_by == NamedBy::Alias;
                PageReference {
                    document,
                    occurrence,
                    named_by,
                    declaration,
                }
            })
            .collect()
    }

    /// Every line that writes a property key, as its key occurrence, in path and then source
    /// order.
    pub fn occurrences_of_property(&self, key: &str) -> Vec<(&IndexedDocument, &Occurrence)> {
        let target = Target::Property(key.trim().to_lowercase());
        if !matches!(&target, Target::Property(key) if self.property_keys.contains_key(key)) {
            return Vec::new();
        }
        self.documents
            .values()
            .flat_map(|document| {
                document
                    .occurrences
                    .iter()
                    .filter(|occurrence| {
                        occurrence.kind == OccurrenceKind::PropertyKey
                            && occurrence.target == target
                    })
                    .map(move |occurrence| (document, occurrence))
            })
            .collect()
    }

    /// What find references is about at an offset of a document: the page, block or
    /// property key that the occurrence there names, and the document's own page anywhere
    /// else. `None` when the index holds no document at the path.
    pub fn subject_at(&self, path: &Path, offset: usize) -> Option<Subject> {
        let document = self.documents.get(path)?;
        Some(match document.occurrence_at(offset) {
            Some(occurrence) => match &occurrence.target {
                Target::Page(name) => Subject::Page(self.page_id(name)),
                Target::Block(id) => Subject::Block(id.clone()),
                Target::Property(key) => Subject::Property(key.clone()),
            },
            None => Subject::Page(PageId::Document(path.to_path_buf())),
        })
    }

    /// Every range that names a subject, in path and then source order.
    ///
    /// With `include_declaration`, the list starts with where the subject gets its name: the
    /// start of a page's file and the names in its `alias::` property, or the `id::` value of
    /// the block Logseq gives a block id. A page without a file and a property key have none.
    pub fn references(&self, subject: &Subject, include_declaration: bool) -> Vec<Reference> {
        let at = |document: &IndexedDocument, range| Reference {
            path: document.path().to_path_buf(),
            range,
        };
        let mut declarations = Vec::new();
        let mut found = Vec::new();
        match subject {
            Subject::Page(page) => {
                if let PageId::Document(path) = page
                    && let Some(document) = self.documents.get(path)
                {
                    declarations.push(at(document, SourceRange::new(0, 0)));
                }
                for reference in self.page_references(page) {
                    let place = at(reference.document, reference.occurrence.range);
                    if reference.declaration {
                        declarations.push(place);
                    } else {
                        found.push(place);
                    }
                }
            }
            Subject::Block(id) => {
                if let Some(block) = self.block(id) {
                    declarations.push(at(block.document, block.site.range));
                }
                found.extend(
                    self.occurrences_of_block(id)
                        .into_iter()
                        .map(|(document, occurrence)| at(document, occurrence.range)),
                );
            }
            Subject::Property(key) => found.extend(
                self.occurrences_of_property(key)
                    .into_iter()
                    .map(|(document, occurrence)| at(document, occurrence.range)),
            ),
        }
        if include_declaration {
            declarations.extend(found);
            declarations
        } else {
            found
        }
    }
}
