//! Find the source file of a page by its title, an alias or a journal date.
//!
//! The catalog covers every page and journal in the garden, with no namespace
//! selection or visibility check: it answers where a page lives on disk for a
//! local reader, not what a published site shows. Names match the way page
//! links resolve: case-insensitively after Unicode normalization, then as a
//! journal date in any format Logseq accepts.
//!
//! ```
//! use std::path::Path;
//! use logseq_gardener_sdk::page_lookup::{MatchedBy, PageCatalog};
//! use logseq_gardener_sdk::page_titles::FilenameFormat;
//!
//! let root = std::env::temp_dir().join(format!("lsg-lookup-doc-{}", std::process::id()));
//! std::fs::create_dir_all(root.join("pages")).unwrap();
//! std::fs::write(root.join("pages/Tools___Neovim.md"), "alias:: nvim\n\n- Editor\n").unwrap();
//!
//! let (catalog, diagnostics) = PageCatalog::load(&root, FilenameFormat::TripleLowbar).unwrap();
//! let found = catalog.find("NVIM");
//! assert_eq!(found[0].title, "Tools/Neovim");
//! assert_eq!(found[0].relative_path, Path::new("pages/Tools___Neovim.md"));
//! assert_eq!(found[0].matched_by, MatchedBy::Alias);
//! assert!(diagnostics.is_empty());
//! # std::fs::remove_dir_all(&root).unwrap();
//! ```

use std::path::{Path, PathBuf};

use crate::garden::{DocumentKind, Garden, GardenLoadError, load_garden};
use crate::garden_settings::GardenSettings;
use crate::journals::{JournalFormats, journal_identity};
use crate::page_names::{PageNameIndex, lookup_key, page_aliases};
use crate::page_titles::{FilenameFormat, document_title, page_title};

/// How a query reached a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchedBy {
    /// The query spells the page's title.
    Title,
    /// The query spells one of the page's leading `alias::` names.
    Alias,
    /// The query is a date in a journal format and names that day's journal.
    JournalDate,
}

impl MatchedBy {
    /// The lower-case name used in reports: `title`, `alias` or `journal-date`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Alias => "alias",
            Self::JournalDate => "journal-date",
        }
    }
}

/// One page a query names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLocation {
    /// The page's display title.
    pub title: String,
    /// Source path relative to the garden root.
    pub relative_path: PathBuf,
    /// Whether the source is a page or a journal.
    pub kind: DocumentKind,
    /// How the query reached this page.
    pub matched_by: MatchedBy,
}

/// A source file left out of the catalog, or a settings value replaced by its default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDiagnostic {
    /// Path relative to the garden root.
    pub source: PathBuf,
    /// What was left out and why.
    pub message: String,
}

#[derive(Debug, Clone)]
struct CatalogPage {
    title: String,
    relative_path: PathBuf,
    kind: DocumentKind,
    aliases: Vec<String>,
}

/// Titles, aliases and journal dates of every page in a garden, with their source paths.
#[derive(Debug, Clone)]
pub struct PageCatalog {
    pages: Vec<CatalogPage>,
    names: PageNameIndex<usize>,
}

impl PageCatalog {
    /// Read the garden at `root` and its `logseq/config.edn` journal formats, then catalog it.
    pub fn load(
        root: impl AsRef<Path>,
        format: FilenameFormat,
    ) -> Result<(Self, Vec<CatalogDiagnostic>), GardenLoadError> {
        let root = root.as_ref();
        let mut garden = load_garden(root)?;
        let (settings, settings_diagnostics) = GardenSettings::read(root);
        let journal_formats = JournalFormats::from_settings(&settings);
        let shadowed = garden.remove_shadowed_documents(|document| {
            document_title(document, format, &journal_formats).map(|title| lookup_key(&title))
        });
        let (catalog, mut diagnostics) = Self::build(&garden, format, &journal_formats);
        diagnostics.splice(
            0..0,
            shadowed.into_iter().map(|source| CatalogDiagnostic {
                source,
                message: "page left out: a page in pages/ or journals/ has the same title"
                    .to_owned(),
            }),
        );
        diagnostics.splice(
            0..0,
            settings_diagnostics
                .into_iter()
                .map(|message| CatalogDiagnostic {
                    source: "logseq/config.edn".into(),
                    message,
                }),
        );
        Ok((catalog, diagnostics))
    }

    /// Catalog loaded documents without filesystem access.
    ///
    /// A page whose title cannot be derived, or a journal whose file name is not
    /// a date, is left out with a diagnostic; the rest stay findable.
    pub fn build(
        garden: &Garden,
        format: FilenameFormat,
        journal_formats: &JournalFormats,
    ) -> (Self, Vec<CatalogDiagnostic>) {
        let mut pages = Vec::new();
        let mut names = PageNameIndex::with_journal_formats(journal_formats.clone());
        let mut diagnostics = Vec::new();
        for document in &garden.documents {
            let (title, date) = if document.kind == DocumentKind::Journal {
                match journal_identity(document, journal_formats) {
                    Ok(identity) => (identity.title, Some(identity.date)),
                    Err(error) => {
                        diagnostics.push(CatalogDiagnostic {
                            source: document.relative_path.clone(),
                            message: format!("journal left out: {error}"),
                        });
                        continue;
                    }
                }
            } else {
                match page_title(document, format) {
                    Ok(title) => (title, None),
                    Err(error) => {
                        diagnostics.push(CatalogDiagnostic {
                            source: document.relative_path.clone(),
                            message: format!("page left out: {error}"),
                        });
                        continue;
                    }
                }
            };
            let id = pages.len();
            names.insert(id, &title, document);
            if let Some(date) = date {
                names.insert_journal(id, date);
            }
            pages.push(CatalogPage {
                aliases: page_aliases(document, &title),
                title,
                relative_path: document.relative_path.clone(),
                kind: document.kind,
            });
        }
        (Self { pages, names }, diagnostics)
    }

    /// Every page `name` reaches, in source-path order: none, one, or several when
    /// pages share a title or alias.
    pub fn find(&self, name: &str) -> Vec<PageLocation> {
        let key = lookup_key(name);
        self.names
            .candidates(name)
            .iter()
            .map(|&id| {
                let page = &self.pages[id];
                let matched_by = if lookup_key(&page.title) == key {
                    MatchedBy::Title
                } else if page.aliases.contains(&key) {
                    MatchedBy::Alias
                } else {
                    MatchedBy::JournalDate
                };
                PageLocation {
                    title: page.title.clone(),
                    relative_path: page.relative_path.clone(),
                    kind: page.kind,
                    matched_by,
                }
            })
            .collect()
    }

    /// The number of cataloged pages and journals.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether the garden has no cataloged pages.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::GardenDocument;

    fn document(path: &str, source: &str) -> GardenDocument {
        let kind = if path.starts_with("journals/") {
            DocumentKind::Journal
        } else {
            DocumentKind::Page
        };
        GardenDocument {
            relative_path: path.into(),
            kind,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    fn catalog(documents: Vec<GardenDocument>) -> (PageCatalog, Vec<CatalogDiagnostic>) {
        PageCatalog::build(
            &Garden { documents },
            FilenameFormat::TripleLowbar,
            &JournalFormats::default(),
        )
    }

    fn paths(found: &[PageLocation]) -> Vec<&Path> {
        found
            .iter()
            .map(|page| page.relative_path.as_path())
            .collect()
    }

    #[test]
    fn finds_titles_case_insensitively_including_private_pages() {
        let (catalog, diagnostics) = catalog(vec![
            document("pages/My___AI.md", "- Notes\n"),
            document("pages/Secret.md", "public:: false\n\n- hidden\n"),
        ]);
        assert!(diagnostics.is_empty());
        let found = catalog.find(" my/ai ");
        assert_eq!(paths(&found), [Path::new("pages/My___AI.md")]);
        assert_eq!(found[0].title, "My/AI");
        assert_eq!(found[0].matched_by, MatchedBy::Title);
        assert_eq!(
            paths(&catalog.find("secret")),
            [Path::new("pages/Secret.md")]
        );
        assert!(catalog.find("missing").is_empty());
    }

    #[test]
    fn title_property_and_alias_both_reach_the_page() {
        let (catalog, _) = catalog(vec![document(
            "pages/nvim.md",
            "title:: Neovim\nalias:: [[vim]], Vi IMproved\n\n- Editor\n",
        )]);
        assert_eq!(catalog.find("neovim")[0].matched_by, MatchedBy::Title);
        assert_eq!(catalog.find("Vim")[0].matched_by, MatchedBy::Alias);
        assert_eq!(catalog.find("vi improved")[0].matched_by, MatchedBy::Alias);
        assert!(catalog.find("nvim").is_empty());
    }

    #[test]
    fn shared_alias_returns_every_page_in_path_order() {
        let (catalog, _) = catalog(vec![
            document("pages/A.md", "alias:: shared\n\n- a\n"),
            document("pages/B.md", "alias:: shared\n\n- b\n"),
        ]);
        assert_eq!(
            paths(&catalog.find("Shared")),
            [Path::new("pages/A.md"), Path::new("pages/B.md")]
        );
    }

    #[test]
    fn journals_answer_to_their_title_and_other_date_spellings() {
        let (catalog, diagnostics) = catalog(vec![
            document("journals/2025_02_01.md", "- day\n"),
            document("journals/notes.md", "- not a date\n"),
        ]);
        let by_title = catalog.find("Feb 1st, 2025");
        assert_eq!(paths(&by_title), [Path::new("journals/2025_02_01.md")]);
        assert_eq!(by_title[0].matched_by, MatchedBy::Title);
        assert_eq!(by_title[0].kind, DocumentKind::Journal);
        assert_eq!(
            catalog.find("2025-02-01")[0].matched_by,
            MatchedBy::JournalDate
        );
        assert_eq!(catalog.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source, Path::new("journals/notes.md"));
        assert!(diagnostics[0].message.starts_with("journal left out"));
    }

    #[test]
    fn a_page_without_a_title_is_left_out_with_a_diagnostic() {
        let (catalog, diagnostics) = catalog(vec![
            document("pages/   .md", "- blank\n"),
            document("pages/Kept.md", "- kept\n"),
        ]);
        assert_eq!(catalog.len(), 1);
        assert_eq!(diagnostics[0].source, Path::new("pages/   .md"));
        assert!(diagnostics[0].message.starts_with("page left out"));
    }
    #[test]
    fn load_finds_pages_outside_pages_and_names_shadowed_files() {
        let root = std::env::temp_dir().join(format!(
            "lsg-lookup-folders-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(root.join("pages")).unwrap();
        std::fs::create_dir_all(root.join("mise-tasks/microfreak")).unwrap();
        std::fs::write(root.join("pages/PARAMETERS.md"), "- page\n").unwrap();
        std::fs::write(root.join("mise-tasks/PARAMETERS.md"), "- copy\n").unwrap();
        std::fs::write(root.join("mise-tasks/microfreak/Knobs.md"), "- knobs\n").unwrap();
        let (catalog, diagnostics) =
            PageCatalog::load(&root, FilenameFormat::TripleLowbar).unwrap();
        assert_eq!(
            paths(&catalog.find("parameters")),
            [Path::new("pages/PARAMETERS.md")]
        );
        assert_eq!(
            paths(&catalog.find("Knobs")),
            [Path::new("mise-tasks/microfreak/Knobs.md")]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].source, Path::new("mise-tasks/PARAMETERS.md"));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
