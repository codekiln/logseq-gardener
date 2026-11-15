//! The problems in one document: references Logseq cannot follow and block ids it will
//! change.
//!
//! [`GardenIndex::problems`] reads only what the index already holds, so an editor can ask
//! again after every change.

use std::fmt;
use std::path::{Path, PathBuf};

use super::{GardenIndex, OccurrenceKind, PageId, SourceRange, Target};
use crate::outline::uuid;

/// How serious a [`Problem`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Logseq shows or stores something other than what the text says.
    Error,
    /// The text works, and its author probably meant something else.
    Warning,
}

/// What is wrong at a [`Problem`]'s range.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProblemKind {
    /// A `((uuid))` or `{{embed ((uuid))}}` whose id no block writes. Holds the id.
    MissingBlock(String),
    /// An `id::` that other blocks also write. Holds the id. Logseq keeps the id on the
    /// block it reads first and gives each other block a new id.
    DuplicateBlockId(String),
    /// A `{{embed [[page]]}}` whose page has no file. Holds the name as written.
    MissingEmbeddedPage(String),
}

impl ProblemKind {
    /// A short name for the kind, the same for every problem of that kind.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingBlock(_) => "missing-block",
            Self::DuplicateBlockId(_) => "duplicate-block-id",
            Self::MissingEmbeddedPage(_) => "missing-embedded-page",
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            Self::MissingBlock(_) | Self::DuplicateBlockId(_) => Severity::Error,
            Self::MissingEmbeddedPage(_) => Severity::Warning,
        }
    }
}

/// One sentence that says what is wrong, without the related locations.
impl fmt::Display for ProblemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBlock(id) => write!(f, "No block has the id {id}"),
            Self::DuplicateBlockId(id) => write!(f, "More than one block has the id {id}"),
            Self::MissingEmbeddedPage(name) => {
                write!(f, "The embedded page {name} has no file")
            }
        }
    }
}

/// A place in the garden that a [`Problem`] involves besides its own range.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelatedLocation {
    /// The document's path relative to the garden root.
    pub path: PathBuf,
    pub range: SourceRange,
}

/// One problem in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub kind: ProblemKind,
    /// The source the problem marks: the whole reference or embed, or the value of an `id::`.
    pub range: SourceRange,
    /// For a duplicated id, the value of every other `id::` that writes it, in the order
    /// Logseq reads them. Empty for the other kinds.
    pub related: Vec<RelatedLocation>,
}

impl Problem {
    pub fn severity(&self) -> Severity {
        self.kind.severity()
    }

    /// One sentence that says what is wrong.
    pub fn message(&self) -> String {
        self.kind.to_string()
    }
}

impl GardenIndex {
    /// The problems in the document at `path`, in source order: block references and block
    /// embeds whose block is missing, `id::` values that other blocks also write, and page
    /// embeds whose page has no file. A page link or tag to a page without a file is how
    /// Logseq creates a page, so it is no problem. A path without a document gives nothing.
    pub fn problems(&self, path: &Path) -> Vec<Problem> {
        let Some(document) = self.document(path) else {
            return Vec::new();
        };
        let mut problems = Vec::new();
        for occurrence in &document.occurrences {
            let kind = match (occurrence.kind, &occurrence.target) {
                (
                    OccurrenceKind::BlockReference | OccurrenceKind::BlockEmbed,
                    Target::Block(id),
                ) if uuid(id).is_some() && !self.block_ids.contains_key(id) => {
                    ProblemKind::MissingBlock(id.clone())
                }
                (OccurrenceKind::PageEmbed, Target::Page(name))
                    if matches!(self.page_id(name), PageId::Journal(_) | PageId::Name(_)) =>
                {
                    ProblemKind::MissingEmbeddedPage(name.trim().to_owned())
                }
                _ => continue,
            };
            problems.push(Problem {
                kind,
                range: occurrence.range,
                related: Vec::new(),
            });
        }
        for site in &document.block_ids {
            if self
                .block_ids
                .get(&site.id)
                .is_none_or(|sites| sites.len() < 2)
            {
                continue;
            }
            let related = self
                .block_id_sites(&site.id)
                .into_iter()
                .filter(|other| {
                    other.document.path() != path || other.site.block_start != site.block_start
                })
                .map(|other| RelatedLocation {
                    path: other.document.path().to_path_buf(),
                    range: other.site.range,
                })
                .collect();
            problems.push(Problem {
                kind: ProblemKind::DuplicateBlockId(site.id.clone()),
                range: site.range,
                related,
            });
        }
        problems.sort_by_key(|problem| (problem.range.start, problem.range.end));
        problems
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::index::CONFIG_PATH;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    /// A temporary folder, removed when the value drops.
    struct Folder(PathBuf);

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A garden with these files, loaded.
    fn garden(files: &[(&str, &str)]) -> (Folder, GardenIndex) {
        let root = std::env::temp_dir().join(format!(
            "lsg-problems-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let folder = Folder(root);
        let config = (CONFIG_PATH, "{:file/name-format :triple-lowbar}\n");
        for (path, text) in files.iter().chain([&config]) {
            let path = folder.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let index = GardenIndex::load(&folder.0).unwrap();
        (folder, index)
    }

    const DEAD: &str = "66f1a2b0-0000-4000-8000-00000000dead";
    const SHARED: &str = "66f1a2b0-0000-4000-8000-000000000003";

    fn summary(index: &GardenIndex, path: &str) -> Vec<(String, Severity, String, Vec<String>)> {
        let document = index.document(path.as_ref());
        index
            .problems(path.as_ref())
            .into_iter()
            .map(|problem| {
                let related = problem
                    .related
                    .iter()
                    .map(|place| {
                        let other = index.document(&place.path).unwrap();
                        format!(
                            "{}:{}",
                            place.path.display(),
                            place.range.text(other.source())
                        )
                    })
                    .collect();
                (
                    problem.kind.code().to_owned(),
                    problem.severity(),
                    problem.range.text(document.unwrap().source()).to_owned(),
                    related,
                )
            })
            .collect()
    }

    #[test]
    fn reports_missing_blocks_duplicate_ids_and_embeds_of_missing_pages() {
        let (_folder, mut index) = garden(&[
            (
                "pages/Notes.md",
                &format!(
                    "- (({DEAD})) and {{{{embed (({DEAD}))}}}}\n- {{{{embed [[Quince]]}}}} {{{{embed [[Pyrus]]}}}}\n- Kept\n  id:: {SHARED}\n"
                ),
            ),
            (
                "pages/Pear.md",
                &format!("alias:: Pyrus\n\n- Ripens\n  id:: {SHARED}\n"),
            ),
        ]);
        assert_eq!(
            summary(&index, "pages/Notes.md"),
            [
                (
                    "missing-block".to_owned(),
                    Severity::Error,
                    format!("(({DEAD}))"),
                    vec![]
                ),
                (
                    "missing-block".to_owned(),
                    Severity::Error,
                    format!("{{{{embed (({DEAD}))}}}}"),
                    vec![]
                ),
                (
                    "missing-embedded-page".to_owned(),
                    Severity::Warning,
                    "{{embed [[Quince]]}}".to_owned(),
                    vec![]
                ),
                (
                    "duplicate-block-id".to_owned(),
                    Severity::Error,
                    SHARED.to_owned(),
                    vec![format!("pages/Pear.md:{SHARED}")]
                ),
            ]
        );
        assert_eq!(
            summary(&index, "pages/Pear.md"),
            [(
                "duplicate-block-id".to_owned(),
                Severity::Error,
                SHARED.to_owned(),
                vec![format!("pages/Notes.md:{SHARED}")]
            )]
        );
        let messages: Vec<String> = index
            .problems("pages/Notes.md".as_ref())
            .iter()
            .map(Problem::message)
            .collect();
        assert_eq!(messages[0], format!("No block has the id {DEAD}"));
        assert_eq!(messages[2], "The embedded page Quince has no file");
        assert_eq!(
            messages[3],
            format!("More than one block has the id {SHARED}")
        );

        // A buffer of another page writes the missing id and drops the shared one.
        index.set_buffer(
            "pages/Pear.md".as_ref(),
            format!("alias:: Pyrus\n\n- Ripens\n  id:: {DEAD}\n"),
        );
        assert_eq!(
            summary(&index, "pages/Notes.md"),
            [(
                "missing-embedded-page".to_owned(),
                Severity::Warning,
                "{{embed [[Quince]]}}".to_owned(),
                vec![]
            )]
        );
        assert_eq!(summary(&index, "pages/Pear.md"), []);
        assert_eq!(index.problems("pages/Absent.md".as_ref()), []);
    }

    #[test]
    fn a_page_without_a_file_is_no_problem_outside_an_embed() {
        let (_folder, index) = garden(&[(
            "pages/Notes.md",
            "tags:: Quince, [[Medlar]]\n\n- [[Fruit/Quince]] #quince #[[two words]] [label]([[Sloe]])\n- [[Jan 5th, 2026]] ((not an id)) {{embed ((also no id))}}\n",
        )]);
        assert_eq!(index.problems("pages/Notes.md".as_ref()), []);
    }

    #[test]
    fn two_blocks_of_one_document_with_the_same_id_name_each_other() {
        let (_folder, index) = garden(&[(
            "pages/Notes.md",
            &format!("- One\n  id:: {SHARED}\n- Two\n  id:: {SHARED}\n- Three\n  id:: {SHARED}\n"),
        )]);
        let problems = index.problems("pages/Notes.md".as_ref());
        assert_eq!(problems.len(), 3);
        for problem in &problems {
            assert_eq!(problem.related.len(), 2);
            assert!(
                problem
                    .related
                    .iter()
                    .all(|place| place.range != problem.range)
            );
        }
    }
}
